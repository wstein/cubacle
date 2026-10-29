//! Bounded JSON Lines and duplicate-aware JSON parsing for the fixed-frame wire contract.

use std::fmt;
use std::io::BufRead;

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};

const MAX_LINE_BYTES: usize = 16 * 1024 * 1024;
const MAX_DEPTH: usize = 64;
const MAX_NUMBER_BYTES: usize = 128;

/// Read one physical request line without letting the input grow an unbounded buffer.
/// On an oversized line, drain to LF so the default batch mode can continue safely.
pub(super) fn read_line<R: BufRead>(input: &mut R) -> Result<Option<Value>, String> {
    read_bounded_line(input, MAX_LINE_BYTES)
}

fn read_bounded_line<R: BufRead>(input: &mut R, limit: usize) -> Result<Option<Value>, String> {
    let mut line = Vec::new();
    let mut too_long = false;
    let mut had_input = false;
    loop {
        let chunk = input.fill_buf().map_err(|error| error.to_string())?;
        if chunk.is_empty() {
            return if !had_input {
                Ok(None)
            } else {
                Err("request line must end with LF".to_owned())
            };
        }
        had_input = true;
        let end = chunk.iter().position(|byte| *byte == b'\n');
        let count = end.unwrap_or(chunk.len());
        if !too_long {
            if count > limit.saturating_sub(line.len()) {
                too_long = true;
                line.clear();
            } else {
                line.extend_from_slice(&chunk[..count]);
            }
        }
        input.consume(count + usize::from(end.is_some()));
        if end.is_some() {
            break;
        }
    }
    if too_long {
        return Err(format!("request line exceeds {limit} bytes"));
    }
    if line.ends_with(b"\r") {
        return Err("request line must use LF, not CRLF".to_owned());
    }
    let value = parse(&line)?;
    if !value.is_object() {
        return Err("request line must be a JSON object".to_owned());
    }
    Ok(Some(value))
}

/// Parse without accepting duplicate keys, non-integer number tokens or trailing values.
fn parse(bytes: &[u8]) -> Result<Value, String> {
    check_number_tokens(bytes)?;
    let mut reader = serde_json::Deserializer::from_slice(bytes);
    let value = Seed { depth: 0 }
        .deserialize(&mut reader)
        .map_err(|error| error.to_string())?;
    reader.end().map_err(|error| error.to_string())?;
    Ok(value)
}

/// Bound tokens before serde can parse a long number. Strings and escaped quotes are skipped.
fn check_number_tokens(bytes: &[u8]) -> Result<(), String> {
    let mut index = 0;
    let mut in_string = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' if in_string => index += 2,
            b'"' => {
                in_string = !in_string;
                index += 1;
            }
            b'-' | b'0'..=b'9' if !in_string => {
                let start = index;
                index += 1;
                while index < bytes.len()
                    && !matches!(
                        bytes[index],
                        b' ' | b'\t' | b'\r' | b'\n' | b',' | b']' | b'}' | b':'
                    )
                {
                    index += 1;
                }
                let token = &bytes[start..index];
                if token.len() > MAX_NUMBER_BYTES {
                    return Err("numeric token exceeds 128 bytes".to_owned());
                }
                if token.iter().any(|byte| matches!(byte, b'.' | b'e' | b'E')) {
                    return Err("fractional and exponent number tokens are not integers".to_owned());
                }
            }
            _ => index += 1,
        }
    }
    Ok(())
}

struct Seed {
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for Seed {
    type Value = Value;

    fn deserialize<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Seed {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("strict fixed-frame JSON")
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(Value::from(value))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::from(value))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        // serde classifies lexical -0 as f64. Fractions/exponents were rejected above.
        if value == 0.0 {
            Ok(Value::from(0))
        } else {
            Err(E::custom(
                "integer is outside the signed/unsigned 64-bit range",
            ))
        }
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Value, A::Error> {
        if self.depth >= MAX_DEPTH {
            return Err(de::Error::custom("JSON nesting exceeds 64 containers"));
        }
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(Seed {
            depth: self.depth + 1,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut object: A) -> Result<Value, A::Error> {
        if self.depth >= MAX_DEPTH {
            return Err(de::Error::custom("JSON nesting exceeds 64 containers"));
        }
        let mut values = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate object key {key:?}")));
            }
            let value = object.next_value_seed(Seed {
                depth: self.depth + 1,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

#[cfg(test)]
mod tests;
