use crate::error::Result;
use crate::api::*;

use std::path;

use serde::{Serialize, Deserialize};
use serde_json;
use shellwords;

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
            output,
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
            output,
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

pub fn load_from_reader(reader: impl std::io::Read) -> Result<Entries> {
    let generic_entries: GenericEntries = serde_json::from_reader(reader)?;
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

pub fn save_into_writer(writer: impl std::io::Write, entries: Entries, format: &Format) -> Result<()> {
    let generic_entries = entries
        .iter()
        .map(|entry| from(entry, format))
        .collect::<Result<Vec<_>>>()?;

    serde_json::ser::to_writer_pretty(writer, &generic_entries)
        .map_err(std::convert::Into::into)
}
