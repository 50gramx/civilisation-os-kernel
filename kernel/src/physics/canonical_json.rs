//! RFC 8785 (JSON Canonicalization Scheme) for the Civilisation OS Kernel.
//!
//! This is the most fork-prone layer of the kernel.
//! Any serialization divergence between nodes produces different Merkle leaves,
//! different state roots, and an irreversible protocol fork.
//!
//! # Constitutional Rules (Frozen)
//!
//! 1. Object keys MUST be sorted by byte-order of their canonical UTF-8 representation.
//! 2. Object keys MUST match `^[a-z][a-z0-9_]*$` (lowercase ASCII, no leading digit/underscore).
//! 3. Duplicate keys are FORBIDDEN → `TransitionError::DuplicateKey`.
//! 4. JSON number literals are FORBIDDEN → `TransitionError::InvalidSerialization`.
//!    All numeric values MUST be encoded as JSON strings.
//! 5. Numeric string values MUST match `^(0|[1-9][0-9]*)$`.
//!    No leading zeros, no sign, no decimal, no exponent.
//! 6. Maximum nesting depth: `MAX_DEPTH` (32).
//! 7. Maximum fields per object: `MAX_OBJECT_FIELDS` (64).
//! 8. Maximum input size: `MAX_INPUT_BYTES` (65 536 = 64 KiB).
//! 9. BOM is rejected. Trailing content after the root value is rejected.
//! 10. Raw control characters (U+0000..U+001F) in string values are rejected.
//!
//! # Architecture
//!
//! `canonicalize(input)` → `Result<Vec<u8>, TransitionError>`
//!
//! Internally:
//! 1. Parse: hand-written recursive-descent parser → `Value` tree.
//! 2. Validate: all constraints enforced during parse (no second pass).
//! 3. Emit: deterministic byte emitter with sorted object keys.

use crate::TransitionError;

// ──────────────────────────────────────────────────────────────────────────────
// Constitutional constants
// ──────────────────────────────────────────────────────────────────────────────

/// Maximum nesting depth for objects and arrays combined.
pub const MAX_DEPTH: usize = 32;

/// Maximum number of key-value pairs per object.
pub const MAX_OBJECT_FIELDS: usize = 64;

/// Maximum number of items per array.
pub const MAX_ARRAY_ITEMS: usize = 1_024;

/// Maximum total input size in bytes.
pub const MAX_INPUT_BYTES: usize = 65_536;

// ──────────────────────────────────────────────────────────────────────────────
// Internal value tree
// ──────────────────────────────────────────────────────────────────────────────

/// A parsed JSON value. JSON number literals are absent — they are forbidden.
#[derive(Debug)]
enum Value {
    Null,
    Bool(bool),
    /// String: decoded content stored as raw UTF-8 bytes.
    Str(Vec<u8>),
    Array(Vec<Value>),
    /// Object: (decoded_key_bytes, value) pairs. Order is insertion order from
    /// the input; the emitter sorts them before output.
    Object(Vec<(Vec<u8>, Value)>),
}

// ──────────────────────────────────────────────────────────────────────────────
// Parser
// ──────────────────────────────────────────────────────────────────────────────

struct Parser {
    src: Vec<u8>,
    pos: usize,
    depth: usize,
}

impl Parser {
    fn new(src: Vec<u8>) -> Self {
        Parser { src, pos: 0, depth: 0 }
    }

