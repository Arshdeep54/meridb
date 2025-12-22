# MeriDB

## A Database from Scratch in Rust

MeriDB is a database built from the ground up in Rust as a learning project to understand database internals. It features a custom-built lexer, parser, storage engine, and a terminal-style query interface. The goal is to explore how databases work under the hood by implementing core functionalities from scratch.

---

## Key Features (Implemented)

- Custom lexer and parser generating a strongly typed AST (SNAFU-based errors)
- Terminal-like input handling with history and line editing
- File-backed catalog with binary, versioned metadata (CRC32-checked)
- Table schemas persisted as binary `schema.tbl` per table
- Early page/record layout with fixed-size heap pages (8 KiB) and slot directory
- Modular multi-crate workspace for clean layering

---

## Planned Enhancements

- B-Tree indexing
- Server mode and network protocol
- WAL/transactions and recovery
- Broader SQL support and planner improvements

---

## Workspace Structure

This is a multi-crate cargo workspace under `crates/`:

- `crates/types` — Core token and datatype definitions shared across crates
- `crates/sql` — Lexer, tokens, AST, parser, and structured parser errors (SNAFU)
- `crates/exec` — Executor translating AST into catalog/storage operations
- `crates/catalog` — Catalog trait and file-backed implementation with binary metadata (`metadata.mdb`) and table schemas (`schema.tbl`)
- `crates/storage` — In-memory tables, records, and fixed-size page format (8 KiB) with slot directory; helpers to serialize records/pages
- `crates/api` — Session façade wiring a `Catalog` and `Executor` for clients
- `crates/cli` — Terminal client with history; uses the API session

---

## Installation & Usage

### Prerequisites

- Rust toolchain

### Build & Run

```sh
# Clone the repository
git clone https://github.com/arshdeep54/meridb.git
cd meridb

# Build the project
cargo build

# Run MeriDB
cargo run
```

### Basic CLI Workflow

```sql
-- Create a database (creates data/<db>/ with binary metadata.mdb)
create database mydb;

-- Switch to a database (validates metadata)
use mydb;

-- Create a table (creates data/<db>/tables/<table>/schema.tbl)
create table users(id integer, name text);

-- List databases and tables
show databases;
show tables;

-- Insert (persists in-memory pages; first segment file under data/<db>/tables/<table>/data/)
insert into users values (1, 'Alice');
```

Notes:
- Database metadata and table schemas are binary, versioned, and checksummed (CRC32).
- Table data uses fixed-size heap pages (8 KiB) with a slot directory; persistence is evolving.
- Parser and executor return typed errors; CLI prints user-friendly messages.

---

## Why I Built This?

I wanted to dive deep into database internals and understand how databases parse, store, and execute queries from scratch. This project is a learning exercise in Rust’s memory safety, performance optimizations, and system-level programming capabilities.

---

## 📜 License

MeriDB is released under the MIT License. Feel free to use and modify it!

---

