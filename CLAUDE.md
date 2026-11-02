# Working in this repository

T32 is a draft timestamp standard. The specification (`SPEC.md`) is the product; the conformance vectors, the Rust crate (`rust/`), the TypeScript port (`web/src/lib/t32.ts`), the site (`web/`) and the Internet-Draft (`ietf/`) serve it. Where they disagree with the text, the text decides and the disagreement is a defect.

## Workflow

- Never commit to `main`. It advances only by a merged pull request; the `pre-push` hook and branch protection refuse anything else.
- One unit of work, one worktree, one branch, one pull request:

  ```sh
  scripts/agent start <type>/<slug>   # type: feat fix chore docs perf refactor test
  scripts/agent commit "<message>"
  scripts/agent pr                    # runs scripts/task check, pushes, opens the PR
  scripts/agent done                  # after the merge
  ```

  Worktrees live in `../.worktrees/t32/<branch>/`. Never share one with another agent. `scripts/agent doctor` checks the setup; `scripts/agent sync` rebases onto `main`.
- Pull requests are squash-merged: the PR title becomes the commit on `main`, so write it as a Conventional Commit. The description states the problem, the approach, and what a reviewer should look at sceptically.
- The single required status check is `required`, which needs every CI job.

## Checks

`scripts/task` is the only entry point to the toolchains. CI and the hooks call nothing else.

| Command | Runs |
| --- | --- |
| `scripts/task fmt` | `cargo fmt` |
| `scripts/task fmt:check` | `cargo fmt --check` |
| `scripts/task lint` | clippy with warnings denied; `svelte-check` |
| `scripts/task test` | cargo tests with and without features, the example, `tools/oracle.py --check`, `npm test` |
| `scripts/task build` | `cargo doc` (warnings denied), `cargo package`, the site build |
| `scripts/task check` | all of the above |

A branch passes `scripts/task check` before it becomes a pull request. Never use `--no-verify`, `|| true` or `continue-on-error` to get past a failure: fix the cause.

## Commits

Conventional Commits: `type(scope): subject`, imperative, at most 72 characters, no trailing period. The body explains why. The `commit-msg` hook enforces the format.

Never name Claude, Anthropic, an AI or an assistant in a commit, PR, comment, doc, changelog or release note. No co-author trailers, no "generated with" footers, no robot emoji. The work is published under its author's name. The `commit-msg` hook rejects attribution lines; strip any a tool adds before it reaches git or GitHub.

## Invariants

- **The vectors are generated.** Never edit `vectors/t32-vectors.json` by hand. Change `tools/oracle.py`, then `python3 tools/oracle.py --write vectors/t32-vectors.json`. Every vector, errors included, comes from the oracle.
- **A normative change touches four places in one PR:** the text of `SPEC.md` with an entry in Appendix D; the oracle and the regenerated vectors; the Rust crate; the TypeScript port. Then bring `ietf/draft-sigurdsson-t32-00.tex` along: its BCP 14 keyword counts must match `SPEC.md`'s exactly.
- **Error codes are the spec's names** (`empty`, `too_long`, `invalid_char`, `misplaced_hyphen`, `missing_sign`, `invalid_date`, `not_a_leap_second`, `removed_time`, `out_of_range`), reported in the spec's order: left to right for parsers; `invalid_date`, `not_a_leap_second`, `removed_time`, `out_of_range` for UTC labels.
- **Arithmetic is exact.** No floating point in anything that decides a stamp, a length or a conversion; Rust uses integers, TypeScript BigInt, the oracle `Fraction`.
- **The Guide's Rust example is a doctest.** `rust/src/lib.rs` compiles `web/src/content/docs.md` under `cfg(doctest)`. A Rust block there must compile; rustdoc's hidden `# ` lines are hidden on the site too.
- **The site's draft number, date and vector count** live in `web/src/lib/site.ts`; `site.test.ts` checks them against `SPEC.md` and the vectors. Update them with a new draft.
- **`web/src/lib/t32.ts` stays one file with no dependencies.** People copy it into their projects.
- **A new draft** bumps the number and date in `SPEC.md`, adds an Appendix D entry, and is tagged `spec-draft-<number>` after it merges.

## Releases

- The site deploys from `main` through `.github/workflows/pages.yml`, after CI passes on the merged commit.
- The crate is published by dispatching `.github/workflows/publish-crate.yml` on `main`. It runs CI, refuses a version without a `## [x.y.z]` section in `CHANGELOG.md`, publishes, tags `t32-v<version>`, and makes the GitHub release from that section. Bump `rust/Cargo.toml` and move the `[Unreleased]` notes first. While the spec is a draft, the crate's minor version follows the draft.
- `CHANGELOG.md` follows Keep a Changelog, with compare links at the bottom.

## Writing

The specification is plain, short sentences in the present tense. BCP 14 keywords appear only where a requirement is meant. Prefer an example to an adjective. Comments say why; the code says what.
