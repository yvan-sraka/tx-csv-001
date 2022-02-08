//! # Rust Coding Test
//!
//! Thanks for taking the time to try out our coding challenge. Since we don't do a lot of
//! whiteboarding during the interview process we've found take home problems to be the best way
//! for you to show us your skills. This gives you the chance to think about the problem, solve it
//! when you feel comfortable, and focus on the areas you think are important. This challenge
//! should take you 2-3 hours on the keyboard, but we encourage you to read the problem
//! description and take some time to think about your solution before you dive into it :)
//!
//! ## Overview
//!
//! At XXXXXX we deal with transactions on and off chain. Off chain transactions are generally
//! mutable, and in a lot of ways this a good thing. It lowers the risk of losing your credit card
//! because you know your credit provider has your back, but for vendors integrating into payment
//! networks or with payment partners it means we need to be vigilant and on the lookout for
//! fraudsters. For example, a malicious actor may try to deposit fiat funds, purchase and withdraw
//! BTC, and then reverse their fiat deposit.
//!
//! We'd like you to implement a simple toy payments engine that reads a series of transactions
//! from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the
//! state of clients accounts as a CSV.
//!
//! ## Scoring
//!
//! You will be scored on the following criteria:
//!
//! - **Basics:**
//!
//!     Does your application build? Does it read and write data in the way we'd
//!     like it to? Is it properly formatted?
//!
//! - **Completeness:**
//!
//!     Do you handle all of the cases, including disputes, resolutions, and
//!     chargebacks? Maybe you don't handle disputes and resolutions but you
//!     can tell when a transaction is charged back. Try to cover as much as you
//!     can.
//!
//! - **Correctness:**
//!
//!     For the cases you are handling are you handling them correctly? How do
//!     you know this? Did you test against sample data? If so, include it in the
//!     repo. Did you write unit tests for the complicated bits? Or are you using the
//!     type system to ensure correctness? Tell us about it in the README.
//!
//! - **Safety and Robustness:**
//!
//!     Are you doing something dangerous? Tell us why you chose to do it this
//!     way. How are you handling errors?
//!
//! - **Efficiency:**
//!
//!     Be thoughtful about how you use system resources. Sample data sets may
//!     be small but you may be evaluated against much larger data sets (hint:
//!     transaction IDs are valid u32 values). Can you stream values through
//!     memory as opposed to loading the entire data set upfront? What if your
//!     code was bundled in a server, and these CSVs came from thousands of
//!     concurrent TCP streams?
//!
//! - **Maintainabilit:**
//!
//!     In this case clean code is more important than efficient code because
//!     humans will have to read and review your code without an opportunity for
//!     you to explain it. Inefficient code can often be improved if it is correct and
//!     highly maintainable.
//!
//! Your solution will be scored using a combination of automated and manual scoring. Automated
//! scoring will be used to run your solution against a handful of sample inputs, comparing the
//! resulting output. If our automatic test suite passes, manual scoring will be used for everything
//! else. At XXXXXX we care about clean, correct, safe, and efficient code.
//!
//! Because we use some automated scoring, **it is very important your program exposes the
//! CLI interface described below**. Anything else will be ignored by reviewers. If you think there
//! are inconsistencies in the specification, the described input & output files take precedence.
//!
//! ## Details
//!
//! Given a CSV representing a series of transactions, implement a simple toy transactions engine
//! that processes the payments crediting and debiting accounts. After processing the complete set
//! of payments output the client account balances
//!
//! You should be able to run your payments engine like:
//!
//! ```shell
//! $ cargo run -- transactions.csv > accounts.csv
//! ```
//! The input file is the first and only argument to the binary. Output should be written to stdout.
//! ## Submission
//!
//! When you are ready to share your submission, please email us back and include a link to a
//! GitHub repository containing your solution. The repository should be a simple Rust crate
//! generated using cargo new (or cargo init), should be buildable via cargo build, and runnable via
//! cargo run. Your solution should not mention XXXXXX, XXXXXXX, or any associated XXXXXX products,
//! brands, web domains, etc. This test file or any derivative must not be committed.
//!
//! ## Assumptions
//!
//! You're safe to make the following assumptions:
//!
//! - The client has a single asset account. All transactions are to and from this single asset
//! account;
//! - There are multiple clients. Transactions reference clients. If a client doesn't exist create a
//! new record;
//! - Clients are represented by u16 integers. No names, addresses, or complex client profile
//! info;
//!
//! When in doubt on how to interpret a requirement, try to make assumptions that make sense for
//! a bank (think an ATM or more elaborate transaction processors), and document them.
//!
//! ## Useful Libraries!
//!
//! - Serde for serialization and deserialization
//! - csv for reading and writing CSVs
//! - Any other common crate that you deem secure.

