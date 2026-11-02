# Contributing to T32

Thank you for taking an interest. T32 is a draft, so this is the time when a careful reading, a question or a second implementation changes the most.

You do not need permission to contribute, and no contribution is too small. A sentence that can be read two ways is worth an issue.

## Ways to help

**Read the specification critically.** Read [SPEC.md](SPEC.md), or the [rendered version](https://oddurs.github.io/t32/spec/), as if you had to implement it from the text alone. Report anything ambiguous, missing or wrong as an [issue](https://github.com/oddurs/t32/issues/new/choose).

**Weigh in on the open questions.** Appendix C of the specification lists the questions still open before version 1.0. Each states the current position and what would change it. Experience from your own systems is the most useful evidence.

**Write an implementation.** The format is small, and an independent implementation is the best test the specification can have. Load [`vectors/t32-vectors.json`](vectors/t32-vectors.json), make every vector pass, and add your implementation to [IMPLEMENTATIONS.md](IMPLEMENTATIONS.md) with a pull request. Implementations that disagree with the vectors are valuable too: either the implementation or the specification is wrong, and both are worth knowing.

**Propose a change.** Start a thread in [Discussions](https://github.com/oddurs/t32/discussions) under *Ideas*; its form asks for what a proposal needs. Describe the problem before the solution. If the idea holds up, turn it into a pull request that changes the text, the vectors and both implementations together. [GOVERNANCE.md](GOVERNANCE.md) describes how proposals are decided.

**Improve the implementations or the site.** Bug reports and fixes for the Rust crate, the TypeScript port and the site are welcome through the usual issues and pull requests.

## The repository

| Path | What |
| --- | --- |
| `SPEC.md` | The specification; read it rendered with `cd web && npm run dev` |
| `vectors/t32-vectors.json` | The conformance vectors, generated; never edit by hand |
| `tools/oracle.py` | The exact-arithmetic oracle that generates them (Python 3.8 or later, standard library only) |
| `rust/` | The reference implementation, the `t32` crate |
| `web/src/lib/t32.ts` | The TypeScript implementation |
| `web/` | The site |
| `ietf/` | The specification as an Internet-Draft |

## Setting up

You need Git, the [GitHub CLI](https://cli.github.com/), [rustup](https://rustup.rs/), Node and Python 3. The Rust and Node versions are pinned in `rust-toolchain.toml` and `.node-version`. Once, after cloning:

```sh
scripts/setup    # wires the git hooks, installs the site's packages, checks the tools
```

## Making a change

Every unit of work gets its own branch, in its own worktree beside the checkout, and lands through a pull request. `main` only moves when a pull request merges; the hooks and branch protection refuse anything else.

```sh
scripts/agent start fix/clearer-error-text   # branch and worktree from main; prints where to cd
# … edit, then:
scripts/agent commit "fix(web): say which field is invalid"
scripts/agent pr                             # runs every check, pushes, opens the pull request
# … after it merges:
scripts/agent done                           # removes the worktree and the branch
```

Branches are named `<type>/<slug>`, with type one of `feat fix chore docs perf refactor test`. Commits follow [Conventional Commits](https://www.conventionalcommits.org/): `type(scope): subject`, imperative, at most 72 characters, no trailing period, with a body that says why. The `commit-msg` hook checks this.

One command runs every check CI runs: formatting, lints with warnings denied, all test suites, the vectors, the docs, the crate package and the site build.

```sh
scripts/task check    # or one step: fmt, fmt:check, lint, test, build
```

The `pre-commit` hook runs formatting and lints; `pre-push` runs the whole check. Never skip them with `--no-verify`: CI runs the same check and will fail. `scripts/agent sync` rebases your branch onto `main`; `scripts/agent list` shows your worktrees and their pull requests.

Pull requests are squash-merged, so the pull request's title becomes the commit on `main`: write it as a Conventional Commit too. The description says what problem it solves, how, and what a reviewer should look at sceptically. The project has one maintainer, so a merge needs a passing `required` check and resolved conversations but no approval; that changes when the working group forms.

## Changing normative behaviour

A change to how stamps are encoded, decoded, parsed or converted touches four places, and a pull request should change all four together:

1. **The text** in `SPEC.md`, with an entry in Appendix D.
2. **The oracle** in `tools/oracle.py`, followed by `python3 tools/oracle.py --write vectors/t32-vectors.json`.
3. **The Rust crate**, until `cargo test --all-features` passes.
4. **The TypeScript port**, until `npm test` passes.

`scripts/task check` runs all of these, and fails if the vectors file is out of date.

## Style

The specification is written in plain, short sentences, in the present tense, with BCP 14 keywords used only where a requirement is meant. Prefer an example to an adjective. Rust follows `cargo fmt`. The web code has no formatter; `scripts/task lint` and `scripts/task build` gate it. `.editorconfig` sets indentation and line endings for every file type.

## Licence

Contributions are made under the project's licence, MIT OR Apache-2.0, as described in the [README](README.md#license).
