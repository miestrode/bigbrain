use std::{
    collections::{BinaryHeap, HashSet},
    fmt::{self, Display, Write},
    hash::Hash,
};

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

    fn from_table_idx(idx: usize) -> Self {
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
            constituents: HashSet::from([idx]),
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

    pub fn minterms(&self) -> usize {
        self.outputs
            .iter()
            .map(|output| if matches!(output, Output::One) { 1 } else { 0 })
            .sum()
    }

    pub fn is_minterm(&self, idx: usize) -> bool {
        matches!(self.outputs[idx], Output::One)
    }

    pub fn prime_implicants(&self) -> HashSet<Implicant<N>> {
        let mut implicants = (0..Self::ENTRIES)
            .filter(|&idx| !matches!(self.outputs[idx], Output::Zero))
            .map(Implicant::from_table_idx)
            .collect::<Vec<_>>();
        let mut prime_implicants = HashSet::new();

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
                    prime_implicants.insert(first.clone());
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
        let mut covering = self.prime_implicants();
        let mut covered_minterms = HashSet::<usize>::with_capacity(self.minterms());
        let mut implicants = Vec::new();

        while covered_minterms.len() < self.minterms() {
            let implicant = covering
                .iter()
                .max_by_key(|prime| {
                    prime
                        .constituents
                        .difference(&covered_minterms)
                        .filter(|&&constituent| self.is_minterm(constituent))
                        .count()
                })
                .unwrap()
                .clone();
            covering.remove(&implicant);

            covered_minterms.extend(
                implicant
                    .constituents
                    .iter()
                    .filter(|&&constituent| self.is_minterm(constituent)),
            );

            implicants.push(implicant);
        }

        implicants
    }
}
