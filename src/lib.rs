/*!
This crate provides support for reading and writing JSON compilation database files.

A compilation database is a set of records which describe the compilation of the
source files in a given project. It describes the compiler invocation command to
compile a source module to an object file.

This database can have many forms. One well known and supported format is the JSON
compilation database, which is a simple JSON file having the list of compilation
as an array. The definition of the JSON compilation database files is done in the
LLVM project [documentation](https://clang.llvm.org/docs/JSONCompilationDatabase.html).
*/

mod inner;

pub use api::*;
pub use error::*;

mod error {
    use std::error;
    use std::fmt;
    use std::io;

    /// This error type encompasses any error that can be returned by this crate.
    #[derive(Debug)]
    pub enum Error {
        /// Represents basic IO failure.
        IoError(io::Error),
        /// Represents JSON read or write failure.
        SyntaxError(serde_json::Error),
        /// Represents semantic failure during procession of the file content.
        SemanticError(String),
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                Error::IoError(_) => write!(f, "IO problem."),
                Error::SyntaxError(_) => write!(f, "Syntax problem."),
                Error::SemanticError(ref message) => write!(f, "Semantic problem: {}", message),
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
}

mod api {
    use super::error::*;
    use super::inner;

    use std::fs;
    use std::io;
    use std::path;

    /// Represents an entry of the compilation database.
    #[derive(Debug, PartialEq)]
    pub struct Entry {
        /// The main translation unit source processed by this compilation step.
        /// This is used by tools as the key into the compilation database.
        /// There can be multiple command objects for the same file, for example if the same
        /// source file is compiled with different configurations.
        pub file: path::PathBuf,
        /// The compile command executed. After JSON unescaping, this must be a valid command
        /// to rerun the exact compilation step for the translation unit in the environment
        /// the build system uses. Shell expansion is not supported.
        pub command: Vec<String>,
        /// The working directory of the compilation. All paths specified in the command or
        /// file fields must be either absolute or relative to this directory.
        pub directory: path::PathBuf,
        /// The name of the output created by this compilation step. This field is optional.
        /// It can be used to distinguish different processing modes of the same input file.
        pub output: Option<path::PathBuf>,
    }

    /// Represents the content of the compilation database.
    ///
    /// A compilation database is a JSON file, which consist of an array of “command objects”,
    /// where each command object specifies one way a translation unit is compiled in the project.
    pub type Entries = Vec<Entry>;

    /// Represents the expected format of the JSON compilation database.
    #[derive(Debug, PartialEq, Eq)]
    pub struct Format {
        /// Controls which field to emit in the final database.
        ///
        /// In the output the field `command` is a string and `arguments` is an array of
        /// strings. Either `command` or `arguments` is required.
        pub command_as_array: bool,
    }

    impl Default for Format {
        fn default() -> Self {
            Format {
                command_as_array: true,
            }
        }
    }

    /// The conventional name for a compilation database file which tools are looking for.
    pub const DEFAULT_FILE_NAME: &str = "compile_commands.json";

    /// Load the content of the given file and parse it as a compilation database.
    pub fn load_from_file(file: &path::Path) -> Result<Entries, Error> {
        let reader = fs::OpenOptions::new().read(true).open(file)?;

        load_from_reader(reader)
    }

    /// Load the content of the given stream and parse it as a compilation database.
    pub fn load_from_reader(reader: impl io::Read) -> Result<Entries, Error> {
        inner::load_from_reader(reader)
    }

    /// Persists the entries into the given file name with the given format.
    pub fn save_into_file(
        file: &path::Path,
        entries: Entries,
        format: &Format,
    ) -> Result<(), Error> {
        let writer = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file)?;

        save_into_writer(writer, entries, format)
    }

    /// Persists the entries into the given stream with the given format.
    pub fn save_into_writer(
        writer: impl io::Write,
        entries: Entries,
        format: &Format,
    ) -> Result<(), Error> {
        inner::save_into_writer(writer, entries, format)
    }
}
