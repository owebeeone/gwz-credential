# Explicit-key source admission: bounded contract question

Date: 2026-10-19. Status: implementation stopped before production changes.
This is a confirmed limitation of the direct synchronous selection approach,
not proof that every implementation behind the accepted API is impossible.

## Observation and executed counterexample

`Provider::select` returns `Result<Arc<dyn SelectedCredential>, AuthError>`
synchronously. `CredentialContext::select` invokes it inline, after reserving the
selection charge and before a conversation exists. It passes immutable input and
selection, with no deadline/cancellation or pending source-work handle. The
public integration obligation requires selection and provider polls to return
promptly. `PreparedCredential::begin` is the later point that reserves a retained
conversation and introduces its absolute deadline/cancellation/observation.

`tests/source_admission.rs` models an explicit-file source read held at an
external boundary using a channel barrier. Once the read has entered, both
owner and operation close are requested and the independent retainer is driven.
The selecting call is still blocked; its memory reservation remains charged;
there is no registered `RetainedWork`. Releasing that one owned probe read lets
the thread return and all charges disappear. The probe always releases and joins
its own thread and uses no host credentials, sockets, sleeps or real key bytes.
It passes on the baseline API and intentionally demonstrates a nonconforming
blocking provider; it does not claim a defect in providers that obey the port.

On 2026-10-19 the focused command passed on Mac:
`cargo test --offline --test source_admission`.

## Existing supported route and alternatives examined

Core reference `src/git/endpoint/ssh_key_snapshot.rs` does not read directly on
selection: Registry::start reserves a bounded snapshot and uses its existing
Supervisor/Job; Reservation::read checks control around each chunk, then intern
publishes exact-byte identity only after joined admission and a live-request
check. Core regular_file::open refuses special files, including Unix FIFO and
Windows devices/pipes. That protects special-file admission but does not bound
ordinary disk/network-filesystem read latency. Replacing this with plain
`std::fs::read`, an unchecked metadata/read pair or a blocking read in a provider
poll would discard existing prompt-return and cleanup ownership properties.

A preloaded provider could make selection prompt, but loading before the kernel
reservation creates a separate source-admission owner/budget/lifecycle. There is
no such composition owner in this isolated member; declaring it a prerequisite
would need explicit contract treatment, not an uncharged constructor workaround.

Moving the read to begin/start could potentially use a retained session and
injected source execution. It changes when exact bytes become admitted and
requires defining how selected snapshot identity is established before native
key borrowing and later reuse. This option has not been disproved; it has not
been implemented or accepted. A new ad hoc thread/supervisor or a source-service
port is not silently introduced by this investigation.

## Contract question and next handoff

Which accepted owner admits the first bounded file snapshot, under what request
control, and at which lifecycle boundary? Specifically, should select consume an
already admitted snapshot from a separately specified source-admission service,
or should admission run after begin under its existing retained conversation?
For either route, specify original caller/base authority, no path reopen or
fallback, exact-byte identity, absolute deadline, cancellation, and ownership of
a stalled read until joined/disposed. The existing native key pin must continue
through NeedNetwork and confirmed physical disposal.

This demonstrably blocks implementing the direct synchronous-read approach
without violating the accepted prompt-return obligation. It is not an authorized
API amendment or reason to change Connection/core/CLI/Python. The provider task
pauses at this question while independent public hosting and native portable
qualification proceed. No real explicit-key provider, key parser, native SSH
authentication, application integration or placement/reuse qualification is
claimed. The next bounded step is one reviewed source-admission decision and
its smallest provider proof; Connection composition remains separate.

## Operator disposition

The operator authorized correcting the acquisition boundary and reconciling the
requirements before production implementation. The now-pinned Connection §7.1
requires explicit key reread before EVERY pool lookup; begin-only acquisition
would read too late. A complete revised lifecycle is DRAFT at root
`dev-docs/GwzCredentialAcquisitionAmendment.md` (c9640db2), under joint review.
This question remains the original discovery record, not a veto on that newly
authorized redesign or a claim that the actual provider outcome is impossible.
