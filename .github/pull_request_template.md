<!-- For anything beyond an editorial fix, link the issue or discussion this follows from. The title becomes the squashed commit: write it as a Conventional Commit. -->

## What and why

<!-- The problem, and the approach. -->

## Look at sceptically

<!-- Where the change is weak, uncertain or easy to get wrong. Say it before a reviewer finds it. -->

## Kind

- [ ] Editorial (wording, examples, no change in meaning)
- [ ] Erratum (the text, vectors or implementations disagreed)
- [ ] Substantive (changes what conforms; follows a discussion)
- [ ] Code, site or tooling only

## Checklist

- [ ] `scripts/task check` passes
- [ ] For a change to normative behaviour: `SPEC.md` changed with an entry in Appendix D; `tools/oracle.py` changed and `vectors/t32-vectors.json` regenerated; the Rust crate and the TypeScript port updated
- [ ] For a release: `CHANGELOG.md` has the version's section
