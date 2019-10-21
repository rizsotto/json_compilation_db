use crate::api::*;
use crate::error::Error;

use std::path;

use serde::{Deserialize, Serialize};
use serde_json;
use shellwords;

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
    // TODO: add validation
    let generic_entries: GenericEntries = try_from_entries(entries, format)?;

    serde_json::ser::to_writer_pretty(writer, &generic_entries).map_err(std::convert::Into::into)
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
