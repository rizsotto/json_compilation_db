use json_cdb::*;

#[test]
fn load_failed_on_not_existing_file() {
    let comp_db_file = std::path::Path::new("/not/existing/path");
    let result = load_from_file(&comp_db_file);
    assert_io_error!(&result);
}

#[test]
fn load_failed_on_non_json_content() -> Result<()> {
    let directory = fixtures::create_test_dir()?;
    let file = fixtures::create_file_with_content(&directory, br#"this is not json"#)?;

    let result = load_from_file(file.as_path());
    assert_syntax_error!(&result);

    Ok(())
}

#[test]
fn load_failed_not_expected_json_content() -> Result<()> {
    let directory = fixtures::create_test_dir()?;
    let file = fixtures::create_file_with_content(&directory, br#"{ "file": "string" }"#)?;

    let result = load_from_file(file.as_path());
    assert_syntax_error!(&result);

    Ok(())
}

#[test]
fn load_failed_on_semantic_problem() -> Result<()> {
    let directory = fixtures::create_test_dir()?;
    let file = fixtures::create_file_with_content(
        &directory,
        br#"[
            {
                "directory": " ",
                "file": "./file_a.c",
                "command": "cc -Dvalue=\"this"
            }
        ]"#,
    )?;

    let result = load_from_file(file.as_path());
    assert_semantic_error!(result);

    Ok(())
}

#[test]
fn load_successful_on_empty_array() -> Result<()> {
    let expected = Entries::new();

    let directory = fixtures::create_test_dir()?;
    let file = fixtures::create_file_with_content(&directory, br#"[]"#)?;

    let entries = load_from_file(file.as_path())?;
    assert_eq!(expected, entries);

    Ok(())
}

#[test]
fn load_successful_content_with_string_command_syntax() -> Result<()> {
    let expected = fixtures::expected_values();

    let directory = fixtures::create_test_dir()?;
    let file = fixtures::create_file_with_content(
        &directory,
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
        ]"#,
    )?;

    let entries = load_from_file(file.as_path())?;
    assert_eq!(expected, entries);

    Ok(())
}

#[test]
fn load_successful_content_with_array_command_syntax() -> Result<()> {
    let expected = fixtures::expected_values();

    let directory = fixtures::create_test_dir()?;
    let file = fixtures::create_file_with_content(
        &directory,
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
        ]"#,
    )?;

    let entries = load_from_file(file.as_path())?;
    assert_eq!(expected, entries);
    Ok(())
}

#[test]
fn save_failed_on_not_exsisting_directory() {
    let comp_db_file = std::path::Path::new("/not/existing/path");

    let input = fixtures::expected_values();
    let format = Format::default();

    let result = save_into_file(&comp_db_file, input, &format);
    assert_io_error!(&result);
}

#[test]
fn save_successful_with_string_command_syntax() -> Result<()> {
    let expected = fixtures::expected_values();
    let input = fixtures::expected_values();

    let directory = fixtures::create_test_dir()?;
    let file = fixtures::create_file(&directory);
    let format = Format {
        command_as_array: false,
    };

    save_into_file(file.as_path(), input, &format)?;

    let entries = load_from_file(file.as_path())?;
    assert_eq!(expected, entries);

    let content = fixtures::read_content_from(file.as_path())?;
    println!("{}", content);

    Ok(())
}

#[test]
fn save_successful_with_array_command_syntax() -> Result<()> {
    let expected = fixtures::expected_values();
    let input = fixtures::expected_values();

    let directory = fixtures::create_test_dir()?;
    let file = fixtures::create_file(&directory);
    let format = Format {
        command_as_array: true,
    };

    save_into_file(file.as_path(), input, &format)?;

    let entries = load_from_file(file.as_path())?;
    assert_eq!(expected, entries);

    let content = fixtures::read_content_from(file.as_path())?;
    println!("{}", content);

    Ok(())
}

mod fixtures {
    use super::*;
    use std::fs;
    use std::io::{Read, Write};
    use std::path;

    #[macro_export]
    macro_rules! assert_io_error {
        ($x:expr) => {
            match $x {
                Err(Error::IoError(_)) => assert!(true),
                _ => assert!(false, "shout be io error"),
            }
        };
    }

    #[macro_export]
    macro_rules! assert_syntax_error {
        ($x:expr) => {
            match $x {
                Err(Error::SyntaxError(_)) => assert!(true),
                _ => assert!(false, "shout be syntax error"),
            }
        };
    }

    #[macro_export]
    macro_rules! assert_semantic_error {
        ($x:expr) => {
            match $x {
                Err(Error::SemanticError(_)) => assert!(true),
                _ => assert!(false, "shout be semantic error"),
            }
        };
    }

    macro_rules! vec_of_strings {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    pub fn create_test_dir() -> Result<tempfile::TempDir> {
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

    fn with_content(path: &path::Path, content: &[u8]) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        file.write_all(content)?;
        Ok(())
    }

    pub fn create_file_with_content(
        directory: &tempfile::TempDir,
        content: &[u8],
    ) -> Result<path::PathBuf> {
        let file = create_file(directory);
        with_content(file.as_path(), content)?;
        Ok(file)
    }

    pub fn read_content_from(path: &path::Path) -> Result<String> {
        let mut file = fs::OpenOptions::new().read(true).open(path)?;

        let mut result = String::new();
        file.read_to_string(&mut result)?;
        Ok(result)
    }

    pub fn expected_values() -> Entries {
        vec![
            Entry {
                directory: path::PathBuf::from("/home/user"),
                file: path::PathBuf::from("./file_a.c"),
                command: vec_of_strings!("cc", "-c", "./file_a.c", "-o", "./file_a.o"),
                output: None,
            },
            Entry {
                directory: path::PathBuf::from("/home/user"),
                file: path::PathBuf::from("./file_b.c"),
                command: vec_of_strings!("cc", "-c", "./file_b.c", "-o", "./file_b.o"),
                output: Some(path::PathBuf::from("./file_b.o")),
            },
        ]
    }
}
