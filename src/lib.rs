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

mod inner;

pub use error::*;
pub use api::*;

mod error {
    use std::fmt;
    use std::error;
    use std::io;

    #[derive(Debug)]
    pub enum Error {
        IoError(io::Error),
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

    impl From<io::Error> for Error {
        fn from(cause: io::Error) -> Self {
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

mod api {
    use super::error::*;
    use super::inner;

    use std::path;
    use std::io;
    use std::fs;

    /// Represents an entry of the compilation database.
    #[derive(Debug, PartialEq)]
    pub struct Entry {
        pub file: path::PathBuf,
        pub command: Vec<String>,
        pub directory: path::PathBuf,
        pub output: Option<path::PathBuf>,
    }

    /// Represents the content of the compilation database.
    pub type Entries = Vec<Entry>;

    /// Represents the expected format of the JSON compilation database.
    #[derive(Debug, PartialEq, Eq)]
    pub struct Format {
        pub command_as_array: bool,
    }

    impl Default for Format {
        fn default() -> Self {
            Format {
                command_as_array: true,
            }
        }
    }

    pub fn load_from_file(file: &path::Path) -> Result<Entries> {
        let reader = fs::OpenOptions::new()
            .read(true)
            .open(file)?;

        load_from_reader(reader)
    }

    pub fn load_from_reader(reader: impl io::Read) -> Result<Entries> {
        inner::load_from_reader(reader)
    }

    pub fn save_into_file(file: &path::Path, entries: Entries, format: &Format) -> Result<()> {
        let writer = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file)?;

        save_into_writer(writer, entries, format)
    }

    pub fn save_into_writer(writer: impl io::Write, entries: Entries, format: &Format) -> Result<()> {
        inner::save_into_writer(writer, entries, format)
    }
}
