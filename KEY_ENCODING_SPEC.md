# Key Encoding Specification and Compatibility

This document describes the key encoding formats used in kulfi-utils and ensures backward compatibility.

## Encoding Formats

### Secret Keys
- **Storage Format**: Hexadecimal (lowercase), 64 characters
- **Display Implementation**: `data_encoding::HEXLOWER.encode(&bytes)`
- **Parsing**: Accepts both:
  - Hex format (64 chars) - primary format
  - BASE32_NOPAD format - for backward compatibility with iroh's alternative format

### Public Keys (ID52)
- **Storage Format**: BASE32_DNSSEC encoding, 52 characters
- **Display Implementation**: `data_encoding::BASE32_DNSSEC.encode(&bytes)`
- **Parsing**: BASE32_DNSSEC decoding

## Compatibility Guarantees

1. **Secret keys stored in hex format will always be readable**
   - Example: `bc90278b15788c779afd222e4bdd8f265eb97854670390a96ed93c934b4fe975`

2. **Public keys (ID52) maintain consistent encoding**
   - Example: `ci1k08umg4sc8eb6gg609h358obgc4sjmdgm8or68h1mas91c5j0`

3. **Round-trip guarantee**: 
   - `parse(encode(key)) == key` for all valid keys
   - `encode(parse(str)) == str` for all valid encoded strings

## Test Coverage

The `encoding_compatibility` test suite verifies:
- Secret key hex encoding/decoding
- Secret key roundtrip (generate → encode → decode → encode)
- Public key ID52 encoding/decoding
- Bytes representation compatibility
- Backward compatibility with existing keys

## Migration Notes

When updating kulfi-utils:
1. Run `cargo test -p kulfi-utils --test encoding_compatibility`
2. Add any new test vectors from production keys
3. Never change the encoding format without a migration plan

## Implementation Details

The encoding logic is centralized in `kulfi-utils/src/keys.rs`:
- `SecretKey` newtype wrapper handles all secret key encoding
- `PublicKey` newtype wrapper handles all public key encoding
- Direct access to underlying types is restricted to maintain consistency