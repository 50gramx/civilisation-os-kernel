# Canonical JSON Serialization Specification (The Bytes)

Before we can compute a State Root, we must hash payloads. Before we can hash a payload, we must serialize it. If the serialization produces even a single misplaced space or a reordered key across different browser engines (V8, JavaScriptCore, SpiderMonkey), the hashes will diverge, the Merkle tree will fracture, and the Civilisation OS will fork.

We must freeze the bytes mathematically.

## The Standard: JCS (RFC 8785)
The Civilisation OS strictly implements the JSON Canonical Serialization (JCS) standard, as defined in RFC 8785. 

### Core Serialization Invariants:

#### 1. Formatting and Spacing
- **Zero Whitespace:** No spaces, tabs, or line breaks outside of explicitly defined string values. 
- Correct: `{"epoch":42,"type":"bond"}`
- Incorrect: `{"epoch": 42, "type": "bond"}`

#### 2. Key Ordering & Unicode Normalization
- **ASCII-Only Keys:** JSON keys MUST be explicitly restricted to ASCII characters matching the regex `^[a-z0-9_]+$`. This permanently eliminates homoglyph attacks, invisible character forks, and cross-engine surrogate pairing discrepancies. Multilingual strings are permitted exclusively in payload *values* (e.g., Markdown URIs, justification hashes).
- **Lexicographical Sort:** All keys within a JSON object MUST be ordered lexicographically.
- **Raw Code Units Only:** Sorting is performed over Unicode code points as defined by RFC 8785. Surrogate halves are NOT interpreted separately. 
- **NO Normalization:** Keys MUST NOT be normalized (e.g., no NFC or NFD transformations). Raw code units are compared byte-equivalent after UTF-8 decoding. Any locale-dependent comparison triggers a permanent hard fork.
- **Nested Objects:** The sorting is applied recursively to all nested objects.

#### 3. String Encoding
- **Encoding:** Strictly UTF-8 over the wire, manipulating UTF-16 code points during the JCS sort.
- **Escaping:** Control characters and specific symbols must be escaped according to RFC 8785 rules.

#### 4. The Absolute Numeric Ban
*This is the most notorious source of cross-platform nondeterminism. We ban the JSON `Number` type entirely for consensus-critical magnitudes.*
- **All magnitudes are Strings:** All economic and numeric values in the consensus payload MUST be serialized as strings.
  - Correct: `{"staked_weight": "1500"}`
  - Incorrect: `{"staked_weight": 1500}`
- **Absolute Float Prohibition:** No decimals are allowed, even within strings. All fractions are handled later by the kernel's FixedPoint engine.
  - Incorrect: `{"yield": "1.25"}` (Rejected at canonicalization)

- **Allowed Numeric String Grammar:** 
  To prevent normalization forks (e.g., treating `"001500"`, `"+1500"`, and `" 1500"` as identical before hashing), the string MUST strictly match the following regular expression for unsigned magnitudes:
  `^(0|[1-9][0-9]*)$`
  *   Meaning: Either exactly `"0"`, or no leading zeros, no plus signs, no whitespace, no negative signs.
  *   *(If signed magnitudes are required in future payloads, they must precisely match `^-?(0|[1-9][0-9]*)$`)*.

By forcing strict grammar strings for all numbers, we guarantee that no JavaScript engine, Python script, or C parser can auto-trim leading zeros, alter scientific notation (`1.5e3`), or introduce floating-point rounding quirks before the WASM kernel processes the payload.

#### 5. Strict Parsing Rejections
- **Duplicate Keys:** If a JSON object contains duplicate keys (e.g., `{"epoch":42, "epoch":43}`), the parser MUST throw a hard error. Rejection is mandatory.
- **Unknown Fields:** If a payload contains any key not explicitly defined in the Rust schema definition, the payload is orphaned.

#### 6. Cryptographic Hash Primitives (Frozen)
- **Algorithm:** SHA-256 exactly as defined in FIPS 180-4. 
- **Prohibitions:** No truncated variants. No SHA-3. No BLAKE2. Any change to the hashing algorithm is a hard fork.
- **Merkle Domain Separation:** To prevent second preimage attacks on the Merkle trees, leaf nodes and internal nodes MUST be domain-separated using a single-byte prefix before hashing:
  - `leaf_hash = SHA256(0x00 || serialized_leaf)`
  - `node_hash = SHA256(0x01 || left_hash || right_hash)`
- **Merkle Tree Shape:** The tree MUST be a **Perfect Binary Padded Tree**. 
  - **Leaf Ordering:** Leaves are ordered strictly by the JCS-serialized bytes of their content, ascending. 
  - **Empty Tree Rule:** If a tree contains zero leaves (e.g., zero approved bonds in an epoch), the root is explicitly defined as `SHA256(0x00 || empty_byte_string)`.
  - **Depth Rule:** If `leaf_count > 0`, the tree size is padded to the next power of two (`tree_size = next_power_of_two(leaf_count)`).
  - **Padding Rule (Odd-Node):** Padding is accomplished by duplicating the final valid node at that specific level to pair with itself (`node_hash = SHA256(0x01 || final_node || final_node)`), continuing this duplication upwards until the power-of-two level is filled. This guarantees the tree perfectly balances to a single root without zero-hash padding ambiguities.

## Implementation Enforcement
The Rust deterministic kernel will expose a pure function:
`pub fn canonicalize_and_hash(raw_json_bytes: &[u8]) -> Result<[u8; 32], CanonicalError>`

Any payload violating JCS, containing a bare JSON number for a magnitude, containing a decimal string, or possessing duplicate/unknown keys returns a `CanonicalError` and is instantly dropped.
