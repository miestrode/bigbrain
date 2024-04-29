use std::{
    collections::HashSet,
    fmt::{self, Display, Write},
    hash::Hash,
    iter::Sum,
    time::Instant,
};

use good_lp::{coin_cbc, variable, Expression, ProblemVariables, Solution, SolverModel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Output {
    Zero,
    One,
    DontCare,
}

impl Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(match self {
            Output::Zero => '0',
            Output::One => '1',
            Output::DontCare => '-',
        })
    }
}

#[derive(Clone, Eq)]
pub struct Implicant<const N: usize> {
    values: [Output; N],
    constituents: HashSet<usize>,
}

impl<const N: usize> Hash for Implicant<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.values.hash(state);
    }
}

impl<const N: usize> PartialEq for Implicant<N> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl<const N: usize> PartialOrd for Implicant<N> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Ord for Implicant<N> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.constituent_count().cmp(&other.constituent_count())
    }
}

impl<const N: usize> Implicant<N> {
    fn constituent_count(&self) -> usize {
        self.constituents.len()
    }

    fn from_table_idx(idx: usize, is_minterm: bool) -> Self {
        Self {
            values: (0..N)
                .map(|var| {
                    if (idx >> var) % 2 == 1 {
                        Output::One
                    } else {
                        Output::Zero
                    }
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            constituents: if is_minterm {
                HashSet::from([idx])
            } else {
                HashSet::new()
            },
        }
    }

    fn try_merge(&self, other: &Self) -> Option<Self> {
        let mut diff_idx = None;

        for idx in 0..N {
            match (self.values[idx], other.values[idx]) {
                (a, b) if a == b => continue,
                (Output::Zero, Output::One) | (Output::One, Output::Zero) if diff_idx.is_none() => {
                    diff_idx = Some(idx)
                }
                _ => return None,
            }
        }

        diff_idx.map(|diff_idx| {
            let mut result = self.clone();

            result.constituents.extend(other.constituents.iter());
            result.values[diff_idx] = Output::DontCare;

            result
        })
    }
}

impl<const N: usize> Display for Implicant<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            &self
                .values
                .iter()
                .rev()
                .map(ToString::to_string)
                .collect::<String>(),
        )
    }
}

pub struct Table<const N: usize>
where
    [(); 1 << N]: Sized,
{
    pub outputs: [Output; 1 << N],
}

impl<const N: usize> Table<N>
where
    [(); 1 << N]: Sized,
{
    const ENTRIES: usize = 1 << N;

    pub fn minterms(&self) -> HashSet<usize> {
        self.outputs
            .iter()
            .enumerate()
            .filter_map(|(idx, output)| matches!(output, Output::One).then_some(idx))
            .collect()
    }

    pub fn is_minterm(&self, idx: usize) -> bool {
        matches!(self.outputs[idx], Output::One)
    }

    pub fn prime_implicants(&self) -> Vec<Implicant<N>> {
        let mut implicants = (0..Self::ENTRIES)
            .filter(|&idx| !matches!(self.outputs[idx], Output::Zero))
            .map(|idx| Implicant::from_table_idx(idx, matches!(self.outputs[idx], Output::One)))
            .collect::<Vec<_>>();
        let mut prime_implicants = Vec::new();

        loop {
            let mut merged = vec![false; implicants.len()];
            let mut next_implicants = HashSet::with_capacity(implicants.capacity());

            for first_idx in 0..implicants.len() {
                let first = &implicants[first_idx];

                for second_idx in first_idx..implicants.len() {
                    let second = &implicants[second_idx];

                    if let Some(merge) = first.try_merge(second) {
                        next_implicants.insert(merge);

                        merged[first_idx] = true;
                        merged[second_idx] = true;
                    }
                }

                if !merged[first_idx] {
                    prime_implicants.push(first.clone());
                }
            }

            if next_implicants.is_empty() {
                break;
            }

            implicants = next_implicants.into_iter().collect();
        }

        prime_implicants
    }

    pub fn minimize(&self) -> Vec<Implicant<N>> {
        fn cover<T: Eq + Hash>(universe: &HashSet<T>, sets: &[&HashSet<T>]) -> Vec<usize> {
            let mut problem = ProblemVariables::new();

            let include_vars = problem.add_vector(variable().min(0).max(1).integer(), sets.len());

            let mut model = problem
                .minimise(Expression::sum(include_vars.iter()))
                .using(coin_cbc);

            model.set_parameter("loglevel", "0");

            for item in universe {
                model = model.with(
                    Expression::sum(sets.iter().zip(&include_vars).filter_map(
                        |(set, include_var)| set.contains(item).then_some(include_var),
                    ))
                    .geq(1),
                );
            }

            let solution = model.solve().unwrap();

            include_vars
                .iter()
                .enumerate()
                .filter_map(|(idx, include_var)| {
                    (solution.value(*include_var) > 0.0).then_some(idx)
                })
                .collect()
        }

        let mut primes = self.prime_implicants();

        let sets = primes
            .iter()
            .map(|prime| &prime.constituents)
            .collect::<Vec<_>>();
        let universe = self.minterms();

        cover(&universe, &sets)
            .iter()
            .rev()
            .map(|&idx| primes.remove(idx))
            .collect()
    }
}
