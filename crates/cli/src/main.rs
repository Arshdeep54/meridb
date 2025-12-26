use std::fs;
use std::path::PathBuf;

use api::Session;
use clap::Parser;
use sql::ast::ASTNode;
use sql::parse_command;
use tracing::{debug, info};

use crate::input_handler::InputHandler;

pub mod input_handler;
mod logging;

#[derive(Debug, Parser)]
#[command(name = "meridb", version, about = "MeriDB CLI")]
struct Args {
    /// Data root directory (where databases live)
    #[arg(long = "data-dir", value_name = "PATH", default_value = "data")]
    data_dir: PathBuf,

    /// Database to select before executing (equivalent to: USE <db>)
    #[arg(short = 'd', long = "database", value_name = "DB")]
    database: Option<String>,

    /// Execute a single SQL statement non-interactively and exit
    #[arg(short = 'e', long = "exec", value_name = "SQL")]
    exec: Option<String>,
}

fn main() {
    let args = Args::parse();

    let _guard = logging::init_logging(&args.data_dir, args.database.as_deref());

    fs::create_dir_all(&args.data_dir).ok();

    // Non-interactive: --exec
    if let Some(sql) = args.exec {
        let mut session = Session::file_backed(args.data_dir);

        if let Some(db) = args.database {
            info!("Using database: {}", db);
            if let Err(e) = session.execute(ASTNode::USE {
                database_name: db.clone(),
            }) {
                eprintln!("Exec error: {e}");
                std::process::exit(1);
            }
        }

        match parse_command(&sql) {
            Ok(ast) => match session.execute(ast) {
                Ok(qr) => {
                    println!("{}", qr);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Exec error: {e}");
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Parse error: {}", e);
                std::process::exit(2);
            }
        }
    }

    let history_file = args.data_dir.join("history.txt");
    let mut input_handler =
        InputHandler::with_history_file(history_file).expect("Failed to initialize input handler");

    let mut session = Session::file_backed(args.data_dir);

    //preselect database for the REPL if -d/--database is provided
    if let Some(db) = args.database {
        info!("Using database: {}", db);
        if let Err(e) = session.execute(ASTNode::USE { database_name: db }) {
            eprintln!("Exec error: {e}");
        }
    }

    while let Ok(line) = input_handler.readline("meridb> ") {
        if line.eq_ignore_ascii_case("exit") {
            break;
        }
        match parse_command(&line) {
            Ok(ast) => {
                debug!("Parsed AST: {:#?}", ast);
                match session.execute(ast) {
                    Ok(qr) => println!("{}", qr),
                    Err(e) => eprintln!("Exec error: {e}"),
                }
            }
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
}