use anyhow::Result; // handy construct on top of `Result<T, Box<dyn Error>>`
use serde::{Deserialize, Serialize};

// ### Input
//
// The input will be a CSV file with the columns type, client, tx, and amount. You can assume the
// type is a string, the client column is a valid u16 client ID, the tx is a valid u32 transaction
// ID, and the amount is a decimal value with a precision of up to four places past the decimal.
//
// For example:
//
// ```csv
// type,  client, tx, amount
// deposit,    1,  1,    1.0
// deposit,    2,  2,    2.0
// deposit,    1,  3,    2.0
// withdrawal, 1,  4,    1.5
// withdrawal, 2,  5,    3.0
// ```
//
// The client ID will be unique per client though are not guaranteed to be ordered. Transactions to
// the client account 2 could occur before transactions to the client account 1. Likewise,
// transaction IDs (tx) are globally unique, though are also not guaranteed to be ordered. You can
// assume the transactions occur chronologically in the file, so if transaction b appears after a
// in the input file then you can assume b occurred chronologically after a. Whitespaces and
// decimal precisions (up to four places past the decimal) must be accepted by your program.

/// Client IDs are stored on 16-bits unsigned integers
type ClientID = u16;
/// Transaction IDs are stored on 32-bits unsigned integers
type TxID = u32;

/// Using a tuple-struct for `Input`, since `type` is a reserved keyword and couldn't be used as a
/// field name...
#[derive(Debug, Deserialize)]
struct Input(
    /// Transaction type
    Tx,
    /// Client ID
    ClientID,
    /// Transaction ID
    TxID,
    /// Transactions of type Dispute, Resolve or Chargeback does not specify an Amount
    Option<Amount>,
);

// ### Output
//
// The output should be a list of client IDs (client), available amounts (available), held amounts
// (held), total amounts (total), and whether the account is locked (locked). Columns are defined
// as:
//
// - available:
//
//     The total funds that are available for trading, staking, withdrawal, etc. This
//     should be equal to the total - held amounts
//
// - held:
//
//     The total funds that are held for dispute. This should be equal to total -
//     available amounts
//
// - total:
//
//     The total funds that are available or held. This should be equal to available +
//     held
//
// - locked:
//
//     Whether the account is locked. An account is locked if a charge back occurs
//
//
// For example:
//
// ```csv
// client, available, held, total, locked
//      1,       1.5,  0.0,   1.5,  false
//      2,       2.0,  0.0,   2.0,  false
// ```
//
// Spacing and displaying decimals for round values do not matter. Row ordering also does not
// matter. The above output will be considered the exact same as the following:
//
// ```csv
// client,available,held,total,locked
// 2,2,0,2,false
// 1,1.5,0,1.5,false
// ```

/// A wrapper (using `newtype` construct) around `f64` primitive type
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
struct Amount(f64);

/// ### Precision
///
/// You can assume a precision of four places past the decimal and should output values with the
/// same level of precision.
impl std::fmt::Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.5}", self)
    }
}

/// Explicitly authorizing `+` binary operation on `Amount` (to allow further rounding behavior?)
impl std::ops::Add for Amount {
    type Output = Amount;

    fn add(self, rhs: Self) -> Self::Output {
        Amount(self.0 + rhs.0)
    }
}

/// Explicitly authorizing `-` binary operation on `Amount` (to allow further rounding behavior?)
impl std::ops::Sub for Amount {
    type Output = Amount;

    fn sub(self, rhs: Self) -> Self::Output {
        Amount(self.0 - rhs.0)
    }
}

/// ### Types of Transactions
#[derive(Debug, Deserialize)]
#[allow(non_camel_case_types)]
enum Tx {
    /// #### Deposit
    ///
    /// A deposit is a credit to the client's asset account, meaning it should increase the
    /// available and total funds of the client account.
    ///
    /// A deposit looks like:
    ///
    /// ```csv
    /// type,    client, tx, amount
    /// deposit,      1,  1,    1.0
    deposit,

    /// #### Withdrawal
    ///
    /// A withdraw is a debit to the client's asset account, meaning it should decrease the
    /// available and total funds of the client account.
    ///
    /// A withdrawal looks like:
    ///
    /// ```csv
    /// type,  client, tx, amount
    /// withdrawal, 2,  2,    1.0
    /// ```
    ///
    /// If a client does not have sufficient available funds the withdrawal should fail and the
    /// total amount of funds should not change.
    withdrawal,

