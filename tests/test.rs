use json_cdb::*;
use serde_json::json;

mod failures {
    use super::*;

    // TODO: check how meaningful the error string is.
    macro_rules! assert_io_error {
        ($x:expr) => {
            match $x {
                Err(Error::IoError(_)) => assert!(true),
                _ => assert!(false, "shout be io error"),
            }
        };
    }

    // TODO: check how meaningful the error string is.
    macro_rules! assert_syntax_error {
        ($x:expr) => {
            match $x {
                Err(Error::SyntaxError(_)) => assert!(true),
                _ => assert!(false, "shout be syntax error"),
            }
        };
    }

    // TODO: check how meaningful the error string is.
    macro_rules! assert_semantic_error {
        ($x:expr) => {
            match $x {
                Err(Error::SemanticError(_)) => assert!(true),
                _ => assert!(false, "shout be semantic error"),
            }
        };
    }

    #[test]
    fn load_not_existing_file() {
        let file = std::path::Path::new("/not/existing/path");

        let result = load_from_file(file);

        assert_io_error!(&result);
    }

    #[test]
    fn load_non_json_content() -> Result<(), Error> {
        let directory = fixtures::create_test_dir()?;
        let file = fixtures::create_file_with_content(&directory, br#"this is not json"#)?;

        let result = load_from_file(file.as_path());

        assert_syntax_error!(&result);

        Ok(())
    }

    // TODO: this should be a semantic error.
    #[test]
    fn load_not_expected_json_content() -> Result<(), Error> {
        let content = json!({ "file": "string" });
        let directory = fixtures::create_test_dir()?;
        let file = fixtures::create_file_with_json_content(&directory, content)?;

        let result = load_from_file(file.as_path());

        assert_syntax_error!(&result);

        Ok(())
    }

    #[test]
    fn load_on_semantic_problem() -> Result<(), Error> {
        let content = json!([
            {
                "directory": " ",
                "file": "./file_a.c",
                "command": "cc -Dvalue=\"this"
            }
        ]);
        let directory = fixtures::create_test_dir()?;
        let file = fixtures::create_file_with_json_content(&directory, content)?;

        let result = load_from_file(file.as_path());

        assert_semantic_error!(result);

        Ok(())
    }

    #[test]
    fn save_not_existing_directory() {
        let file = std::path::Path::new("/not/existing/path");
        let input = vec![Entry {
            directory: std::path::PathBuf::from("/home/user"),
            file: std::path::PathBuf::from("./file_a.c"),
            arguments: vec_of_strings!("cc", "-c", "./file_a.c", "-o", "./file_a.o"),
            output: None,
        }];
        let format = Format::default();

        let result = save_into_file(file, input, &format);

        assert_io_error!(&result);
    }
}

mod success {
    use super::*;

    mod empty {
        use super::*;

        #[test]
        fn load_empty_array() -> Result<(), Error> {
            let content = json!([]);
            let directory = fixtures::create_test_dir()?;
            let file = fixtures::create_file_with_json_content(&directory, content)?;

            let entries = load_from_file(file.as_path())?;

            let expected = Entries::new();
            assert_eq!(expected, entries);

            Ok(())
        }
    }

    mod basic {
        use super::*;

        fn expected_values() -> Entries {
            vec![
                Entry {
                    directory: std::path::PathBuf::from("/home/user"),
                    file: std::path::PathBuf::from("./file_a.c"),
                    arguments: vec_of_strings!("cc", "-c", "./file_a.c", "-o", "./file_a.o"),
                    output: None,
                },
                Entry {
                    directory: std::path::PathBuf::from("/home/user"),
                    file: std::path::PathBuf::from("./file_b.c"),
                    arguments: vec_of_strings!("cc", "-c", "./file_b.c", "-o", "./file_b.o"),
                    output: Some(std::path::PathBuf::from("./file_b.o")),
                },
            ]
        }

        fn expected_with_array_syntax() -> serde_json::Value {
            json!([
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
            ])
        }

        fn expected_with_string_syntax() -> serde_json::Value {
            json!([
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
            ])
        }

        #[test]
        fn load_content_with_string_command_syntax() -> Result<(), Error> {
            let content = expected_with_string_syntax();
            let directory = fixtures::create_test_dir()?;
            let file = fixtures::create_file_with_json_content(&directory, content)?;

            let entries = load_from_file(file.as_path())?;

            let expected = expected_values();
            assert_eq!(expected, entries);

            Ok(())
        }

        #[test]
        fn load_content_with_array_command_syntax() -> Result<(), Error> {
            let content = expected_with_array_syntax();
            let directory = fixtures::create_test_dir()?;
            let file = fixtures::create_file_with_json_content(&directory, content)?;

            let entries = load_from_file(file.as_path())?;

            let expected = expected_values();
            assert_eq!(expected, entries);

            Ok(())
        }

        #[test]
        fn save_with_string_command_syntax() -> Result<(), Error> {
            let directory = fixtures::create_test_dir()?;
            let file = fixtures::create_file(&directory);
            let input = expected_values();
            let format = Format {
                command_as_array: false,
            };

            save_into_file(file.as_path(), input, &format)?;

            let content = fixtures::read_json_from(file.as_path())?;
            let expected = expected_with_string_syntax();
            assert_eq!(expected, content);

            Ok(())
        }

