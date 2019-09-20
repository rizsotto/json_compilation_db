use super::Result;

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

/// Represents a compilation database.
pub trait CompilationDatabase {

    fn load(&self) -> Result<Entries>;

    fn save(&self, entries: Entries) -> Result<()>;
}

/// Represents an entry of the compilation database.
#[derive(Debug)]
pub struct Entry {
    pub directory: std::path::PathBuf,
    pub file: std::path::PathBuf,
    pub command: Vec<String>,
    pub output: Option<std::path::PathBuf>,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.directory == other.directory
            && self.file == other.file
            && self.command == other.command
    }
}

pub type Entries = Vec<Entry>;
