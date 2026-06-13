# SQL Test Harness Feature Documentation

This document explains the purpose, architecture, and operation of the SQL Integration Test Harness in MeriDB.

---

## What It Is
The SQL Test Harness is an end-to-end integration testing framework designed to verify query correctness, storage engine serialization, and database catalog states. Instead of embedding SQL test scripts as strings inside Rust test code, it uses a file-driven "golden master" approach.

---

## How It Works
The test runner is implemented as a standalone crate in the Cargo workspace (`crates/sql_test`).

1.  **Discovery**:
    *   The runner scans the test directory for all files with the `.sql` extension.
2.  **Isolation**:
    *   For each `.sql` file discovered, the runner generates an isolated temporary folder under the compilation target directory.
    *   This folder acts as the file-backed storage root for that specific test run, ensuring no state leakage between tests.
3.  **Statement Parsing**:
    *   The runner reads the SQL file and splits its contents into individual statements. It carefully handles whitespace and ensures that semicolons are preserved so that the database's parser can cleanly process them.
4.  **Sequential Execution**:
    *   For each statement, the runner sends the query to the query executor, tracking the formatted output (whether a success table, a count of affected rows, or a database error).
5.  **Output Aggregation**:
    *   Both the query and its formatted output/error are appended to a running string buffer.
6.  **Assertion and Golden Master Validation**:
    *   The runner reads the corresponding `.result` file (which contains the expected outputs).
    *   It asserts that the newly captured output buffer matches the expected content exactly.
    *   If a mismatch occurs, the test runner fails and prints a detailed diff of the expected vs. actual query results.

---

## How to Run the SQL Integration Tests
To run the SQL integration tests, execute the standard workspace test command:

```bash
cargo test -p sql_test
```

---

## How to Generate or Auto-Update Expected Results
When you write a new test case (`.sql` file) or intentionally change the output formatting or functionality of the database, you can automatically generate or update the `.result` files using the `UPDATE_EXPECT` environment variable:

```bash
UPDATE_EXPECT=1 cargo test -p sql_test
```

This updates all `.result` files with the actual output produced by the execution engine, ready to be reviewed and committed.