    /// #### Dispute
    ///
    /// A dispute represents a client's claim that a transaction was erroneous and should be
    /// reversed. The transaction shouldn't be reversed yet but the associated funds should be held.
    /// This means that the clients available funds should decrease by the amount disputed, their
    /// held funds should increase by the amount disputed, while their total funds should remain the
    /// same.
    ///
    /// A dispute looks like:
    ///
    /// ```csv
    /// type, client, tx, amount
    /// dispute,   1,  1,
    /// ```
    ///
    /// Notice that a dispute does not state the amount disputed. Instead a dispute references the
    /// transaction that is disputed by ID. If the tx specified by the dispute doesn't exist you can
    /// ignore it and assume this is an error on our partners side.
    dispute,

    /// #### Resolve
    ///
    /// A resolve represents a resolution to a dispute, releasing the associated held funds. Funds
    /// that were previously disputed are no longer disputed. This means that the clients held funds
    /// should decrease by the amount no longer disputed, their available funds should increase by
    /// the amount no longer disputed, and their total funds should remain the same.
    ///
    /// A resolve looks like:
    ///
    /// ```csv
    /// type, client, tx, amount
    /// resolve,   1,  1,
    /// ```
    ///
    /// Like disputes, resolves do not specify an amount. Instead they refer to a transaction that
    /// was  under dispute by ID. If the tx specified doesn't exist, or the tx isn't under dispute,
    /// you can ignore the resolve and assume this is an error on our partner's side.
    resolve,

    /// #### Chargeback
    ///
    /// A chargeback is the final state of a dispute and represents the client reversing a
    /// transaction. Funds that were held have now been withdrawn. This means that the clients held
    /// funds and total funds should decrease by the amount previously disputed. If a chargeback
    /// occurs the client's account should be immediately frozen.
    ///
    /// A chargeback looks like:
    ///
    /// ```csv
    /// type,  client, tx, amount
    /// chargeback, 1,  1,
    /// ```
    ///
    /// Like a dispute and a resolve a chargeback refers to the transaction by ID (tx) and does not
    /// specify an amount. Like a resolve, if the tx specified doesn't exist, or the tx isn't under
    /// dispute, you can ignore chargeback and assume this is an error on our partner's side.
    chargeback,
}

/// Here is a simple dumb algorithm that loop over the input values, mutating a collection of
/// ledgers. This stateful approach is required and forbid us for doing a lot of naive optimization,
/// e.g. using rayon parallel iterator, since transaction shouldn't be evaluated out of order...
#[derive(Debug)]
struct Ledger {
    available: Amount,
    held: Amount,
    status: LedgerStatus,
}

/// A ledger couldn't be not both locked and under dispute
#[derive(Debug, PartialEq)]
enum LedgerStatus {
    Default,
    Disputed,
    Locked,
}

/// By default every client get an empty of fund unlocked account
impl Default for Ledger {
    fn default() -> Self {
        Ledger {
            available: Amount(0.0),
            held: Amount(0.0),
            status: LedgerStatus::Default,
        }
    }
}

#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    /// Global history of all transactions (designed to be shared between several threads)
    static ref HISTORY: Mutex<HashMap<TxID, Amount>> = Mutex::new(HashMap::new());
}

/// Simple macro to insert a new transaction in global history
macro_rules! history_insert {
    ($tx_id: expr, $amount: expr) => {
        HISTORY.lock().unwrap().insert($tx_id, $amount);
    };
}

/// Neat trick to make history reading won't fail in non-strict mode if transaction not found
macro_rules! history_get {
    ($tx_id: expr) => {
        match HISTORY.lock().unwrap().get($tx_id) {
            Some(x) => *x,
            None => {
                #[cfg(feature = "strict_mode")]
                panic!("transaction ID {} not found", $tx_id);
                continue;
            }
        }
    };
}

