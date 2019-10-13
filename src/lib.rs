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

pub use error::{Error, Result};
pub use api::*;

mod error {

    use std::fmt;
    use std::error;

    #[derive(Debug)]
    pub enum Error {
        IoError(std::io::Error),
        SyntaxError(serde_json::Error),
        SemanticError(String),
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                Error::IoError(_) =>
                    write!(f, "IO problem."),
                Error::SyntaxError(_) =>
                    write!(f, "Syntax problem."),
                Error::SemanticError(ref message) =>
                    write!(f, "Semantic problem: {}", message),
            }
        }
    }

    impl error::Error for Error {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match *self {
                Error::IoError(ref cause) => Some(cause),
                Error::SyntaxError(ref cause) => Some(cause),
                Error::SemanticError(_) => None,
            }
        }
    }

    impl From<std::io::Error> for Error {
        fn from(cause: std::io::Error) -> Self {
            Error::IoError(cause)
        }
    }

    impl From<serde_json::Error> for Error {
        fn from(cause: serde_json::Error) -> Self {
            Error::SyntaxError(cause)
        }
    }

    impl From<String> for Error {
        fn from(message: String) -> Self {
            Error::SemanticError(message)
        }
    }

    pub type Result<T> = std::result::Result<T, Error>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