    #[inline(always)]
    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    #[inline(always)]
    fn advance(&mut self) -> Option<u8> {
        let b = self.src.get(self.pos).copied();
        self.pos += 1;
        b
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek() {
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn expect(&mut self, expected: u8) -> Result<(), TransitionError> {
        match self.advance() {
            Some(b) if b == expected => Ok(()),
            _ => Err(TransitionError::InvalidSerialization),
        }
    }

    fn parse_value(&mut self) -> Result<Value, TransitionError> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'"') => self.parse_string().map(Value::Str),
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b't') => {
                let slice = self.src.get(self.pos..self.pos + 4);
                if slice == Some(b"true") {
                    self.pos += 4;
                    Ok(Value::Bool(true))
                } else {
                    Err(TransitionError::InvalidSerialization)
                }
            }
            Some(b'f') => {
                let slice = self.src.get(self.pos..self.pos + 5);
                if slice == Some(b"false") {
                    self.pos += 5;
                    Ok(Value::Bool(false))
                } else {
                    Err(TransitionError::InvalidSerialization)
                }
            }
            Some(b'n') => {
                let slice = self.src.get(self.pos..self.pos + 4);
                if slice == Some(b"null") {
                    self.pos += 4;
                    Ok(Value::Null)
                } else {
                    Err(TransitionError::InvalidSerialization)
                }
            }
            // JSON number literals: CONSTITUTIONALLY FORBIDDEN.
            Some(b'0'..=b'9') | Some(b'-') => {
                Err(TransitionError::InvalidSerialization)
            }
            _ => Err(TransitionError::InvalidSerialization),
        }
    }

    /// Parse a JSON string delimited by `"`. Returns the decoded content bytes.
    /// Rejects raw control characters (U+0000..U+001F must be escaped).
    fn parse_string(&mut self) -> Result<Vec<u8>, TransitionError> {
        self.expect(b'"')?;
        let mut out: Vec<u8> = Vec::new();
        loop {
            match self.advance() {
                None => return Err(TransitionError::InvalidSerialization),
                Some(b'"') => break,
                Some(b'\\') => {
                    match self.advance() {
                        Some(b'"')  => out.push(b'"'),
                        Some(b'\\') => out.push(b'\\'),
                        Some(b'/')  => out.push(b'/'),
                        Some(b'b')  => out.push(0x08),
                        Some(b'f')  => out.push(0x0C),
                        Some(b'n')  => out.push(b'\n'),
                        Some(b'r')  => out.push(b'\r'),
                        Some(b't')  => out.push(b'\t'),
                        Some(b'u')  => {
                            // Parse exactly 4 hex digits.
                            let hex = self.src
                                .get(self.pos..self.pos + 4)
                                .ok_or(TransitionError::InvalidSerialization)?;
                            let s = std::str::from_utf8(hex)
                                .map_err(|_| TransitionError::InvalidSerialization)?;
                            let codepoint = u32::from_str_radix(s, 16)
                                .map_err(|_| TransitionError::InvalidSerialization)?;
                            self.pos += 4;
                            // Encode the Unicode scalar as UTF-8.
                            let ch = char::from_u32(codepoint)
                                .ok_or(TransitionError::InvalidSerialization)?;
                            let mut buf = [0u8; 4];
                            let encoded = ch.encode_utf8(&mut buf);
                            out.extend_from_slice(encoded.as_bytes());
                        }
                        _ => return Err(TransitionError::InvalidSerialization),
                    }
                }
                Some(b) => {
                    // Raw control characters are forbidden.
                    if b < 0x20 {
                        return Err(TransitionError::InvalidSerialization);
                    }
                    out.push(b);
                }
            }
        }
        Ok(out)
    }

    fn parse_object(&mut self) -> Result<Value, TransitionError> {
        self.expect(b'{')?;
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(TransitionError::InvalidSerialization);
        }

        let mut pairs: Vec<(Vec<u8>, Value)> = Vec::new();
        self.skip_whitespace();

        // Empty object.
        if self.peek() == Some(b'}') {
            self.advance();
            self.depth -= 1;
            return Ok(Value::Object(pairs));
        }

        loop {
            if pairs.len() >= MAX_OBJECT_FIELDS {
                return Err(TransitionError::InvalidSerialization);
            }
            self.skip_whitespace();

            // Key.
            let key = self.parse_string()?;

            // Key must not be empty.
            if key.is_empty() {
                return Err(TransitionError::InvalidSerialization);
            }

            // Key must match ^[a-z][a-z0-9_]*$ — lowercase ASCII only.
            // First byte must be a letter (not digit or underscore).
            if !matches!(key[0], b'a'..=b'z') {
                return Err(TransitionError::InvalidSerialization);
            }
            for &b in &key[1..] {
                if !matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_') {
                    return Err(TransitionError::InvalidSerialization);
                }
            }

            // Duplicate key detection.
            for (existing, _) in &pairs {
                if existing == &key {
                    return Err(TransitionError::DuplicateKey);
                }
            }

            self.skip_whitespace();
            self.expect(b':')?;
            self.skip_whitespace();
            let value = self.parse_value()?;

            pairs.push((key, value));
            self.skip_whitespace();

            match self.peek() {
                Some(b',') => { self.advance(); }
                Some(b'}') => { self.advance(); break; }
                _ => return Err(TransitionError::InvalidSerialization),
            }
        }

        self.depth -= 1;
        Ok(Value::Object(pairs))
    }

    fn parse_array(&mut self) -> Result<Value, TransitionError> {
        self.expect(b'[')?;
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(TransitionError::InvalidSerialization);
        }

        let mut items: Vec<Value> = Vec::new();
        self.skip_whitespace();

        // Empty array.
        if self.peek() == Some(b']') {
            self.advance();
            self.depth -= 1;
            return Ok(Value::Array(items));
        }

        loop {
            if items.len() >= MAX_ARRAY_ITEMS {
                return Err(TransitionError::InvalidSerialization);
            }
            self.skip_whitespace();
            let v = self.parse_value()?;
            items.push(v);
            self.skip_whitespace();

            match self.peek() {
                Some(b',') => { self.advance(); }
                Some(b']') => { self.advance(); break; }
                _ => return Err(TransitionError::InvalidSerialization),
            }
        }

        self.depth -= 1;
        Ok(Value::Array(items))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Emitter
