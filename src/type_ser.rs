use crate::*;

use serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};

pub struct FormattedEntries<'a> {
    entries: &'a [Entry],
    format: &'a Format,
}

impl<'a> FormattedEntries<'a> {
    pub fn new(entries: &'a [Entry], format: &'a Format) -> Self {
        FormattedEntries { entries, format }
    }
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

struct FormattedEntry<'a> {
    entry: &'a Entry,
    format: &'a Format,
}

impl<'a> Serialize for FormattedEntry<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let size = if self.entry.output.is_some() { 4 } else { 3 };
        let mut state = serializer.serialize_struct("Entry", size)?;
        state.serialize_field("directory", &self.entry.directory)?;
        state.serialize_field("file", &self.entry.file)?;
        if self.format.command_as_array {
            state.serialize_field("arguments", &self.entry.arguments)?;
        } else {
            let command = shell_words::join(&self.entry.arguments);
            state.serialize_field("command", &command)?;
        }
        if self.entry.output.is_some() && !self.format.drop_output_field {
            state.serialize_field("output", &self.entry.output)?;
        }
        state.end()
    }
}
