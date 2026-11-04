# Internet-Draft

`draft-sigurdsson-t32-00.tex` is the T32 specification laid out as an IETF Internet-Draft, following the structure of RFC 9562. Its requirements are those of draft 0.3 of [SPEC.md](../SPEC.md) (tag `spec-draft-0.3`), restated in full; a difference between the two is a defect in one of them. The motivation, terminology, figures and worked example are informative additions for IETF readers. Unlike SPEC.md, it makes only its own vector tables normative, because an Internet-Draft carries its normative content itself.

The PDF is built with every deploy and published at <https://t32.serre.ai/draft-sigurdsson-t32-00.pdf>, linked from the site's Specification page. `scripts/task build` builds it with [tectonic](https://tectonic-typesetting.github.io) into `web/static/`.

## Links in the next revision

A published revision is never edited, so `-00` still names the repository's first home, `github.com/oddurs/t32`. GitHub redirects those links to `github.com/serre-ai/t32` for as long as no new repository takes the old name. The next revision, `-01`, uses the durable addresses:

| What | Address |
| --- | --- |
| Source, errata and questions | <https://github.com/serre-ai/t32> |
| Implementations | <https://github.com/serre-ai/t32/blob/main/IMPLEMENTATIONS.md> |
| The specification at its draft tag | `https://github.com/serre-ai/t32/blob/spec-draft-<number>/SPEC.md` |
| Latest version | <https://t32.serre.ai/spec/> |
| This draft as a PDF | `https://t32.serre.ai/draft-sigurdsson-t32-01.pdf` |

To build it with TeX Live instead, run `pdflatex` three times: the second pass places the table of contents, the third corrects its page numbers.

```sh
cd ietf
for pass in 1 2 3; do pdflatex draft-sigurdsson-t32-00.tex; done
```

## Submitting

The draft is prepared for review. Whether to submit it to the IETF is a decision for the project's working group, not its editor ([GOVERNANCE.md](../GOVERNANCE.md)).

The IETF Datatracker accepts drafts only as RFCXML v3, so this file is for reading and review, not for upload. To submit, convert it to RFCXML (or to kramdown-rfc Markdown, which produces RFCXML), check it with `xml2rfc` and `idnits`, and upload at <https://datatracker.ietf.org/submit/>. Artwork stays within 69 columns: xml2rfc indents artwork by 3 within the 72-column limit of RFC text.

Posting an Internet-Draft grants the IETF Trust the rights described in BCP 78, and BCP 79 asks contributors to disclose any patents they know of. Both apply on submission; the repository's MIT and Apache-2.0 licences do not change that.
