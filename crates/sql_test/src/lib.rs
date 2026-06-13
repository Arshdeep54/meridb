use api::Session;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SqlTestRunner {
    testcases_dir: PathBuf,
}

impl SqlTestRunner {
    pub fn new<P: AsRef<Path>>(testcases_dir: P) -> Self {
        Self {
            testcases_dir: testcases_dir.as_ref().to_path_buf(),
        }
    }

    pub fn run_all(&self) {
        let entries = fs::read_dir(&self.testcases_dir).expect("Failed to read testcases directory");
        let mut sql_files = Vec::new();

        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "sql") {
                sql_files.push(path);
            }
        }

        sql_files.sort();

        let mut failed = false;
        let mut failures = Vec::new();

        for sql_file in sql_files {
            let file_name = sql_file.file_name().unwrap().to_str().unwrap();
            let test_name = sql_file.file_stem().unwrap().to_str().unwrap();
            
            let temp_dir = PathBuf::from("target/sql_test_data").join(test_name);
            if temp_dir.exists() {
                let _ = fs::remove_dir_all(&temp_dir);
            }
            fs::create_dir_all(&temp_dir).unwrap();

            let result = self.run_test_file(&sql_file, &temp_dir);
            let _ = fs::remove_dir_all(&temp_dir);

            match result {
                Ok(_) => println!("Test {} ... ok", file_name),
                Err(err) => {
                    println!("Test {} ... FAILED\n{}", file_name, err);
                    failed = true;
                    failures.push((file_name.to_string(), err));
                }
            }
        }

        if failed {
            panic!("SQL integration tests failed: {:?}", failures);
        }
    }

    fn run_test_file(&self, sql_file: &Path, temp_dir: &Path) -> Result<(), String> {
        let content = fs::read_to_string(sql_file)
            .map_err(|e| format!("Failed to read SQL file: {}", e))?;

        let statements = parse_statements(&content);
        let mut actual_output = String::new();
        let mut session = Session::file_backed(temp_dir.to_path_buf());

        for statement in statements {
            let ast = match sql::parse_command(&statement) {
                Ok(ast) => ast,
                Err(err) => {
                    actual_output.push_str(&format!("-- query:\n{}\n-- error:\nParse error: {}\n\n", statement, err));
                    continue;
                }
            };

            match session.execute(ast) {
                Ok(result) => {
                    actual_output.push_str(&format!(
                        "-- query:\n{}\n-- result:\n{}\n\n",
                        statement,
                        format!("{}", result).trim()
                    ));
                }
                Err(err) => {
                    actual_output.push_str(&format!(
                        "-- query:\n{}\n-- error:\n{}\n\n",
                        statement,
                        err.trim()
                    ));
                }
            }
        }

        let result_file = sql_file.with_extension("result");

        if std::env::var("UPDATE_EXPECT").is_ok() {
            fs::write(&result_file, &actual_output)
                .map_err(|e| format!("Failed to write result file: {}", e))?;
            return Ok(());
        }

        if !result_file.exists() {
            return Err(format!(
                "Result file does not exist: {}. Run with UPDATE_EXPECT=1 to generate it.",
                result_file.display()
            ));
        }

        let expected_output = fs::read_to_string(&result_file)
            .map_err(|e| format!("Failed to read result file: {}", e))?;

        if actual_output.trim() != expected_output.trim() {
            return Err(format!(
                "Mismatch in output for {}.\n\n=== Expected ===\n{}\n=== Actual ===\n{}",
                sql_file.display(),
                expected_output,
                actual_output
            ));
        }

        Ok(())
    }
}

fn parse_statements(content: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;

    let cleaned_lines: Vec<&str> = content
        .lines()
        .filter(|line| !line.trim().starts_with("--"))
        .collect();
    let cleaned_content = cleaned_lines.join("\n");

    for ch in cleaned_content.chars() {
        match ch {
            '\'' => {
                in_single_quote = !in_single_quote;
                current.push(ch);
            }
            ';' if !in_single_quote => {
                current.push(ch);
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    statements.push(trimmed.to_string());
                }
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        statements.push(trimmed.to_string());
    }

    statements
}
