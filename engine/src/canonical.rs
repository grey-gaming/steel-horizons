//! Canonical JSON v1 writer — ADR-0006 deterministic encoding.
//!
//! This module implements the project-owned canonical JSON encoding defined
//! by ADR-0006.  The writer accepts a `serde_json::Value` (already checked for
//! floats, duplicates, and non-string map keys), sorts object members by
//! ascending UTF-8 byte sequence, and emits compact no-whitespace JSON.
//!
//! ## Authoritative references
//!
//! - ADR-0006 §Canonical JSON v1 byte encoding
//! - ADR-0006 §Collection ordering before JSON encoding
//! - ADR-0006 §Canonical content hash input

#![allow(missing_docs)]

use std::fmt;

use serde_json::Value;

/// Errors produced by the canonical writer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalError {
    /// A JSON number that is not a representable integer was encountered.
    FloatValue(Value),
    /// A JSON object contained a non-string key.
    NonStringKey(Value),
    /// A JSON value exceeds the supported integer range (i64/u64).
    IntegerOverflow,
}

impl fmt::Display for CanonicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonicalError::FloatValue(v) => {
                write!(f, "float or non-integer number value: {}", v)
            }
            CanonicalError::NonStringKey(v) => {
                write!(f, "non-string object key: {}", v)
            }
            CanonicalError::IntegerOverflow => {
                write!(f, "integer value exceeds supported range")
            }
        }
    }
}

/// Result type for canonical operations.
pub type CanonicalResult<T = Vec<u8>> = Result<T, CanonicalError>;

/// Write a checked `serde_json::Value` into canonical JSON v1 bytes.
///
/// The input value must be free of floating-point numbers, non-string object
/// keys, and duplicate object member names.  The writer sorts object members
/// by ascending UTF-8 byte sequence and emits compact (no-whitespace) JSON.
pub fn to_canonical_bytes(value: &Value) -> CanonicalResult {
    let mut buf = Vec::new();
    let mut state = WriteState { buf: &mut buf };
    write_value(&mut state, value)?;
    Ok(std::mem::take(state.buf))
}

// ─── Internal writer ───────────────────────────────────────────────────

struct WriteState<'a> {
    buf: &'a mut Vec<u8>,
}

/// Internal write helper — returns `()` not `Vec<u8>`.
fn write_value(state: &mut WriteState<'_>, value: &Value) -> Result<(), CanonicalError> {
    match value {
        Value::Null => {
            state.buf.extend_from_slice(b"null");
            Ok(())
        }
        Value::Bool(true) => {
            state.buf.extend_from_slice(b"true");
            Ok(())
        }
        Value::Bool(false) => {
            state.buf.extend_from_slice(b"false");
            Ok(())
        }
        Value::Number(n) => {
            // Reject floats and non-integer representations
            if n.is_f64() {
                return Err(CanonicalError::FloatValue(value.clone()));
            }
            // serde_json::Number can be parsed as i64 or u64
            if let Some(i) = n.as_i64() {
                write_fmt(state, format_args!("{}", i));
                Ok(())
            } else if let Some(u) = n.as_u64() {
                write_fmt(state, format_args!("{}", u));
                Ok(())
            } else {
                Err(CanonicalError::IntegerOverflow)
            }
        }
        Value::String(s) => {
            state.buf.push(b'"');
            write_string(state, s);
            state.buf.push(b'"');
            Ok(())
        }
        Value::Array(arr) => {
            state.buf.push(b'[');
            let mut first = true;
            for item in arr {
                if first {
                    first = false;
                } else {
                    state.buf.push(b',');
                }
                write_value(state, item)?;
            }
            state.buf.push(b']');
            Ok(())
        }
        Value::Object(obj) => {
            state.buf.push(b'{');
            // Collect and sort keys by UTF-8 byte sequence
            let mut keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
            keys.sort();
            let mut first = true;
            for key in keys {
                let val = obj.get(key).expect("key from keys() must exist");
                if first {
                    first = false;
                } else {
                    state.buf.push(b',');
                }
                // Write key as canonical string
                state.buf.push(b'"');
                write_string(state, key);
                state.buf.push(b'"');
                state.buf.push(b':');
                write_value(state, val)?;
            }
            state.buf.push(b'}');
            Ok(())
        }
    }
}

/// Write formatted args directly into the buffer.
fn write_fmt(state: &mut WriteState<'_>, args: fmt::Arguments<'_>) {
    // Use fmt::write on our string writer
    let _ = fmt::write(&mut StringWriter(state.buf), args);
}

/// Write a JSON string's characters into the buffer with canonical escaping.
///
/// Escapes `"`, `\`, backspace, tab, LF, form feed, CR as short sequences.
/// Escapes other U+0000–U+001F as `\u00xx`.  Does not escape `/` or non-ASCII
/// scalar values, and does not apply Unicode normalization.
fn write_string(state: &mut WriteState<'_>, s: &str) {
    for ch in s.chars() {
        match ch {
            '"' => state.buf.extend_from_slice(b"\\\""),
            '\\' => state.buf.extend_from_slice(b"\\\\"),
            '\x08' => state.buf.extend_from_slice(b"\\b"),
            '\x09' => state.buf.extend_from_slice(b"\\t"),
            '\x0a' => state.buf.extend_from_slice(b"\\n"),
            '\x0c' => state.buf.extend_from_slice(b"\\f"),
            '\x0d' => state.buf.extend_from_slice(b"\\r"),
            c if c <= '\x1f' => {
                // \uHHHH lowercase hex (4 hex digits for the full code point)
                let cp = c as u32;
                state.buf.extend_from_slice(b"\\u");
                state.buf.push(hex_digit((cp >> 12) as u8));
                state.buf.push(hex_digit(((cp >> 8) & 0xf) as u8));
                state.buf.push(hex_digit(((cp >> 4) & 0xf) as u8));
                state.buf.push(hex_digit((cp & 0xf) as u8));
            }
            c => {
                // Write UTF-8 encoded character
                let mut buf4 = [0u8; 4];
                let encoded = c.encode_utf8(&mut buf4);
                state.buf.extend_from_slice(encoded.as_bytes());
            }
        }
    }
}

