use crate::Result;
use std::fs;

/// Represents an entry of the compilation database.
#[derive(Debug)]
pub struct Entry {
    pub file: std::path::PathBuf,
    pub command: Vec<String>,
    pub directory: std::path::PathBuf,
    pub output: Option<std::path::PathBuf>,
}

pub type Entries = Vec<Entry>;

/// Represents the expected format of the JSON compilation database.
#[derive(Debug, PartialEq, Eq)]
pub struct Format {
    pub command_as_array: bool,
    pub drop_output_field: bool,
}

impl Default for Format {
    fn default() -> Self {
        Format {
            command_as_array: true,
            drop_output_field: false,
        }
    }
}

pub fn load_from_file(file: &std::path::Path) -> Result<Entries> {
    let reader = fs::OpenOptions::new()
        .read(true)
        .open(file)?;

    load_from_reader(reader)
}

pub fn load_from_reader(_reader: impl std::io::Read) -> Result<Entries> {
    unimplemented!()
}

pub fn save_into_file(file: &std::path::Path, entries: Entries, format: &Format) -> Result<()> {
    let writer = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file)?;

    save_into_writer(writer, entries, format)
}

pub fn save_into_writer(_writer: impl std::io::Write, _entries: Entries, _format: &Format) -> Result<()> {
    unimplemented!()
}
