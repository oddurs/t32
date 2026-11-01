# Implementations

Implementations of T32 known to the editor. An implementation conforms to a class when it passes every vector for that class in [`vectors/t32-vectors.json`](vectors/t32-vectors.json); the classes are defined in the specification under *Conventions and conformance*.

To add yours, open a pull request that adds a row, with a link to where it runs the vectors. Partial implementations are welcome: say which classes they cover.

| Implementation | Language | Classes | Vectors | Maintainer | Notes |
| --- | --- | --- | --- | --- | --- |
| [`t32`](https://crates.io/crates/t32) ([source](rust/)) | Rust | All | All pass from 0.3.0 | [@oddurs](https://github.com/oddurs) | Reference implementation. No required dependencies; optional serde. |
| [`t32.ts`](web/src/lib/t32.ts) | TypeScript | All | All pass | [@oddurs](https://github.com/oddurs) | One file, BigInt arithmetic. Runs the site's playground. |
| [`oracle.py`](tools/oracle.py) | Python | Encoder, decoder, converter, parser | Generates them | [@oddurs](https://github.com/oddurs) | Exact rational arithmetic, written from the text. A checking tool, not a library. |

## Wanted

Independent implementations are what take the specification to version 1.0 (see [GOVERNANCE.md](GOVERNANCE.md)). These would be especially welcome, each by an author other than the editor:

- **Go, Python, Java or Kotlin, C or C++, Swift, Zig.** A library with parsing, encoding and decoding.
- **PostgreSQL.** A type or a set of functions for T32-n columns, with range queries.
- **SQLite.** A loadable extension.
- **Command-line tools.** A converter in the spirit of `date`.

If you start one, say so in [Discussions](https://github.com/oddurs/t32/discussions), so that others can join rather than duplicate.
