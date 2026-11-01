# Internet-Draft

`draft-sigurdsson-t32-00.tex` is the T32 specification laid out as an IETF Internet-Draft, following the structure of RFC 9562. Its requirements are those of draft 0.3 of [SPEC.md](../SPEC.md) (tag `spec-draft-0.3`), restated in full; a difference between the two is a defect in one of them. The motivation, terminology, figures and worked example are informative additions for IETF readers. Unlike SPEC.md, it makes only its own vector tables normative, because an Internet-Draft carries its normative content itself.

Build the PDF with any TeX Live installation. Run it three times: the second pass places the table of contents, the third corrects its page numbers.

```sh
cd ietf
for pass in 1 2 3; do pdflatex draft-sigurdsson-t32-00.tex; done
```

Or run `tectonic draft-sigurdsson-t32-00.tex`, which fetches missing packages and repeats passes itself.

## Submitting

The draft is prepared for review. Whether to submit it to the IETF is a decision for the project's working group, not its editor ([GOVERNANCE.md](../GOVERNANCE.md)).

The IETF Datatracker accepts drafts only as RFCXML v3, so this file is for reading and review, not for upload. To submit, convert it to RFCXML (or to kramdown-rfc Markdown, which produces RFCXML), check it with `xml2rfc` and `idnits`, and upload at <https://datatracker.ietf.org/submit/>. Artwork stays within 69 columns: xml2rfc indents artwork by 3 within the 72-column limit of RFC text.

Posting an Internet-Draft grants the IETF Trust the rights described in BCP 78, and BCP 79 asks contributors to disclose any patents they know of. Both apply on submission; the repository's MIT and Apache-2.0 licences do not change that.
