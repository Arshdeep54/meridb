# MeriDB

## 🚀 A Database from Scratch in Rust

MeriDB is a database built from the ground up in Rust as a learning project to understand database internals. It features a custom-built lexer, parser, storage engine, and a terminal-style query interface. The goal is to explore how databases work under the hood by implementing core functionalities from scratch.

---

## 🔥 Key Features (Implemented)

✅ **Custom Lexer & Parser** - Converts raw input into structured queries with case-insensitive parsing.\
✅ **Terminal-Like Input Handling** - Supports command history with up/down arrow keys, cursor movements, and real-time editing.\
✅ **Storage Engine** - Implements pages and records for efficient data storage.\
✅ **Query Execution Engine (WIP)** - Currently being developed to process and execute queries.\
✅ **Modular Codebase** - Well-structured with separate modules for parsing, execution, input handling, and storage.

---

## 🎯 Future Enhancements

🔹 **B-Tree Indexing** - Implementing indexing for faster data retrieval.\
🔹 **Server Mode** - Enabling remote connections for client-server architecture.\
🔹 **Dockerization** - Making MeriDB easier to deploy with Docker support.\
🔹 **Extended SQL Support** - Expanding query capabilities over time.

---

## 📂 Project Structure

```
meridb/
├── Cargo.toml      # Rust dependencies and metadata
├── src/
│   ├── bin/meridb.rs        # Main entry point
│   ├── database/            # Session management
│   ├── executor/            # Query execution logic
│   ├── input_handler/       # Terminal-like user input handling
│   ├── parser/              # Lexer, parser, AST representation
│   ├── storage/             # Pages, records, and tables
│   └── types/               # Custom data types
└── tests/                   # Unit tests for different modules
```

---

## 🛠️ Installation & Usage

### Prerequisites

- Rust (latest stable version)

### Build & Run

```sh
# Clone the repository
git clone https://github.com/arshdeep54/meridb.git
cd meridb

# Build the project
cargo build --release

# Run MeriDB
cargo run
```


## 👨‍💻 Why I Built This?

I wanted to dive deep into database internals and understand how databases parse, store, and execute queries from scratch. This project is a learning exercise in Rust’s memory safety, performance optimizations, and system-level programming capabilities.

If you're interested in databases, compilers, or low-level optimizations, feel free to check it out! 🚀

---

## 📜 License

MeriDB is released under the MIT License. Feel free to use and modify it!

---

