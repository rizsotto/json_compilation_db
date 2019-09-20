/// This crate provides support for reading and writing JSON compilation database files.
///
/// A compilation database is a set of records which describe the compilation of the
/// source files in a given project. It describes the compiler invocation command to
/// compile a source module to an object file.
///
/// This database can have many forms. One well known and supported format is the JSON
/// compilation database, which is a simple JSON file having the list of compilation
/// as an array. The definition of the JSON compilation database files is done in the
/// LLVM project [documentation](https://clang.llvm.org/docs/JSONCompilationDatabase.html).

pub mod api;

pub use super::error::{Error, Result};
pub use super::api::*;

// TODO: define error type and result
mod error {

    type Error = &'static str;

    type Result<T> = std::result::Result<T, Error>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
