# Contributing to T32

Thank you for taking an interest. T32 is a draft, so this is the time when a careful reading, a question or a second implementation changes the most.

You do not need permission to contribute, and no contribution is too small. A sentence that can be read two ways is worth an issue.

## Ways to help

**Read the specification critically.** Read [SPEC.md](SPEC.md), or the [rendered version](https://oddurs.github.io/t32/spec/), as if you had to implement it from the text alone. Report anything ambiguous, missing or wrong as an [issue](https://github.com/oddurs/t32/issues/new/choose).

**Weigh in on the open questions.** Appendix C of the specification lists the questions still open before version 1.0. Each states the current position and what would change it. Experience from your own systems is the most useful evidence.

**Write an implementation.** The format is small, and an independent implementation is the best test the specification can have. Load [`vectors/t32-vectors.json`](vectors/t32-vectors.json), make every vector pass, and add your implementation to [IMPLEMENTATIONS.md](IMPLEMENTATIONS.md) with a pull request. Implementations that disagree with the vectors are valuable too: either the implementation or the specification is wrong, and both are worth knowing.

**Propose a change.** Start a thread in [Discussions](https://github.com/oddurs/t32/discussions) under *Proposals*. Describe the problem before the solution. If the idea holds up, turn it into a pull request that changes the text, the vectors and both implementations together. [GOVERNANCE.md](GOVERNANCE.md) describes how proposals are decided.

**Improve the implementations or the site.** Bug reports and fixes for the Rust crate, the TypeScript port and the site are welcome through the usual issues and pull requests.

## The repository

| Path | What | Check with |
| --- | --- | --- |
| `SPEC.md` | The specification | Read it rendered: `cd web && npm run dev` |
| `vectors/t32-vectors.json` | The conformance vectors, generated | `python3 tools/oracle.py --check vectors/t32-vectors.json` |
| `tools/oracle.py` | The exact-arithmetic oracle that generates them | `python3 tools/oracle.py` |
| `rust/` | The reference implementation, the `t32` crate | `cargo test --all-features` |
| `web/src/lib/t32.ts` | The TypeScript implementation | `npm test` |
| `web/` | The site | `npm run check && npm run build` |

## Changing normative behaviour

A change to how stamps are encoded, decoded, parsed or converted touches four places, and a pull request should change all four together:

1. **The text** in `SPEC.md`, with an entry in Appendix D.
2. **The oracle** in `tools/oracle.py`, followed by `python3 tools/oracle.py --write vectors/t32-vectors.json`.
3. **The Rust crate**, until `cargo test --all-features` passes.
4. **The TypeScript port**, until `npm test` passes.

CI runs all of these and fails if the vectors file is out of date.

## Style

The specification is written in plain, short sentences, in the present tense, with BCP 14 keywords used only where a requirement is meant. Prefer an example to an adjective. Code follows each language's formatter: `cargo fmt` and the project's Prettier settings.

## Licence

Contributions are made under the project's licence, MIT OR Apache-2.0, as described in the [README](README.md#license).
