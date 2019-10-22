use crate::api::*;
use crate::error::{Error, Error::SemanticError};

use std::fmt;
use std::path;

use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};
use shellwords;

struct FormattedEntries<'a> {
    entries: &'a Entries,
    format: &'a Format,
}

struct FormattedEntry<'a> {
    entry: &'a Entry,
    format: &'a Format,
}

impl<'a> Serialize for FormattedEntries<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.entries.len()))?;
        for e in self.entries {
            let fe = FormattedEntry {
                entry: e,
                format: self.format,
            };
            seq.serialize_element(&fe)?;
        }
        seq.end()
    }
}

impl<'a> Serialize for FormattedEntry<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        fn to_command(arguments: &[String]) -> String {
            shellwords::join(
                arguments
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .as_ref(),
            )
        }

        match (self.format.command_as_array, self.entry.output.is_some()) {
            (true, true) => {
                let mut state = serializer.serialize_struct("Entry", 4)?;
                state.serialize_field("directory", &self.entry.directory)?;
                state.serialize_field("file", &self.entry.file)?;
                state.serialize_field("arguments", &self.entry.arguments)?;
                state.serialize_field("output", &self.entry.output)?;
                state.end()
            }
            (true, false) => {
                let mut state = serializer.serialize_struct("Entry", 3)?;
                state.serialize_field("directory", &self.entry.directory)?;
                state.serialize_field("file", &self.entry.file)?;
                state.serialize_field("arguments", &self.entry.arguments)?;
                state.end()
            }
            (false, true) => {
                let mut state = serializer.serialize_struct("Entry", 4)?;
                state.serialize_field("directory", &self.entry.directory)?;
                state.serialize_field("file", &self.entry.file)?;
                state.serialize_field("command", &to_command(&self.entry.arguments))?;
                state.serialize_field("output", &self.entry.output)?;
                state.end()
            }
            (false, false) => {
                let mut state = serializer.serialize_struct("Entry", 3)?;
                state.serialize_field("directory", &self.entry.directory)?;
                state.serialize_field("file", &self.entry.file)?;
                state.serialize_field("command", &to_command(&self.entry.arguments))?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Directory,
            File,
            Command,
            Arguments,
            Output,
        };
        const FIELDS: &[&str] =
            &["directory", "file", "command", "arguments", "output"];

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter
                            .write_str("`directory`, `file`, `command`, `arguments`, or `output`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "directory" => Ok(Field::Directory),
                            "file" => Ok(Field::File),
                            "command" => Ok(Field::Command),
                            "arguments" => Ok(Field::Arguments),
                            "output" => Ok(Field::Output),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct EntryVisitor;

        impl<'de> Visitor<'de> for EntryVisitor {
            type Value = Entry;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Entry")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Entry, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut directory: Option<path::PathBuf> = None;
                let mut file: Option<path::PathBuf> = None;
                let mut command: Option<String> = None;
                let mut arguments: Option<Vec<String>> = None;
                let mut output: Option<path::PathBuf> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Directory => {
                            if directory.is_some() {
                                return Err(de::Error::duplicate_field("directory"));
                            }
                            directory = Some(map.next_value()?);
                        }
                        Field::File => {
                            if file.is_some() {
                                return Err(de::Error::duplicate_field("file"));
                            }
                            file = Some(map.next_value()?);
                        }
                        Field::Command => {
                            if command.is_some() {
                                return Err(de::Error::duplicate_field("command"));
                            }
                            command = Some(map.next_value()?);
                        }
                        Field::Arguments => {
                            if arguments.is_some() {
                                return Err(de::Error::duplicate_field("arguments"));
                            }
                            arguments = Some(map.next_value()?);
                        }
                        Field::Output => {
                            if output.is_some() {
                                return Err(de::Error::duplicate_field("output"));
                            }
                            output = Some(map.next_value()?);
                        }
                    }
                }
                let directory = directory.ok_or_else(|| de::Error::missing_field("directory"))?;
                let file = file.ok_or_else(|| de::Error::missing_field("file"))?;
                let arguments = arguments.map_or_else(
                    || {
                        command
                            .ok_or_else(|| de::Error::missing_field("`command` or `arguments`"))
                            .and_then(|cmd| {
                                shellwords::split(cmd.as_str()).map_err(|_| {
                                    de::Error::invalid_value(
                                        de::Unexpected::Str(cmd.as_str()),
                                        &"quotes needs to be matched",
                                    )
                                })
                            })
                    },
                    Ok,
                )?;
                Ok(Entry {
                    directory,
                    file,
                    arguments,
                    output,
                })
            }
        }

        deserializer.deserialize_struct("Entry", FIELDS, EntryVisitor)
    }
}

fn validate(entries: &[Entry]) -> Result<(), Error> {
    let _ = entries
        .iter()
        .map(|entry| {
            if entry.arguments.is_empty() {
                return Err(SemanticError("Field `argument` can't be empty array."));
            }

            Ok(())
        })
        .collect::<Result<Vec<()>, Error>>()?;

    Ok(())
}

pub fn load_from_reader(reader: impl std::io::Read) -> Result<Entries, Error> {
    let entries: Entries = serde_json::from_reader(reader)?;
    validate(&entries)?;

    Ok(entries)
}

pub fn save_into_writer(
    writer: impl std::io::Write,
    entries: Entries,
    format: &Format,
) -> Result<(), Error> {
    validate(&entries)?;

    let fe = FormattedEntries {
        entries: &entries,
        format,
    };
    serde_json::to_writer_pretty(writer, &fe)?;

    Ok(())
}
