use crate::api::*;
use crate::error::Error;

use std::path;

use serde::{Deserialize, Serialize};
use serde_json;
use shellwords;

fn validate(entry: &Entry) -> Result<(), Error> {
    // TODO: add validation
    Ok(())
}

fn validate_array(entries: &Entries) -> Result<(), Error> {
    // TODO: add validation
    Ok(())
}

fn to_command(arguments: &[String]) -> String {
    shellwords::join(
        arguments
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_ref(),
    )
}

fn to_json(entry: &Entry, format: &Format) -> Result<serde_json::Value, Error> {
    match (format.command_as_array, entry.output.is_some()) {
        (true,  true)  =>
            Ok(serde_json::json!({
                "directory": entry.directory,
                "file": entry.file,
                "arguments": entry.arguments,
                "output": entry.output,
            })),
        (true,  false) =>
            Ok(serde_json::json!({
                "directory": entry.directory,
                "file": entry.file,
                "arguments": entry.arguments,
            })),
        (false, true)  =>
            Ok(serde_json::json!({
                "directory": entry.directory,
                "file": entry.file,
                "command": to_command(entry.arguments.as_ref()),
                "output": entry.output,
            })),
        (false, false) =>
            Ok(serde_json::json!({
                "directory": entry.directory,
                "file": entry.file,
                "command": to_command(entry.arguments.as_ref()),
            })),
    }
}

fn to_json_array(entries: &Entries, format: &Format) -> Result<serde_json::Value, Error> {
    let array = entries
        .into_iter()
        .map(|entry| to_json(&entry, format))
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(serde_json::Value::Array(array))
}

fn from_json(value: &serde_json::Value) -> Result<Entry, Error> {
    unimplemented!()
}

pub fn load_from_reader(reader: impl std::io::Read) -> Result<Entries, Error> {
    // TODO: add validation
    let generic_entries: GenericEntries = serde_json::from_reader(reader)?;

    try_into_entries(generic_entries)
}

pub fn save_into_writer(
    writer: impl std::io::Write,
    entries: Entries,
    format: &Format,
) -> Result<(), Error> {
    let _ = validate_array(&entries)?;
    let json = to_json_array(&entries, format)?;
    let result = serde_json::to_writer_pretty(writer, &json)?;

    Ok(result)
}

// TODO: kill this type and use raw `serde_json::Value` type!

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

fn try_from_entries(values: Entries, format: &Format) -> Result<GenericEntries, Error> {
    values
        .into_iter()
        .map(|entry| try_from_entry(entry, format))
        .collect::<Result<Vec<_>, Error>>()
}

fn try_from_entry(value: Entry, format: &Format) -> Result<GenericEntry, Error> {
    let directory = path_to_string(value.directory.as_path())?;
    let file = path_to_string(value.file.as_path())?;
    let output = match value.output {
        Some(ref path) => path_to_string(path).map(Option::Some),
        None => Ok(None),
    }?;
    if format.command_as_array {
        Ok(GenericEntry::ArrayEntry {
            directory,
            file,
            arguments: value.arguments.clone(),
            output,
        })
    } else {
        Ok(GenericEntry::StringEntry {
            directory,
            file,
            command: shellwords::join(
                value
                    .arguments
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .as_ref(),
            ),
            output,
        })
    }
}

fn path_to_string(path: &path::Path) -> Result<String, Error> {
    match path.to_str() {
        Some(str) => Ok(str.to_string()),
        None => Err(format!("Failed to convert to string {:?}", path).into()),
    }
}

fn try_into_entries(values: GenericEntries) -> Result<Entries, Error> {
    values
        .into_iter()
        .map(try_into_entry)
        .collect::<Result<Entries, Error>>()
}

fn try_into_entry(value: GenericEntry) -> Result<Entry, Error> {
    match value {
        GenericEntry::ArrayEntry {
            directory,
            file,
            arguments,
            output,
        } => {
            let directory_path = path::PathBuf::from(directory);
            let file_path = path::PathBuf::from(file);
            let output_path = output.map(path::PathBuf::from);
            Ok(Entry {
                directory: directory_path,
                file: file_path,
                arguments: arguments.clone(),
                output: output_path,
            })
        }
        GenericEntry::StringEntry {
            directory,
            file,
            command,
            output,
        } => match shellwords::split(command.as_str()) {
            Ok(arguments) => {
                let directory_path = path::PathBuf::from(directory);
                let file_path = path::PathBuf::from(file);
                let output_path = output.clone().map(path::PathBuf::from);
                Ok(Entry {
                    directory: directory_path,
                    file: file_path,
                    arguments: arguments,
                    output: output_path,
                })
            }
            Err(_) => Err(format!("Quotes are mismatch in {:?}", command).into()),
        },
    }
}