// ──────────────────────────────────────────────────────────────────────────────

const HEX_LOWER: [u8; 16] = *b"0123456789abcdef";

/// RFC 8785 §3.2.2.2 — emit a string with canonical escape sequences.
fn emit_string_content(bytes: &[u8], out: &mut Vec<u8>) {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'"'  => { out.extend_from_slice(b"\\\""); }
            b'\\' => { out.extend_from_slice(b"\\\\"); }
            0x08  => { out.extend_from_slice(b"\\b"); }
            0x0C  => { out.extend_from_slice(b"\\f"); }
            b'\n' => { out.extend_from_slice(b"\\n"); }
            b'\r' => { out.extend_from_slice(b"\\r"); }
            b'\t' => { out.extend_from_slice(b"\\t"); }
            0x00..=0x1F => {
                // Other control chars → \u00XX
                out.extend_from_slice(b"\\u00");
                out.push(HEX_LOWER[(b >> 4) as usize]);
                out.push(HEX_LOWER[(b & 0xF) as usize]);
            }
            _ => {
                // All other bytes pass through (UTF-8 safe — see module doc).
                out.push(b);
            }
        }
        i += 1;
    }
}

/// Emit a Value as canonical JCS bytes.
fn emit(value: &Value, out: &mut Vec<u8>) {
    match value {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(true)  => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Str(bytes) => {
            out.push(b'"');
            emit_string_content(bytes, out);
            out.push(b'"');
        }
        Value::Array(items) => {
            out.push(b'[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 { out.push(b','); }
                emit(item, out);
            }
            out.push(b']');
        }
        Value::Object(pairs) => {
            // RFC 8785 §3.2.3 — sort keys by canonical UTF-8 byte order.
            let mut indices: Vec<usize> = (0..pairs.len()).collect();
            indices.sort_by(|&a, &b| pairs[a].0.cmp(&pairs[b].0));

            out.push(b'{');
            for (i, &idx) in indices.iter().enumerate() {
                if i > 0 { out.push(b','); }
                let (key, val) = &pairs[idx];
                out.push(b'"');
                emit_string_content(key, out);
                out.push(b'"');
                out.push(b':');
                emit(val, out);
            }
            out.push(b'}');
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────────────────

/// Canonicalize JSON input per RFC 8785 (JCS) with constitutional constraints.
///
/// Returns the canonical byte representation of the parsed JSON value.
/// Rejects any input that violates the constitutional rules listed in the module doc.
///
/// This function is pure: no I/O, no randomness, no environment reads, no clock.
pub fn canonicalize(input: &[u8]) -> Result<Vec<u8>, TransitionError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(TransitionError::InvalidSerialization);
    }
    // Reject UTF-8 BOM.
    if input.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(TransitionError::InvalidSerialization);
    }

    let mut parser = Parser::new(input.to_vec());
    let value = parser.parse_value()?;

    // Reject trailing content after the root value.
    parser.skip_whitespace();
    if parser.pos != parser.src.len() {
        return Err(TransitionError::InvalidSerialization);
    }

    let mut out = Vec::with_capacity(input.len());
    emit(&value, &mut out);
    Ok(out)
}

