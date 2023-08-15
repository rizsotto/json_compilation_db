// from https://github.com/serde-rs/json/issues/404#issuecomment-892957228

use std::io::{self, Read};

use serde::de::DeserializeOwned;
use serde_json::{Deserializer, Error, Result};

pub fn iter_json_array<T, R>(mut reader: R) -> impl Iterator<Item=Result<T>>
    where
        T: DeserializeOwned,
        R: io::Read,
{
    let mut at_start = false;
    std::iter::from_fn(move || yield_next_obj(&mut reader, &mut at_start).transpose())
}

fn yield_next_obj<T, R>(mut reader: R, at_start: &mut bool) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        R: io::Read,
{
    if !*at_start {
        *at_start = true;
        if read_skipping_ws(&mut reader)? == b'[' {
            // read the next char to see if the array is empty
            let peek = read_skipping_ws(&mut reader)?;
            if peek == b']' {
                Ok(None)
            } else {
                deserialize_single(io::Cursor::new([peek]).chain(reader)).map(Some)
            }
        } else {
            Err(serde::de::Error::custom("expected `[`"))
        }
    } else {
        match read_skipping_ws(&mut reader)? {
            b',' => deserialize_single(reader).map(Some),
            b']' => Ok(None),
            _ => Err(serde::de::Error::custom("expected `,` or `]`")),
        }
    }
}

fn deserialize_single<T, R>(reader: R) -> Result<T>
    where
        T: DeserializeOwned,
        R: io::Read,
{
    let next_obj = Deserializer::from_reader(reader).into_iter::<T>().next();
    match next_obj {
        Some(result) => result.map_err(Into::into),
        None => Err(serde::de::Error::custom("premature EOF")),
    }
}

fn read_skipping_ws(mut reader: impl io::Read) -> Result<u8> {
    loop {
        let mut byte = 0u8;
        if let Err(io) = reader.read_exact(std::slice::from_mut(&mut byte)) {
            return Err(Error::io(io))
        }
        if !byte.is_ascii_whitespace() {
            return Ok(byte);
        }
    }
}
