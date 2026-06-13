use sql_test::SqlTestRunner;
use std::path::PathBuf;

#[test]
fn test_sqlness() {
    let mut base_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| "crates/sql_test".to_string()));
    base_dir.push("testcases");
    let runner = SqlTestRunner::new(base_dir);
    runner.run_all();
}
