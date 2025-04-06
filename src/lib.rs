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

extern crate core;

use serde::ser::{SerializeSeq, Serializer};
use serde_json::Error;

mod iterator;
mod type_de;
mod type_ser;

/// The conventional name for a compilation database file which tools are looking for.
pub const DEFAULT_FILE_NAME: &str = "compile_commands.json";

/// Represents an entry of the compilation database.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The main translation unit source processed by this compilation step.
    /// This is used by tools as the key into the compilation database.
    /// There can be multiple command objects for the same file, for example if the same
    /// source file is compiled with different configurations.
    pub file: std::path::PathBuf,
    /// The compile command executed. This must be a valid command to rerun the exact
    /// compilation step for the translation unit in the environment the build system uses.
    /// Shell expansion is not supported.
    pub arguments: Vec<String>,
    /// The working directory of the compilation. All paths specified in the command or
    /// file fields must be either absolute or relative to this directory.
    pub directory: std::path::PathBuf,
    /// The name of the output created by this compilation step. This field is optional.
    /// It can be used to distinguish different processing modes of the same input file.
    pub output: Option<std::path::PathBuf>,
}

/// Serialize entries from the iterator into a Writer object.
pub fn write(
    writer: impl std::io::Write,
    entries: impl Iterator<Item = Entry>,
) -> Result<(), Error> {
    let mut ser = serde_json::Serializer::pretty(writer);
    let mut seq = ser.serialize_seq(None)?;
    for entry in entries {
        seq.serialize_element(&entry)?;
    }
    seq.end()
}

/// Deserialize entries from the Reader object into an iterator of Entries.
pub fn read(reader: impl std::io::Read) -> impl Iterator<Item = Result<Entry, Error>> {
    iterator::iter_json_array(reader)
}
