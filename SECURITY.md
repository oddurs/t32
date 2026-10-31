# Security

T32 is a data format, so most of its security questions are about parsing untrusted text and about the leap-second table. The specification's *Security and privacy considerations* section covers the design; this page covers reporting.

## Reporting a vulnerability

Please report suspected vulnerabilities privately, through **Security → Report a vulnerability** on the [repository](https://github.com/oddurs/t32/security), rather than in a public issue. Include what you found, how to reproduce it, and which versions are affected.

You can expect an acknowledgement within a week. Once a fix is ready it is published with an advisory that credits you, unless you would rather not be named.

## In scope

- Panics, hangs, unbounded allocation or arithmetic overflow in the `t32` crate or the TypeScript implementation, on any input.
- Parsing that accepts text the grammar rejects, or that maps two different texts to stamps that compare unequal when they should not.
- A specification rule that leads conforming implementations to produce a misleading time.

## Supported versions

| Component | Supported |
| --- | --- |
| `t32` crate | The latest 0.x release |
| Specification | The latest draft |