/// I choose to design my code under few principles:
///
/// - Literate programming (bunch of comments surrounding my code) in the way of the Rust `std` was
///   written to be the most self-explanatory possible (please `cargo doc --no-deps` all this!)
///
/// - Binary-oriented approach, since the code will be tested as binary, I write my unit-test using
///   this approach, one `main` business logic enabling or disabling feature using cargo, rather
///   than split in functions and types and types exposed in the way library crate would be designed
///
/// - By default the program will fail silently on erroneous transactions, but with
///   `--feature strict_mode` it will panic if such invalid operation occurs, an improvement would
///   be to have an `export LOG_LEVEL=verbose` mode (using e.g. `log` crate) to warn user without
///   stopping the program on a non-recovered error!
fn main() -> Result<()> {
    // This `accounts` data-structure could be in the future an abstraction around a cold-storage
    // database (using e.g. CBOR or SLED)
    let mut accounts: HashMap<ClientID, Ledger> = HashMap::new();
    // The following code is heavily inspired by CSV crate usage example
    // from https://docs.rs/csv/latest/csv/#example-with-serde
    let mut rdr = csv::ReaderBuilder::new()
        // Because it's not explicitly specified of we should handle the absence of amount field...
        // https://docs.rs/csv/latest/csv/struct.ReaderBuilder.html#method.flexible
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(std::io::stdin());
    // `continue` block mixed with macro invocation currently mislead `rust-analyser` to false
    // positive on `unreachable blocks` lint...
    #[allow(unreachable_code)]
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic deserialization.
        let tx: Input = result?;
        let ledger = accounts.entry(tx.1).or_insert_with(Ledger::default);
        if ledger.status == LedgerStatus::Locked {
            #[cfg(feature = "strict_mode")]
            panic!("account locked, transaction are forbidden");
            continue;
        }
        match tx.0 {
            // Store deposit or withdrawal transaction amount to history
            Tx::deposit => {
                let amount = tx.3.expect("missing amount in deposit transaction");
                ledger.available = ledger.available + amount;
                history_insert!(tx.2, amount);
            }
            Tx::withdrawal => {
                let amount = tx.3.expect("missing amount in withdrawal transaction");
                if amount <= ledger.available {
                    ledger.available = ledger.available - amount;
                    history_insert!(tx.2, amount);
                } else {
                    #[cfg(feature = "strict_mode")]
                    panic!("client {} can't withdraw (not enough money)", tx.1);
                }
            }
            // Retrieve deposit or withdrawal transaction amount from history
            Tx::dispute => {
                let amount = history_get!(&tx.2);
                ledger.status = LedgerStatus::Disputed;
                ledger.available = ledger.available - amount;
                ledger.held = ledger.held + amount;
            }
            Tx::resolve => {
                if ledger.status == LedgerStatus::Disputed {
                    let amount = history_get!(&tx.2);
                    ledger.status = LedgerStatus::Default;
                    ledger.held = ledger.held - amount;
                    ledger.available = ledger.available + amount;
                } else {
                    #[cfg(feature = "strict_mode")]
                    panic!("transaction {} should be disputed to be resolved", tx.2);
                }
            }
            Tx::chargeback => {
                if ledger.status == LedgerStatus::Disputed {
                    let amount = history_get!(&tx.2);
                    ledger.status = LedgerStatus::Locked;
                    ledger.held = ledger.held - amount;
                } else {
                    #[cfg(feature = "strict_mode")]
                    panic!("transaction {} should be disputed to be chargeback", tx.2);
                }
            }
        }
    }
    // From https://docs.rs/csv/latest/csv/tutorial/index.html#writing-with-serde
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    // We still need to write headers manually.
    wtr.write_record(&["client", "available", "held", "total", "locked"])?;
    #[cfg(feature = "sorted")]
    let accounts = {
        let mut v = accounts.iter().collect::<Vec<(&ClientID, &Ledger)>>();
        v.sort_by(|a, b| a.0.cmp(b.0));
        v
    };
    // But now we can write records by providing a normal Rust value.
    for (client_id, ledger) in accounts {
        wtr.serialize((
            client_id,
            ledger.available,
            ledger.held,
            ledger.available + ledger.held,
            ledger.status == LedgerStatus::Locked,
        ))?;
    }
    wtr.flush()?;
    Ok(())
}

// Unordered list of improvement ideas:
//
// - using `criterion` for statistically accurate benchmarking over using other data structure than
//   the standard `HashMap`, for e.g. a pre-allocated `Vec` could give better result if Client ID
//   space is continuous and small
//
// - check the correctness of the program using fuzzing with `Arbitrary` crate
//
// - write more tests, for e.g. with `#[should_fail]` decorator in `strict_mode`
#[cfg(test)]
use assert_cmd::Command;
#[test]
fn example() {
    // `transactions.csv`
    const INPUT: &str = r#"type,  client, tx, amount
deposit,    1,  1,    1.0
deposit,    2,  2,    2.0
deposit,    1,  3,    2.0
withdrawal, 1,  4,    1.5
withdrawal, 2,  5,    3.0
"#;
    // `accounts.csv`
    const OUTPUT: &str = r#"client,available,held,total,locked
1,1.5,0.0,1.5,false
2,2.0,0.0,2.0,false
"#;
    // From https://docs.rs/assert_cmd/latest/assert_cmd/#examples
    let assert = Command::new("cargo")
        .args(["run", "--features", "sorted"])
        .write_stdin(INPUT)
        .assert();
    // Improvement: have a test that is robust to CSV formatting (currently I'm cheating requiring
    // the `sorted` feature in test mode)
    assert.success().stdout(OUTPUT);
    // let stdout = String::from_utf8_lossy(&assert.success().get_output().clone().stdout);
    // let result = csv::Reader::from_reader(stdout).records();
    // let expected = csv::Reader::from_reader(OUTPUT).records();
    // TODO: turn `result` and `expected` to `Vec`, sort and compare them!
}

// Thanks for reading me along the way 🦀! /Yvan <yvan@sraka.xyz>
