# Changelog

## 0.13.0 (unreleased)

### General

- Literal byte strings (`b"..."`) may now be used in the `ipld!` macro.

### DagCbor

- **BREAKING:** `Vec<u8>`, `&[u8]`, etc. now encode & decode as a byte strings, not as a lists of bytes.
- **BREAKING:** `Vec<T>` encoding & decoding now requires `T: 'static`.

