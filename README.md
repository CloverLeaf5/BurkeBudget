# BurkeBudget

A command line budgeting app written in Rust with a rusqlite DB created to practice with the Rust language.

It utilizes rusty-money to format money Strings and chrono to format dates.
It also utilizes textplots and was the source of a pull request to display tick labels on the y-axis.

![budget screenshots](https://raw.githubusercontent.com/CloverLeaf5/BurkeBudget/main/.github/budget.JPG)

## Run the application

```
git clone https://github.com/CloverLeaf5/BurkeBudget
cd BurkeBudget
cargo build --release
cd target/release
./burkebudget
```

Note that the burkebudgetDB.db file should be kept in the same directory as the executable
