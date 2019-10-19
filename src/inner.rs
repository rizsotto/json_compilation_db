use crate::error::Result;
use crate::api::*;

use std::path;

use serde::{Serialize, Deserialize};
use serde_json;
use shellwords;

pub fn load_from_reader(reader: impl std::io::Read) -> Result<Entries> {
    let generic_entries: GenericEntries = serde_json::from_reader(reader)?;

    try_into_entries(generic_entries)
}

pub fn save_into_writer(writer: impl std::io::Write, entries: Entries, format: &Format) -> Result<()> {
    let generic_entries: GenericEntries = try_from_entries(entries, format)?;

    serde_json::ser::to_writer_pretty(writer, &generic_entries)
        .map_err(std::convert::Into::into)
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


fn try_from_entries(values: Entries, format: &Format) -> Result<GenericEntries> {
    values
        .into_iter()
        .map(|entry| try_from_entry(entry, format))
        .collect::<Result<Vec<_>>>()
}

fn try_from_entry(value: Entry, format: &Format) -> Result<GenericEntry> {
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
            arguments: value.command.clone(),
            output,
        })
    } else {
        Ok(GenericEntry::StringEntry {
            directory,
            file,
            command: shellwords::join(
                value.command
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .as_ref()),
            output,
        })
    }
}

fn path_to_string(path: &path::Path) -> Result<String> {
    match path.to_str() {
        Some(str) => Ok(str.to_string()),
        None => Err(format!("Failed to convert to string {:?}", path).into()),
    }
}

fn try_into_entries(values: GenericEntries) -> Result<Entries> {
    values.into_iter()
        .map(|entry| try_into_entry(entry))
        .collect::<Result<Entries>>()
}

fn try_into_entry(value: GenericEntry) -> Result<Entry> {
    match value {
        GenericEntry::ArrayEntry { directory, file, arguments, output } => {
            let directory_path = path::PathBuf::from(directory);
            let file_path = path::PathBuf::from(file);
            let output_path = output.map(path::PathBuf::from);
            Ok(Entry {
                directory: directory_path,
                file: file_path,
                command: arguments.clone(),
                output: output_path,
            })
        }
        GenericEntry::StringEntry { directory, file, command, output } => {
            match shellwords::split(command.as_str()) {
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
                }
                Err(_) =>
                    Err(format!("Quotes are mismatch in {:?}", command).into()),
            }
        }
    }
}