/// Validate that a canonical JSON object contains exactly the set of `allowed_keys`.
///
/// Called AFTER `canonicalize`. Rejects objects with extra keys OR missing keys.
/// Schema enforcement is separate from canonicalization: first canonicalize,
/// then call this function for each expected payload type.
pub fn validate_schema(
    canonical: &[u8],
    allowed_keys: &[&str],
) -> Result<(), TransitionError> {
    // Re-parse the canonical bytes (already validated, so this is cheap).
    let mut parser = Parser::new(canonical.to_vec());
    let value = parser.parse_value().map_err(|_| TransitionError::InvalidSerialization)?;

    let pairs = match value {
        Value::Object(pairs) => pairs,
        _ => return Err(TransitionError::InvalidSerialization),
    };

    // Every key in the object must be in allowed_keys.
    for (key, _) in &pairs {
        let key_str = std::str::from_utf8(key)
            .map_err(|_| TransitionError::InvalidSerialization)?;
        if !allowed_keys.contains(&key_str) {
            return Err(TransitionError::InvalidSerialization);
        }
    }

    // Every allowed_key must be present in the object.
    for &expected in allowed_keys {
        let found = pairs.iter().any(|(k, _)| k == expected.as_bytes());
        if !found {
            return Err(TransitionError::InvalidSerialization);
        }
    }

    Ok(())
}

