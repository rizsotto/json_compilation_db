# JSON Compilation Database

This crate provides support for reading and writing JSON compilation database files.

## Overview

A _compilation database_ is a set of records which describe the compilation of the
source files in a given project. It describes the compiler invocation command to
compile a single source file to an object file.

This database can have many forms. One well known and supported format is the JSON
compilation database, which is a simple JSON file having the list of compilation
as an array. The definition of the JSON compilation database files is done in the
LLVM project [documentation](https://clang.llvm.org/docs/JSONCompilationDatabase.html).

## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
json_compilation_db = "1.0"
```

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `json_compilation_db` by you, shall be licensed as MIT, without
any additional terms or conditions.
