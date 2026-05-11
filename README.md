# kframework-rs

Rust bindings for the [K Framework](https://github.com/runtimeverification/k): a KORE pattern parser/serializer and FFI wrappers for the KLLVM execution backend.


## Crates

### `kframework`

Parses and manipulates KORE (K's intermediate representation) in pure Rust, with no native dependencies.

- Lexer and parser for textual KORE patterns, sorts, sentences, and definitions
- Full AST for KORE (`Pattern`, `Sort`, `App`, `Sentence`, `Definition`, ...)
- JSON serialization and deserialization via `serde`
- `PatternVisitor` trait for custom traversals
- `kore-to-json` binary: reads a `.kore` file and prints JSON to stdout


### `kframework_ffi`

Rust wrappers around the KLLVM C API (`interpreter.so`). Lets you execute K semantics compiled with `kompile --llvm-kompile-type c` from Rust.

- `Pattern`: owns a `kore_pattern *`; parse from a KORE string or build from a `Symbol`
- `Block`: wraps KLLVM's interned term representation; runs execution via `take_steps`
- `Sort` / `Symbol`: RAII wrappers with `Drop` implementations
- `Marshaller<H>`: converts a `kore::Pattern` tree to kllvm, substituting typed variables through a user-supplied `VarHandler`; caches variable-free subtrees across calls


## Getting Started

### Prerequisites

- Rust 1.85.0+
- `libclang` (`kframework_ffi` only)
- A `kompile`-built `interpreter.so` on `LD_LIBRARY_PATH` (`kframework_ffi` only)


### Build

```sh
# Build the pure-Rust kframework crate only
cargo build -p kframework

# Build everything (requires interpreter.so)
cargo build
```


### Test

```sh
cargo test
```


### Format & Lint

```sh
cargo fmt
cargo clippy --all-targets -- --deny warnings
```

Or run all checks at once:

```sh
make
```


## Usage

### Parsing a KORE Pattern

```rust
use kframework::kore::Parser;

let mut parser = Parser::new(r#"\dv{SortInt{}}("42")"#)?;
let pattern = parser.pattern()?;
println!("{:?}", pattern);
```


### Converting a `.kore` File to JSON

```sh
cargo run --bin kore-to-json -- path/to/file.kore
```


### Executing K Semantics via KLLVM

```rust
use kframework_ffi::kllvm;

kllvm::init();

let pattern: kllvm::Pattern = KORE_STRING.parse().expect("parse failed");
let mut block: kllvm::Block = pattern.into();
block.take_steps(-1); // run to completion

let result: kllvm::Pattern = block.into();
println!("{result}");

kllvm::free_all_memory();
```

See [`kframework_ffi/src/kllvm.rs`](kframework_ffi/src/kllvm.rs) for the full API.


## Examples

### Fuzzer (`examples/fuzzer`)

Uses `kframework_ffi` with [honggfuzz](https://github.com/rust-fuzz/honggfuzz-rs) to fuzz K semantics compiled from the bundled Imp language definition.

See [`examples/fuzzer/README.md`](examples/fuzzer/README.md) for setup instructions.


## License

BSD 3-Clause. Copyright (c) 2024-2026 Runtime Verification Inc. See [LICENSE](LICENSE) for details.
