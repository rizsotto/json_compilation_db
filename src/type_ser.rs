use crate::api::*;

use serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};
use shellwords;

pub struct FormattedEntries<'a> {
    pub entries: &'a Entries,
    pub format: &'a Format,
}

pub struct FormattedEntry<'a> {
    pub entry: &'a Entry,
    pub format: &'a Format,
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

        let size = if self.entry.output.is_some() { 4 } else { 3 };
        let mut state = serializer.serialize_struct("Entry", size)?;
        state.serialize_field("directory", &self.entry.directory)?;
        state.serialize_field("file", &self.entry.file)?;
        if self.format.command_as_array {
            state.serialize_field("arguments", &self.entry.arguments)?;
        } else {
            state.serialize_field("command", &to_command(&self.entry.arguments))?;
        }
        if self.entry.output.is_some() {
            state.serialize_field("output", &self.entry.output)?;
        }
        state.end()
    }
}
