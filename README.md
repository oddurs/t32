# T32

A compact universal timestamp. T32 writes any moment as 8 base32 characters: a count of atomic (TAI) seconds since 2000, offset so the past encodes too.

```
G0S7G6B1        2026-10-31 03:13:00 UTC, to the second
G0S7G6B1G0      … to about a millisecond
G0S7            the 12-day span that contains it
```

- **Length is resolution.** Drop characters for a coarser span, append them for finer time. Every stamp names an interval.
- **Text order is time order.** Byte-wise sorting is chronological, and a prefix is a range query.
- **Uniform seconds.** TAI has no leap seconds, so from 1961 on, subtracting two stamps gives true elapsed time.

**Site:** https://oddurs.github.io/t32/ (playground, docs, reference tables)
**Specification:** [SPEC.md](SPEC.md) (draft 0.2)

## Repository

| Path | What |
| --- | --- |
| [`SPEC.md`](SPEC.md) | The specification, with test vectors and the TAI − UTC table |
| [`rust/`](rust) | Reference implementation, standard library only |
| [`web/`](web) | The site (SvelteKit, static) and a single-file TypeScript implementation, [`web/src/lib/t32.ts`](web/src/lib/t32.ts) |
| [`tools/oracle.py`](tools/oracle.py) | An independent check in exact rational arithmetic, written from the spec |

## Development

```sh
cd rust && cargo test                      # reference implementation
cd web && npm ci && npm test               # TypeScript implementation
cd web && npm run dev                      # site at http://localhost:5173
python3 tools/oracle.py                    # recompute the spec's vectors exactly
```

The specification's text and test vectors are normative. Both implementations and the oracle reproduce every vector. Pushes to `main` deploy the site to the `gh-pages` branch.

## Status

T32 is a draft proposal, not an adopted standard. Objections, edge cases and prior art are welcome as [issues](https://github.com/oddurs/t32/issues).
