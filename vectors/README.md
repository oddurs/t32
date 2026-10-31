# Conformance vectors

[`t32-vectors.json`](t32-vectors.json) holds about 600 test vectors for T32 draft 0.2. They are normative: an implementation conforms to a class of the specification when it reproduces every vector for that class.

| Section | Class | Each vector |
| --- | --- | --- |
| `parse` | Parser | `input` → `stamp` (canonical) or `error` |
| `encode_utc` | Encoder, converter | `utc` and `length` → `stamp` or `error` |
| `encode_unix` | Encoder, converter | `unix_nanos` and `length` → `stamp` or `error` |
| `utc_errors` | Converter | `utc` → `error` |
| `decode` | Decoder, converter | `stamp` → `start_units`, `start_utc`, `start_inserted`, `end_utc`, `end_inserted` |
| `durations` | Parser | `units` ↔ `text`, `nanos` → `units`, `text` → `canonical` or `error` |

The file's `conventions` field explains the units, the text forms and the error codes. Large integers are decimal strings.

The vectors are generated, not written by hand:

```sh
python3 tools/oracle.py --write vectors/t32-vectors.json   # regenerate
python3 tools/oracle.py --check vectors/t32-vectors.json   # verify (CI does this)
```

The generator is an exact rational-arithmetic oracle written directly from the specification. It shares no code with the implementations, so a vector that an implementation fails points to a defect in one of the three. Reports are welcome either way.

To use the vectors in your own implementation, load the file from this repository at a fixed commit, or vendor a copy with a note of the commit it came from.
