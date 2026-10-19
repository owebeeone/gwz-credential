# Portable Credential implementation — accepted

Date: 2026-10-19.

**GO / GO** at gwz-credential
`d6fc64d6a38457018377020765229405920a4833`.
Immutable root API/boundary authority:
`0de87d56764792ea70cbe75e5ea0148d3ad7b43c`.
The final evidence commit changes documentation only, with identical reviewed
source, tests, toolchain, package and lockfile.

- [Code report](CredentialPortable-ReviewCode-2.md): GO, no open P0–P3.
- [State report](CredentialPortable-ReviewState-2.md): GO, no open P0–P3.
- [Implementation scope and resume point](Implementation.md).

## Delivered

A registered local member with an independent Rust library. Explicit provider
and operation owners, cloneable selection views, independent conversations,
bounded shared admission/retention, absolute deadlines/cancellation, opaque
endpoint-local response commitment, native key-borrow pin/probe interfaces,
connection-scope grants and pre-effect generation-qualified terminal observation.
No globals, sockets, native workers or other GWZ production dependencies. The
only production dependency is the already-used pinned zeroize 1.9.0.

Standalone tests use fake providers and stepped retention. They cover 10,000
sequential operations, 500 replayable randomized schedules, publication/cancel
ordering, independent operations/conversations, native-key call barriers, retained
cleanup, lost observations, full terminal provenance, allocation-budget refusal
and native paths. They perform no real authentication or wire traffic.

## Gates at the reviewed source

All passed from this member:

- cargo test --offline
- cargo fmt --check
- cargo clippy --offline --all-targets -- -D warnings
- cargo doc --offline --no-deps

No core/native/platform build is required for these portable tests. Real provider,
platform, installed endpoint or production test-cost qualification was not run.

## Review ledger

| Revision | Code | State | Result |
| --- | --- | --- | --- |
| 3c342c4a | NO-GO, 4 P2 | NO-GO, 4 P2 | Six distinct implementation defects; both axes independently found missing retained observation and selection charge omissions. |
| edf0d776 | NO-GO, original findings closed; new local P2-5 | GO | Fixed peer payload omitted from newly introduced long-lived observation charge. |
| d6fc64d6 | GO | GO | All findings closed; same reviewers verified original counterexamples and correction preservation. |

Two remediation rounds. One architectural implementation root, the missing
observation boundary, closed under the existing accepted design. No design
amendment or new architectural root in subsequent rounds. Remaining defects were
local transition, compatibility and accounting omissions; regressions now cover
them. No production escape rate is claimed for an unintegrated library.
Reports are preserved verbatim. Intermediate packages are in history, byte
unchanged; historical references were not rewritten.

## Limits and stop condition

This acceptance covers the portable lifecycle/provider kernel only. There are
no real credential-source providers, HTTP offer parser, SSH cryptography,
production progress service, endpoint bridge, native qualification or CLI/Python
adoption. Those remain explicit next components, not delivered capabilities.
The caller must inject the bounded service that continues progress after public
handles disappear, and trusted providers must honor declared allocation/disposal.

No GitHub hosting, push, tag, publishing, activation or release was performed.
No adjacent source or builder lane was changed. Python remains paused. The
portable implementation stops at this recorded checkpoint.
