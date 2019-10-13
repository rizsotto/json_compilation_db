use std::path;

use super::Result;
use super::{CompilationDatabase, Entry, Entries};
use super::Format;


/// Represents a JSON compilation database file.
///
/// https://clang.llvm.org/docs/JSONCompilationDatabase.html
pub struct JsonCompilationDatabase<'a> {
    file: &'a path::Path,
    format: &'a Format,
}

impl<'a> JsonCompilationDatabase<'a> {

    pub fn new(path: &'a path::Path, format: &'a Format) -> Self {
        JsonCompilationDatabase { file: path, format, }
    }
}

impl<'a> CompilationDatabase for JsonCompilationDatabase<'a> {

    fn load(&self) -> Result<Entries> {
        debug!("Reading from: {:?}", self.file);
        db::load(self.file)
    }

    fn save(&self, entries: Entries) -> Result<()> {
        debug!("Writing to: {:?}", self.file);
        db::save(self.file, entries, self.format)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::config::Format;
    use std::fs;
    use std::io::{Read, Write};

    macro_rules! vec_of_strings {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    #[test]
    #[should_panic]
    fn test_load_json_failed() {
        let comp_db_file = TestFile::new()
            .expect("test file setup failed");
        comp_db_file.write(br#"this is not json"#)
            .expect("test file content write failed");

        let formatter = Format::default();
        let sut = JsonCompilationDatabase::new(comp_db_file.path(), &formatter);
        let _ = sut.load().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_load_not_expected_json_failed() {
        let comp_db_file = TestFile::new()
            .expect("test file setup failed");
        comp_db_file.write(br#"{ "file": "string" }"#)
            .expect("test file content write failed");

        let formatter = Format::default();
        let sut = JsonCompilationDatabase::new(comp_db_file.path(), &formatter);
        let _ = sut.load().unwrap();
    }

    #[test]
    fn test_load_empty() -> Result<()> {
        let comp_db_file = TestFile::new()?;
        comp_db_file.write(br#"[]"#)?;

        let formatter = Format::default();
        let sut = JsonCompilationDatabase::new(comp_db_file.path(), &formatter);
        let entries = sut.load()?;

        let expected = Entries::new();
        assert_eq!(expected, entries);
        Ok(())
    }

    #[test]
    fn test_load_string_command() -> Result<()> {
        let comp_db_file = TestFile::new()?;
        comp_db_file.write(
            br#"[
                {
                    "directory": "/home/user",
                    "file": "./file_a.c",
                    "command": "cc -c ./file_a.c -o ./file_a.o"
                },
                {
                    "directory": "/home/user",
                    "file": "./file_b.c",
                    "output": "./file_b.o",
                    "command": "cc -c ./file_b.c -o ./file_b.o"
                }
            ]"#
        )?;

        let formatter = Format::default();
        let sut = JsonCompilationDatabase::new(comp_db_file.path(), &formatter);
        let entries = sut.load()?;

        let expected = expected_values();
        assert_eq!(expected, entries);
        Ok(())
    }

    #[test]
    fn test_load_array_command() -> Result<()> {
        let comp_db_file = TestFile::new()?;
        comp_db_file.write(
            br#"[
                {
                    "directory": "/home/user",
                    "file": "./file_a.c",
                    "arguments": ["cc", "-c", "./file_a.c", "-o", "./file_a.o"]
                },
                {
                    "directory": "/home/user",
                    "file": "./file_b.c",
                    "output": "./file_b.o",
                    "arguments": ["cc", "-c", "./file_b.c", "-o", "./file_b.o"]
                }
            ]"#
        )?;

        let formatter = Format::default();
        let sut = JsonCompilationDatabase::new(comp_db_file.path(), &formatter);
        let entries = sut.load()?;

        let expected = expected_values();
        assert_eq!(expected, entries);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_load_path_problem() {
        let comp_db_file = TestFile::new()
            .expect("test file setup failed");
        comp_db_file.write(br#"[
                {
                    "directory": " ",
                    "file": "./file_a.c",
                    "command": "cc -Dvalue=\"this"
                }
            ]"#)
            .expect("test file content write failed");

        let formatter = Format::default();
        let sut = JsonCompilationDatabase::new(comp_db_file.path(), &formatter);
        let _ = sut.load().unwrap();
    }

    #[test]
    fn test_save_string_command() -> Result<()> {
        let comp_db_file = TestFile::new()?;

        let formatter = Format { command_as_array: false, ..Format::default() };
        let sut = JsonCompilationDatabase::new(comp_db_file.path(), &formatter);

        let input = expected_values();
        sut.save(input)?;

        let entries = sut.load()?;

        let expected = expected_values();
        assert_eq!(expected, entries);

        let content = comp_db_file.read()?;
        println!("{}", content);

        Ok(())
    }

    #[test]
    fn test_save_array_command() -> Result<()> {
        let comp_db_file = TestFile::new()?;

        let formatter = Format { command_as_array: true, ..Format::default() };
        let sut = JsonCompilationDatabase::new(comp_db_file.path(), &formatter);

        let input = expected_values();
        sut.save(input)?;

        let entries = sut.load()?;

        let expected = expected_values();
        assert_eq!(expected, entries);

        let content = comp_db_file.read()?;
        println!("{}", content);

        Ok(())
    }

    #[allow(dead_code)]
    struct TestFile {
        directory: tempfile::TempDir,
        file: path::PathBuf,
    }

    impl TestFile {

        pub fn new() -> Result<TestFile> {
            let directory = tempfile::Builder::new()
                .prefix("bear-test-")
                .rand_bytes(12)
                .tempdir()?;

            let mut file = directory.path().to_path_buf();
            file.push("comp-db.json");

            Ok(TestFile { directory, file })
        }

        pub fn path(&self) -> &path::Path {
            self.file.as_path()
        }

        pub fn write(&self, content: &[u8]) -> Result<()> {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(self.path())?;

            file.write(content)?;
            Ok(())
        }

        pub fn read(&self) -> Result<String> {
            let mut file = fs::OpenOptions::new()
                .read(true)
                .open(self.path())?;

            let mut result = String::new();
            file.read_to_string(&mut result)?;
            Ok(result)
        }
    }

    fn expected_values() -> Entries {
        let mut expected: Entries = Entries::new();
        expected.push(
            Entry {
                directory: path::PathBuf::from("/home/user"),
                file: path::PathBuf::from("./file_a.c"),
                command: vec_of_strings!("cc", "-c", "./file_a.c", "-o", "./file_a.o"),
                output: None,
            }
        );
        expected.push(
            Entry {
                directory: path::PathBuf::from("/home/user"),
                file: path::PathBuf::from("./file_b.c"),
                command: vec_of_strings!("cc", "-c", "./file_b.c", "-o", "./file_b.o"),
                output: Some(path::PathBuf::from("./file_b.o")),
            }
        );
        expected
    }
}


mod db {
    use super::*;
    use std::fs;
    use serde_json;
    use shellwords;

    pub fn load(path: &path::Path) -> Result<Entries> {
        let generic_entries = read(path)?;
        let entries = generic_entries.iter()
            .map(|entry| into(entry))
            .collect::<Result<Entries>>();
        // In case of error, let's be verbose which entries were problematic.
        if entries.is_err() {
            let errors = generic_entries.iter()
                .map(|entry| into(entry))
                .filter_map(Result::err)
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Err(errors.into())
        } else {
            entries
        }
    }

    pub fn save(path: &path::Path, entries: Entries, format: &Format) -> Result<()> {
        let generic_entries = entries
            .iter()
            .map(|entry| from(entry, format))
            .collect::<Result<Vec<_>>>()?;
        write(path, &generic_entries)
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    enum GenericEntry {
        StringEntry {
            directory: String,
            file: String,
            command: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            output: Option<String>,
        },
        ArrayEntry {
            directory: String,
            file: String,
            arguments: Vec<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            output: Option<String>,
        },
    }

    type GenericEntries = Vec<GenericEntry>;

    fn read(path: &path::Path) -> Result<GenericEntries> {
        let file = fs::OpenOptions::new()
            .read(true)
            .open(path)?;
        let entries: GenericEntries = serde_json::from_reader(file)?;
        Ok(entries)
    }

    fn write(path: &path::Path, entries: &[GenericEntry]) -> Result<()> {
        let file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;
        serde_json::ser::to_writer_pretty(file, entries)
            .map_err(std::convert::Into::into)
    }

    // TODO: https://doc.rust-lang.org/std/convert/trait.TryFrom.html
    fn from(entry: &Entry, format: &Format) -> Result<GenericEntry> {
        fn path_to_string(path: &path::Path) -> Result<String> {
            match path.to_str() {
                Some(str) => Ok(str.to_string()),
                None => Err(format!("Failed to convert to string {:?}", path).into()),
            }
        }

        let directory = path_to_string(entry.directory.as_path())?;
        let file = path_to_string(entry.file.as_path())?;
        let output = match entry.output {
            Some(ref path) => path_to_string(path).map(Option::Some),
            None => Ok(None),
        }?;
        if format.command_as_array {
            Ok(GenericEntry::ArrayEntry {
                directory,
                file,
                arguments: entry.command.clone(),
                output
            })
        } else {
            Ok(GenericEntry::StringEntry {
                directory,
                file,
                command: shellwords::join(
                    entry.command
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .as_ref()),
                output
            })
        }
    }

    // TODO: https://doc.rust-lang.org/std/convert/trait.TryInto.html
    fn into(entry: &GenericEntry) -> Result<Entry> {
        match entry {
            GenericEntry::ArrayEntry { directory, file, arguments, output } => {
                let directory_path = path::PathBuf::from(directory);
                let file_path = path::PathBuf::from(file);
                let output_path = output.clone().map(path::PathBuf::from);
                Ok(Entry {
                    directory: directory_path,
                    file: file_path,
                    command: arguments.clone(),
                    output: output_path,
                })
            },
            GenericEntry::StringEntry { directory, file, command, output } => {
                match shellwords::split(command) {
                    Ok(arguments) => {
                        let directory_path = path::PathBuf::from(directory);
                        let file_path = path::PathBuf::from(file);
                        let output_path = output.clone().map(path::PathBuf::from);
                        Ok(Entry {
                            directory: directory_path,
                            file: file_path,
                            command: arguments,
                            output: output_path,
                        })
                    },
                    Err(_) =>
                        Err(format!("Quotes are mismatch in {:?}", command).into()),
                }
            }
        }
    }
}
