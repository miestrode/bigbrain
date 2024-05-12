# `bigbrain` logic circuit optimizer

Given a set of logic functions, in the form of tables, `bigbrain` is a program for creating an optimized circuit fitting the tables. It uses HiGHS for ILP, and so requires having a C++ compiler and `cmake`, to build it.

## How does it work?

Here's a rough views of the steps involved.

- Find prime implicants for each table using the Quine-McCluskey algorithm
- Find a covering of the minterms using ILP (Currently using the HiGHS solver)
- Add each covering to an e-graph engine
- Perform equality saturation using a set of rewrite rules (using `ekege`)
- Extract each function output from the e-graph. Different metrics may be used:
  - Latency (maximum depth)
  - Total number of gates
  - Gate restrictions (only NAND, only AND, OR and NOT, etc.)
  - Anything else

The result is an optimized circuit which takes into account don't care states, and shares logic between the functions.

The result naturally isn't optimal, and decisions about don't care optimization are localized to each function. This keeps a good mix of performance and quality. 