        #[test]
        fn save_with_array_command_syntax() -> Result<(), Error> {
            let directory = fixtures::create_test_dir()?;
            let file = fixtures::create_file(&directory);
            let input = expected_values();
            let format = Format {
                command_as_array: true,
            };

            save_into_file(file.as_path(), input, &format)?;

            let content = fixtures::read_json_from(file.as_path())?;
            let expected = expected_with_array_syntax();
            assert_eq!(expected, content);

            Ok(())
        }
    }

    mod quoted {
        use super::*;

        fn expected_values() -> Entries {
            vec![
                Entry {
                    directory: std::path::PathBuf::from("/home/user"),
                    file: std::path::PathBuf::from("./file_a.c"),
                    arguments: vec_of_strings!(
                        "cc",
                        "-c",
                        "-D",
                        "name=\\\"me\\\"",
                        "./file_a.c",
                        "-o",
                        "./file_a.o"
                    ),
                    output: None,
                },
                Entry {
                    directory: std::path::PathBuf::from("/home/user"),
                    file: std::path::PathBuf::from("./file_b.c"),
                    arguments: vec_of_strings!(
                        "cc",
                        "-c",
                        "-D",
                        "name=\"me\"",
                        "./file_b.c",
                        "-o",
                        "./file_b.o"
                    ),
                    output: None,
                },
            ]
        }

        fn expected_with_array_syntax() -> serde_json::Value {
            json!([
                {
                    "directory": "/home/user",
                    "file": "./file_a.c",
                    "arguments": ["cc", "-c", "-D", "name=\\\"me\\\"", "./file_a.c", "-o", "./file_a.o"]
                },
                {
                    "directory": "/home/user",
                    "file": "./file_b.c",
                    "arguments": ["cc", "-c", "-D", "name=\"me\"", "./file_b.c", "-o", "./file_b.o"]
                }
            ])
        }

        fn expected_with_string_syntax() -> serde_json::Value {
            json!([
                {
                    "directory": "/home/user",
                    "file": "./file_a.c",
                    "command": "cc -c -D name\\=\\\\\\\"me\\\\\\\" ./file_a.c -o ./file_a.o"
                },
                {
                    "directory": "/home/user",
                    "file": "./file_b.c",
                    "command": "cc -c -D name\\=\\\"me\\\" ./file_b.c -o ./file_b.o"
                }
            ])
        }

        #[test]
        fn load_content_with_string_command_syntax() -> Result<(), Error> {
            let content = expected_with_string_syntax();
            let directory = fixtures::create_test_dir()?;
            let file = fixtures::create_file_with_json_content(&directory, content)?;

            let entries = load_from_file(file.as_path())?;

            let expected = expected_values();
            assert_eq!(expected, entries);

            Ok(())
        }

        #[test]
        fn load_content_with_array_command_syntax() -> Result<(), Error> {
            let content = expected_with_array_syntax();
            let directory = fixtures::create_test_dir()?;
            let file = fixtures::create_file_with_json_content(&directory, content)?;

            let entries = load_from_file(file.as_path())?;

            let expected = expected_values();
            assert_eq!(expected, entries);

            Ok(())
        }

        #[test]
        fn save_with_string_command_syntax() -> Result<(), Error> {
            let directory = fixtures::create_test_dir()?;
            let file = fixtures::create_file(&directory);
            let input = expected_values();
            let format = Format {
                command_as_array: false,
            };

            save_into_file(file.as_path(), input, &format)?;

            let content = fixtures::read_json_from(file.as_path())?;
            let expected = expected_with_string_syntax();
            assert_eq!(expected, content);

            Ok(())
        }

        #[test]
        fn save_with_array_command_syntax() -> Result<(), Error> {
            let directory = fixtures::create_test_dir()?;
            let file = fixtures::create_file(&directory);
            let input = expected_values();
            let format = Format {
                command_as_array: true,
            };

            save_into_file(file.as_path(), input, &format)?;

            let content = fixtures::read_json_from(file.as_path())?;
            let expected = expected_with_array_syntax();
            assert_eq!(expected, content);

            Ok(())
        }
    }
}

mod fixtures {
    use super::*;
    use serde_json::Value;
    use std::fs;
    use std::io::Write;
    use std::path;

    #[macro_export]
    macro_rules! vec_of_strings {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    pub fn create_test_dir() -> Result<tempfile::TempDir, Error> {
        let directory = tempfile::Builder::new()
            .prefix("json_cdb_test-")
            .rand_bytes(12)
            .tempdir()?;

        Ok(directory)
    }

    pub fn create_file(directory: &tempfile::TempDir) -> path::PathBuf {
        let mut file = directory.path().to_path_buf();
        file.push(DEFAULT_FILE_NAME);

        file
    }

    pub fn create_file_with_content(
        directory: &tempfile::TempDir,
        content: &[u8],
    ) -> Result<path::PathBuf, Error> {
        let path = create_file(directory);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.as_path())?;

        file.write_all(content)?;
        Ok(path)
    }

    pub fn create_file_with_json_content(
        directory: &tempfile::TempDir,
        content: Value,
    ) -> Result<path::PathBuf, Error> {
        let path = create_file(directory);
        let file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.as_path())?;
        serde_json::to_writer(file, &content)?;
        Ok(path)
    }

    pub fn read_json_from(path: &path::Path) -> Result<Value, Error> {
        let file = fs::OpenOptions::new().read(true).open(path)?;
        let content: Value = serde_json::from_reader(file)?;
        Ok(content)
    }
}
