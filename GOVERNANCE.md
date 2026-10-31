# Governance

T32 is an open specification. This document describes how it changes, who decides, and how it is meant to grow from one editor's draft into a standard maintained by the people who use it.

## Principles

- **The specification is the product.** Code, the site and the vectors serve the text. Where they disagree, the text decides, and the disagreement is a defect to fix.
- **Changes are made in the open.** Every change to normative text is proposed publicly, carries its reason, and is recorded in Appendix D of the specification.
- **Rough consensus and running code.** Decisions follow the weight of argument, not the count of votes, and a proposal is stronger for an implementation and vectors that show it works.
- **Stability is a feature.** A stamp, once written, means the same interval of time forever. Proposals that would change the meaning of existing stamps are out of scope after version 1.0.

## Roles

**Editor.** Oddur Sigurdsson ([@oddurs](https://github.com/oddurs)) edits the specification. The editor keeps the text coherent, judges consensus, merges changes and records them. The editor may decline a proposal, and does so in writing, with reasons, on the proposal itself.

**Contributors.** Anyone who reports a defect, proposes a change, reviews a proposal, writes an implementation or adds vectors. Contribution needs no permission.

**Implementers.** Authors of the implementations listed in [IMPLEMENTATIONS.md](IMPLEMENTATIONS.md). Their experience carries particular weight, because they find what the text leaves unsaid.

## How a change is made

| Kind | Examples | Where | How it is decided |
| --- | --- | --- | --- |
| Editorial | Typos, clearer wording, examples | Pull request | The editor merges it |
| Erratum | Text and vectors disagree; a rule can be read two ways | Issue, then pull request | The editor fixes it, noting the fix in Appendix D |
| Substantive | New rules, changed behaviour, anything in Appendix C | Discussion, then a proposal | Rough consensus, judged by the editor, after a review period |

A substantive proposal states the problem, the change to the text, its effect on existing stamps and implementations, and new or changed vectors. It is open for review for at least two weeks before a decision, and the decision is written on the proposal with its reasons. A decision can be revisited when new evidence arrives.

## Versions

- **Drafts (0.x).** Any part may change. Each draft is tagged, dated and listed in Appendix D.
- **Version 1.0.** Definitions, Encoding, Decoding and the grammar are frozen. Later 1.x versions may add, for example rows in Appendix A or new profiles, but may not change what an existing stamp means.

Version 1.0 will be published when all of the following hold:

1. Every question in Appendix C has a recorded decision.
2. At least three independent implementations, in at least two languages and by different authors, pass the full vectors file.
3. A public review period of at least 60 days has passed since the last substantive change, announced in advance.
4. No erratum is open.

## Toward a working group

One editor is the right size for a draft. A standard that people depend on needs more than one person, and T32 is meant to outgrow its author.

When version 1.0 is published, or earlier if there is enough interest, a **T32 Working Group** will be formed by a short public charter covering its scope, membership, decision process and the transfer of the repository to a neutral organisation. Its first members will be drawn from active implementers and reviewers. The editor's role then becomes a role of the group, held by whoever the group chooses.

Beyond that, if T32 sees real use in protocols, the group may take the specification to an established standards body, for example as an IETF Internet-Draft, and maintain it there. That step is the group's to take, not the editor's.

If you would like to take part in this, say so in [Discussions](https://github.com/oddurs/t32/discussions). Interest expressed now shapes the charter later.

## Conduct

Everyone taking part in the project follows the [Code of Conduct](CODE_OF_CONDUCT.md).