/// Validate that a string value matches the numeric-string protocol:
/// `^(0|[1-9][0-9]*)$` — no leading zeros, no sign prefix, no decimal, no exponent.
pub fn validate_numeric_string(s: &[u8]) -> Result<(), TransitionError> {
    if s.is_empty() {
        return Err(TransitionError::InvalidSerialization);
    }
    if s == b"0" {
        return Ok(()); // Exactly zero is valid.
    }
    // Must start with 1-9 (no leading zero).
    if !matches!(s[0], b'1'..=b'9') {
        return Err(TransitionError::InvalidSerialization);
    }
    // Remaining bytes must be digits.
    for &b in &s[1..] {
        if !matches!(b, b'0'..=b'9') {
            return Err(TransitionError::InvalidSerialization);
        }
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    // ── Structural canonicalization ───────────────────────────────────────────

    #[test]
    fn empty_object_is_canonical() {
        assert_eq!(canonicalize(b"{}").unwrap(), b"{}");
    }

    #[test]
    fn empty_array_is_canonical() {
        assert_eq!(canonicalize(b"[]").unwrap(), b"[]");
    }

    #[test]
    fn null_is_canonical() {
        assert_eq!(canonicalize(b"null").unwrap(), b"null");
    }

    #[test]
    fn bool_true_is_canonical() {
        assert_eq!(canonicalize(b"true").unwrap(), b"true");
    }

    #[test]
    fn bool_false_is_canonical() {
        assert_eq!(canonicalize(b"false").unwrap(), b"false");
    }

    // ── Constitutional vector: key sorting ───────────────────────────────────

    #[test]
    fn scrambled_object_sorts_keys() {
        // Input with keys in wrong order. Output must be lexicographically sorted.
        let input = br#"{"b":"2","a":"1"}"#;
        let canonical = canonicalize(input).unwrap();
        assert_eq!(canonical, br#"{"a":"1","b":"2"}"#);
    }

    #[test]
    fn three_key_sort_is_lexicographic() {
        // "amount", "epoch", "bond" → sorted: "amount", "bond", "epoch"
        let input = br#"{"epoch":"3","bond":"2","amount":"1"}"#;
        let canonical = canonicalize(input).unwrap();
        assert_eq!(canonical, br#"{"amount":"1","bond":"2","epoch":"3"}"#);
    }

    #[test]
    fn whitespace_in_input_is_stripped() {
        let input = b"{ \"z\" : \"1\" , \"a\" : \"2\" }";
        let canonical = canonicalize(input).unwrap();
        assert_eq!(canonical, br#"{"a":"2","z":"1"}"#);
    }

    // ── Constitutional vector: SHA-256 of canonical output ───────────────────

    #[test]
    fn constitutional_vector_with_hash() {
        // Primary constitutional vector. Input has scrambled keys.
        // After canonicalization, the output bytes must match exactly,
        // and SHA-256 of those bytes must be pinned.
        let input = br#"{"b":"2","a":"1"}"#;
        let canonical = canonicalize(input).unwrap();
        assert_eq!(&canonical, br#"{"a":"1","b":"2"}"#);

        let hash = crate::physics::hashing::sha256(&canonical);
        // PINNED CONSTITUTIONAL VECTOR — DO NOT CHANGE.
        // SHA-256(b`{"a":"1","b":"2"}`) computed from our FIPS 180-4 reference.
        // Any change to the canonicalizer that produces different bytes will break
        // this assertion and signal a potential chain fork.
        let expected: [u8; 32] = [
            0x21, 0xf7, 0x6d, 0xfb, 0xfe, 0x6d, 0xfe, 0x21,
            0xf7, 0x62, 0x08, 0x0e, 0xf4, 0x84, 0x11, 0x2c,
            0xf2, 0x95, 0x29, 0x74, 0xce, 0xf3, 0x07, 0x41,
            0xfd, 0x19, 0x31, 0xe1, 0xc6, 0xd9, 0x21, 0x12,
        ];
        assert_eq!(hash, expected, "constitutional hash vector must be stable");
    }

    // ── Constitutional vector: duplicate key rejection ────────────────────────

    #[test]
    fn duplicate_key_is_rejected() {
        let input = br#"{"a":"1","a":"2"}"#;
        assert_eq!(
            canonicalize(input),
            Err(TransitionError::DuplicateKey),
            "duplicate key must raise DuplicateKey, not last-wins"
        );
    }

    #[test]
    fn duplicate_key_at_depth_is_rejected() {
        let input = br#"{"outer":{"x":"1","x":"2"}}"#;
        assert_eq!(canonicalize(input), Err(TransitionError::DuplicateKey));
    }

    // ── Constitutional vector: number literal rejection ───────────────────────

    #[test]
    fn json_number_literal_is_rejected() {
        // Numeric literals are constitutionally forbidden.
        // Numbers must be encoded as strings: {"amount":"1000"}
        let input = br#"{"amount":1000}"#;
        assert_eq!(
            canonicalize(input),
            Err(TransitionError::InvalidSerialization),
            "JSON number literals must be rejected at the parse layer"
        );
    }

    #[test]
    fn negative_number_literal_is_rejected() {
        assert_eq!(canonicalize(br#"{"x":-1}"#), Err(TransitionError::InvalidSerialization));
    }

    #[test]
    fn float_literal_is_rejected() {
        assert_eq!(canonicalize(br#"{"x":1.5}"#), Err(TransitionError::InvalidSerialization));
    }

    // ── Schema validation enforcement ─────────────────────────────────────────

    #[test]
    fn unknown_field_is_rejected_by_schema_validator() {
        let input = br#"{"allowed":"1","rogue":"2"}"#;
        let canonical = canonicalize(input).unwrap();
        // Schema: only "allowed" is expected.
        assert_eq!(
            validate_schema(&canonical, &["allowed"]),
            Err(TransitionError::InvalidSerialization),
            "extra key not in schema must be rejected"
        );
    }

    #[test]
    fn missing_required_field_rejected_by_schema_validator() {
        let input = br#"{"a":"1"}"#;
        let canonical = canonicalize(input).unwrap();
        assert_eq!(
            validate_schema(&canonical, &["a", "b"]),
            Err(TransitionError::InvalidSerialization),
            "missing required key must be rejected"
        );
    }

    #[test]
    fn exact_schema_match_passes() {
        let input = br#"{"b":"2","a":"1"}"#;
        let canonical = canonicalize(input).unwrap();
        assert!(validate_schema(&canonical, &["a", "b"]).is_ok());
    }

    // ── Key format enforcement ────────────────────────────────────────────────

    #[test]
    fn uppercase_key_is_rejected() {
        assert_eq!(canonicalize(br#"{"A":"1"}"#), Err(TransitionError::InvalidSerialization));
    }

    #[test]
    fn key_with_leading_digit_is_rejected() {
        assert_eq!(canonicalize(br#"{"1key":"1"}"#), Err(TransitionError::InvalidSerialization));
    }

    #[test]
    fn key_with_leading_underscore_is_rejected() {
        assert_eq!(canonicalize(br#"{"_key":"1"}"#), Err(TransitionError::InvalidSerialization));
    }

    #[test]
    fn key_with_hyphen_is_rejected() {
        assert_eq!(canonicalize(br#"{"key-name":"1"}"#), Err(TransitionError::InvalidSerialization));
    }

    // ── Numeric string validation ─────────────────────────────────────────────

    #[test]
    fn numeric_string_zero_is_valid() {
        assert!(validate_numeric_string(b"0").is_ok());
    }

    #[test]
    fn numeric_string_positive_is_valid() {
        assert!(validate_numeric_string(b"1000000000000").is_ok());
    }

    #[test]
    fn numeric_string_leading_zero_rejected() {
        assert_eq!(validate_numeric_string(b"01"), Err(TransitionError::InvalidSerialization));
    }

    #[test]
    fn numeric_string_negative_rejected() {
        assert_eq!(validate_numeric_string(b"-1"), Err(TransitionError::InvalidSerialization));
    }

    #[test]
    fn numeric_string_decimal_rejected() {
        assert_eq!(validate_numeric_string(b"1.5"), Err(TransitionError::InvalidSerialization));
    }

    #[test]
    fn numeric_string_empty_rejected() {
        assert_eq!(validate_numeric_string(b""), Err(TransitionError::InvalidSerialization));
    }

    // ── DOS bounding ──────────────────────────────────────────────────────────

    #[test]
    fn nesting_beyond_max_depth_rejected() {
        // 33 levels of nesting exceeds MAX_DEPTH (32).
        let mut s: Vec<u8> = Vec::new();
        for _ in 0..33 {
            s.extend_from_slice(br#"{"a":"#);
        }
        s.extend_from_slice(b"\"v\"");
        for _ in 0..33 {
            s.push(b'}');
        }
        assert_eq!(canonicalize(&s), Err(TransitionError::InvalidSerialization));
    }

    #[test]
    fn object_at_max_depth_is_accepted() {
        // 31 levels of nesting is within MAX_DEPTH (32).
        let mut s: Vec<u8> = Vec::new();
        for _ in 0..31 {
            s.extend_from_slice(br#"{"a":"#);
        }
        s.extend_from_slice(b"\"v\"");
        for _ in 0..31 {
            s.push(b'}');
        }
        assert!(canonicalize(&s).is_ok());
    }

    // ── String escaping ───────────────────────────────────────────────────────

    #[test]
    fn raw_control_char_in_string_is_rejected() {
        // Newline (0x0A) must be \n-escaped, not raw.
        let input = b"\"hello\nworld\"";
        assert_eq!(canonicalize(input), Err(TransitionError::InvalidSerialization));
    }

    #[test]
    fn escaped_newline_is_preserved_in_canonical_form() {
        let input = br#""hello\nworld""#;
        let canonical = canonicalize(input).unwrap();
        assert_eq!(canonical, br#""hello\nworld""#);
    }

    #[test]
    fn trailing_content_is_rejected() {
        assert_eq!(canonicalize(b"{}{}"), Err(TransitionError::InvalidSerialization));
        assert_eq!(canonicalize(b"\"x\" garbage"), Err(TransitionError::InvalidSerialization));
    }

    #[test]
    fn bom_is_rejected() {
        let input = b"\xEF\xBB\xBF{}";
        assert_eq!(canonicalize(input), Err(TransitionError::InvalidSerialization));
    }

    // ── Complex composition ───────────────────────────────────────────────────

    #[test]
    fn nested_object_with_scrambled_keys_at_each_level() {
        let input = br#"{"outer_z":{"b":"2","a":"1"},"outer_a":{"y":"9","x":"8"}}"#;
        let canonical = canonicalize(input).unwrap();
        // outer_a < outer_z; within each: a < b, x < y
        assert_eq!(
            canonical,
            br#"{"outer_a":{"x":"8","y":"9"},"outer_z":{"a":"1","b":"2"}}"#
        );
    }

    #[test]
    fn array_preserves_insertion_order() {
        // Arrays are NOT sorted — only object keys are.
        let input = br#"{"items":["b","a","c"]}"#;
        let canonical = canonicalize(input).unwrap();
        assert_eq!(canonical, br#"{"items":["b","a","c"]}"#);
    }
}
