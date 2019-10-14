use crate::Result;

/// Represents an entry of the compilation database.
#[derive(Debug)]
pub struct Entry {
    pub file: std::path::PathBuf,
    pub command: Vec<String>,
    pub directory: std::path::PathBuf,
    pub output: Option<std::path::PathBuf>,
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

pub fn load_from_file(file: &std::path::Path) -> Result<Entries> {
    let reader = std::fs::OpenOptions::new()
        .read(true)
        .open(file)?;

    load_from_reader(reader)
}

pub fn load_from_reader(reader: impl std::io::Read) -> Result<Entries> {
    crate::file::load_from_reader(reader)
}

pub fn save_into_file(file: &std::path::Path, entries: Entries, format: &Format) -> Result<()> {
    let writer = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file)?;

    save_into_writer(writer, entries, format)
}

pub fn save_into_writer(writer: impl std::io::Write, entries: Entries, format: &Format) -> Result<()> {
    crate::file::save_into_writer(writer, entries, format)
}


#[cfg(test)]
mod test {
    use super::*;
    use fixtures::*;

    impl PartialEq for Entry {
        fn eq(&self, other: &Entry) -> bool {
            self.directory == other.directory
                && self.file == other.file
                && self.command == other.command
        }
    }

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

        // TODO: expect syntax error
        load_from_file(comp_db_file.path()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_load_not_expected_json_failed() {
        let comp_db_file = TestFile::new()
            .expect("test file setup failed");
        comp_db_file.write(br#"{ "file": "string" }"#)
            .expect("test file content write failed");

        // TODO: expect syntax error
        load_from_file(comp_db_file.path()).unwrap();
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

        // TODO: expect semantic error
        load_from_file(comp_db_file.path()).unwrap();
    }

    #[test]
    fn test_load_empty() -> Result<()> {
        let comp_db_file = TestFile::new()?;
        comp_db_file.write(br#"[]"#)?;

        let entries = load_from_file(comp_db_file.path())?;

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

        let entries = load_from_file(comp_db_file.path())?;

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

        let entries = load_from_file(comp_db_file.path())?;

        let expected = expected_values();
        assert_eq!(expected, entries);
        Ok(())
    }

    #[test]
    fn test_save_string_command() -> Result<()> {
        let comp_db_file = TestFile::new()?;

        let input = expected_values();
        let format = Format { command_as_array: false, ..Format::default() };

        save_into_file(comp_db_file.path(), input, &format)?;

        let entries = load_from_file(comp_db_file.path())?;

        let expected = expected_values();
        assert_eq!(expected, entries);

        let content = comp_db_file.read()?;
        println!("{}", content);

        Ok(())
    }

    #[test]
    fn test_save_array_command() -> Result<()> {
        let comp_db_file = TestFile::new()?;

        let input = expected_values();
        let format = Format { command_as_array: true, ..Format::default() };

        save_into_file(comp_db_file.path(), input, &format)?;

        let entries = load_from_file(comp_db_file.path())?;

        let expected = expected_values();
        assert_eq!(expected, entries);

        let content = comp_db_file.read()?;
        println!("{}", content);

        Ok(())
    }

    mod fixtures {

        use super::*;
        use std::path;
        use std::fs;
        use std::io::{Read, Write};

        #[allow(dead_code)]
        pub struct TestFile {
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

        pub fn expected_values() -> Entries {
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
}
