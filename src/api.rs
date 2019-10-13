use crate::Result;

/// Represents an entry of the compilation database.
#[derive(Debug)]
pub struct Entry {
    pub file: std::path::PathBuf,
    pub command: Vec<String>,
    pub directory: std::path::PathBuf,
    pub output: Option<std::path::PathBuf>,
}

// TODO: Clarify if this is useful? (What is this solving?)
impl PartialEq for Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.file == other.file
            && self.command == other.command
            && self.directory == other.directory
    }
}

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

pub type Entries = Vec<Entry>;

pub fn load_from_file(_file: &std::path::Path) -> Result<Entries> {
    unimplemented!()
}

pub fn load_from_reader(_reader: impl std::io::Read) -> Result<Entries> {
    unimplemented!()
}

pub fn save_into_file(_file: &std::path::Path, _entries: Entries, _format: &Format) -> Result<()> {
    unimplemented!()
}

pub fn save_into_writeer(_writer: impl std::io::Write, _entries: Entries, _format: &Format) -> Result<()> {
    unimplemented!()
}
