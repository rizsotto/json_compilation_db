use crate::api::*;
use crate::error::{Error, Error::SemanticError};

use std::path;

use serde_json::Value;
use shellwords;
use serde::ser::{Serialize, Serializer, SerializeStruct, SerializeSeq};

struct FormattedEntries <'a> {
    entries: &'a Entries,
    format: &'a Format,
}

struct FormattedEntry <'a> {
    entry: &'a Entry,
    format: &'a Format,
}

impl<'a> Serialize for FormattedEntries <'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.entries.len()))?;
        for e in self.entries {
            let fe = FormattedEntry { entry: e, format: self.format };
            seq.serialize_element(&fe)?;
        }
        seq.end()
    }
}

impl<'a> Serialize for FormattedEntry <'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match (self.format.command_as_array, self.entry.output.is_some()) {
            (true,  true)  => {
                let mut state = serializer.serialize_struct("Entry", 4)?;
                state.serialize_field("directory", &self.entry.directory)?;
                state.serialize_field("file", &self.entry.file)?;
                state.serialize_field("arguments", &self.entry.arguments)?;
                state.serialize_field("output", &self.entry.output)?;
                state.end()
            },
            (true,  false)  => {
                let mut state = serializer.serialize_struct("Entry", 3)?;
                state.serialize_field("directory", &self.entry.directory)?;
                state.serialize_field("file", &self.entry.file)?;
                state.serialize_field("arguments", &self.entry.arguments)?;
                state.end()
            },
            (false, true)  => {
                let mut state = serializer.serialize_struct("Entry", 4)?;
                state.serialize_field("directory", &self.entry.directory)?;
                state.serialize_field("file", &self.entry.file)?;
                state.serialize_field("command", &to_command(&self.entry.arguments))?;
                state.serialize_field("output", &self.entry.output)?;
                state.end()
            },
            (false, false)  => {
                let mut state = serializer.serialize_struct("Entry", 3)?;
                state.serialize_field("directory", &self.entry.directory)?;
                state.serialize_field("file", &self.entry.file)?;
                state.serialize_field("command", &to_command(&self.entry.arguments))?;
                state.end()
            },
        }
    }
}


fn validate(entry: &Entry) -> Result<(), Error> {
    if entry.arguments.is_empty() {
        return Err(SemanticError("Field `argument` can't be empty array."))
    }

    Ok(())
}

fn validate_array(entries: &Entries) -> Result<(), Error> {
    let _ = entries.iter()
        .map(validate)
        .collect::<Result<Vec<()>, Error>>()?;

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

fn to_arguments(command: String) -> Result<Vec<String>, Error> {
    shellwords::split(command.as_str())
        .map_err(|_| SemanticError("Mismatched quotes in `command` field."))
}

fn as_path(value: &Value) -> Result<path::PathBuf, Error> {
    // TODO: fix error message
    match value {
        Value::String(content) => Ok(path::PathBuf::from(content)),
        _ => Err(SemanticError("Entry field expected to be string.")),
    }
}

fn as_string(value: &Value) -> Result<String, Error> {
    // TODO: fix error message
    match value {
        Value::String(content) => Ok(content.clone()),
        _ => Err(SemanticError("Entry field expected to be string."))
    }
}

fn as_array(value: &Value) -> Result<Vec<String>, Error> {
    // TODO: fix error message
    match value {
        Value::Array(values) => {
            values.iter()
                .map(as_string)
                .collect()
        },
        _ => Err(SemanticError("Entry field `arguments` expected to be array of strings.")),
    }
}

fn from_json(value: &Value) -> Result<Entry, Error> {
    match value {
        Value::Object(entry) => {
            let directory: path::PathBuf = entry.get("directory")
                .ok_or(SemanticError("Entry field `directory` is required"))
                .and_then(as_path)?;
            let file: path::PathBuf = entry.get("file")
                .ok_or(SemanticError("Entry field `file` is required."))
                .and_then(as_path)?;
            let output: Option<path::PathBuf> = entry.get("output")
                .map(as_path)
                .transpose()?;
            let arguments: Vec<String> = entry.get("arguments")
                .map_or_else(|| entry.get("command")
                    .ok_or(SemanticError("Either `command` or `arguments` is required."))
                    .and_then(as_string)
                    .and_then(to_arguments),
                             as_array)?;

            Ok(Entry { file, arguments, directory, output })
        },
        _ => Err(SemanticError("Compilation database entry expected to be JSON object"))
    }
}

fn from_json_array(value: &Value) -> Result<Entries, Error> {
    match value {
        Value::Array(values) => {
            values.iter()
                .map(from_json)
                .collect()
        },
        _ => Err(SemanticError("Compilation database content expected to be JSON array"))
    }
}

pub fn load_from_reader(reader: impl std::io::Read) -> Result<Entries, Error> {
    let values: Value = serde_json::from_reader(reader)?;
    let entries = from_json_array(&values)?;
    let _ = validate_array(&entries)?;

    Ok(entries)
}

pub fn save_into_writer(
    writer: impl std::io::Write,
    entries: Entries,
    format: &Format,
) -> Result<(), Error> {
    let _ = validate_array(&entries)?;
    let fe = FormattedEntries { entries: &entries, format };
    let result = serde_json::to_writer_pretty(writer, &fe)?;

    Ok(result)
}
