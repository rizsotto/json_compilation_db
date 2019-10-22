use crate::api::*;
use crate::error::{Error, Error::SemanticError};

use std::path;

use serde_json::{json, Value};
use shellwords;

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

fn to_json(entry: &Entry, format: &Format) -> Result<serde_json::Value, Error> {
    match (format.command_as_array, entry.output.is_some()) {
        (true,  true)  =>
            Ok(json!({
                "directory": entry.directory,
                "file": entry.file,
                "arguments": entry.arguments,
                "output": entry.output,
            })),
        (true,  false) =>
            Ok(json!({
                "directory": entry.directory,
                "file": entry.file,
                "arguments": entry.arguments,
            })),
        (false, true)  =>
            Ok(json!({
                "directory": entry.directory,
                "file": entry.file,
                "command": to_command(entry.arguments.as_ref()),
                "output": entry.output,
            })),
        (false, false) =>
            Ok(json!({
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

    Ok(Value::Array(array))
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
    let json = to_json_array(&entries, format)?;
    let result = serde_json::to_writer_pretty(writer, &json)?;

    Ok(result)
}