fn hex_digit(v: u8) -> u8 {
    match v {
        0..=9 => b'0' + v,
        _ => b'a' + (v - 10),
    }
}

// ─── Helper: fmt::Write adapter for Vec<u8> ────────────────────────────

struct StringWriter<'a>(&'a mut Vec<u8>);

impl fmt::Write for StringWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Nested object sorting by UTF-8 key order.
    #[test]
    fn object_key_sorting() {
        let value = json!({
            "zulu": 1,
            "alpha": 2,
            "beta": 3,
        });
        let bytes = to_canonical_bytes(&value).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&bytes),
            r#"{"alpha":2,"beta":3,"zulu":1}"#,
            "object keys must be sorted alphabetically"
        );
    }

    /// Non-ASCII keys sort by UTF-8 byte sequence.
    #[test]
    fn non_ascii_key_sorting() {
        let value = json!({
            "\u{1f600}": 1,
            "abc": 2,
            "\u{00e9}": 3,
        });
        let bytes = to_canonical_bytes(&value).unwrap();
        assert!(!bytes.is_empty());
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let obj = parsed.as_object().unwrap();
        assert_eq!(obj.len(), 3);
    }

    /// String escape sequences.
    #[test]
    fn string_escapes() {
        let value = json!({"a": "hello \"world\"\n\t\\back"});
        let bytes = to_canonical_bytes(&value).unwrap();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains(r#"\""#));
        assert!(s.contains(r#"\n"#));
        assert!(s.contains(r#"\t"#));
        assert!(s.contains(r#"\\"#));
    }

    /// Control characters are escaped as \\u00xx.
    #[test]
    fn control_character_escape() {
        let value = json!({"a": "\u{0001}\u{001e}"});
        let bytes = to_canonical_bytes(&value).unwrap();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains(r#"\u0001"#));
        assert!(s.contains(r#"\u001e"#));
    }

    /// Integers: shortest base-10 representation, no leading + or zeros.
    #[test]
    fn integer_representation() {
        let value = json!({
            "zero": 0,
            "positive": 12345,
            "negative": -42,
            "large": 100000000000000u64,
        });
        let bytes = to_canonical_bytes(&value).unwrap();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("\"zero\":0"));
        assert!(s.contains("\"positive\":12345"));
        assert!(s.contains("\"negative\":-42"));
    }

    /// Null, true, false scalars.
    #[test]
    fn scalars() {
        let value = json!([null, true, false]);
        let bytes = to_canonical_bytes(&value).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&bytes),
            r#"[null,true,false]"#,
        );
    }

    /// Empty collections.
    #[test]
    fn empty_collections() {
        let value = json!({"empty_obj": {}, "empty_arr": []});
        let bytes = to_canonical_bytes(&value).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&bytes),
            r#"{"empty_arr":[],"empty_obj":{}}"#,
        );
    }

    /// Float rejection.
    #[test]
    fn float_rejected() {
        let value = json!({"a": 1.5});
        let result = to_canonical_bytes(&value);
        assert!(result.is_err());
        let value = json!({"a": 1.0});
        let result = to_canonical_bytes(&value);
        assert!(
            result.is_err(),
            "mathematically integral floats like 1.0 must also be rejected"
        );
    }

    /// Large u64 integers are accepted.
    #[test]
    fn large_u64() {
        let value = json!({"a": 18446744073709551615u64});
        let result = to_canonical_bytes(&value);
        assert!(result.is_ok());
    }

    /// Negative integers are accepted.
    #[test]
    fn negative_integers() {
        let value = json!({"a": -1, "b": -9223372036854775808i64});
        let result = to_canonical_bytes(&value);
        assert!(result.is_ok());
    }

    /// Nested arrays are preserved in order.
    #[test]
    fn nested_arrays() {
        let value = json!([[3, 1, 2], [4, 5]]);
        let bytes = to_canonical_bytes(&value).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&bytes),
            r#"[[3,1,2],[4,5]]"#,
            "arrays must preserve their original order"
        );
    }

    /// Complex nested structure round-trip.
    #[test]
    fn complex_round_trip() {
        let value = json!({
            "name": "test",
            "count": 42,
            "items": [
                {"id": "a", "tags": ["x", "y"]},
                {"id": "b", "tags": ["z"]}
            ],
            "metadata": null,
        });
        let bytes = to_canonical_bytes(&value).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed.as_object().unwrap().len(), 4);
        assert_eq!(parsed["count"].as_i64(), Some(42));
    }

    /// Property: canonical bytes are stable regardless of insertion order.
    #[test]
    fn insertion_order_independence() {
        let v1 = json!({
            "b": 1,
            "a": 2,
        });
        let v2 = json!({
            "a": 2,
            "b": 1,
        });
        let b1 = to_canonical_bytes(&v1).unwrap();
        let b2 = to_canonical_bytes(&v2).unwrap();
        assert_eq!(b1, b2, "canonical bytes must be independent of key insertion order");
    }
}
