# Conformance vectors

[`t32-vectors.json`](t32-vectors.json) holds 782 test vectors for T32 draft 0.3. They are normative: an implementation conforms to a class of the specification when it reproduces every vector for that class. Where a vector and the specification's text disagree, the text decides and the disagreement is an erratum.

| Section | Class | Each vector |
| --- | --- | --- |
| `parse` | Parser | `input` → `stamp` (canonical) or `error` |
| `encode_tai` | Encoder | `units` (biased time) and `length` → `stamp` or `error` |
| `encode_utc` | Encoder, converter | `utc` and `length` → `stamp` and `provisional`, or `error` |
| `encode_unix` | Encoder, converter | `unix_nanos` and `length` → `stamp` and `provisional`, or `error` |
| `utc_errors` | Converter | `utc` → `error` |
| `decode` | Decoder | `stamp` → `start_units` |
| `decode` | Decoder, converter | `stamp` → `start_utc`, `start_inserted`, `start_provisional`, and the same for `end_` (`end_utc` is null for the last stamps, which end past the range) |
| `binary` | Encoder | `stamp` → `bytes`: the value V, big-endian, in the fewest whole bytes |
| `upper_bound` | Parser | `prefix` → `bound`, the exclusive upper bound of its range, or null |
| `max_length` | Encoder | resolution in `units` → the longest permitted `length` |
| `durations` | Parser | `units` ↔ `text`; `nanos` → `units` or `error`; `text` → `canonical` or `error` |
| `add` | Decoder | `stamp` + `duration` → `result` at the stamp's length, or `error` |
| `difference` | Decoder | `b_minus_a`: the start of `b` minus the start of `a` |

The file's `conventions` field explains the units, the text forms, the error codes and their order. Large integers are decimal strings.

Every vector is generated, errors included; none is written by hand:

```sh
python3 tools/oracle.py --write vectors/t32-vectors.json   # regenerate
python3 tools/oracle.py --check vectors/t32-vectors.json   # verify (CI does this)
```

The generator is an exact rational-arithmetic oracle written directly from the specification, with its own parsers for stamps, durations and UTC text. It shares no code with the implementations, so a vector that an implementation fails points to a defect in one of the three. Reports are welcome either way.

To use the vectors in your own implementation, load the file from this repository at a fixed tag, such as `spec-draft-0.3`, or vendor a copy with a note of the tag it came from.
