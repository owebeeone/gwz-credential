# Acquisition boundary — accepted design

Date: 2026-10-19. **GO / GO** at root
`5dd59da1ea5123e0de850ba4b2409a4ef1b7e1d9` and portable reference member
`50838c44f8faf4bc6eeb618370ee6ddc10cdd2fc`.

- [Code + Surface](CredentialAcquisition-ReviewCode-2.md): GO, zero open P0–P3.
- [State](CredentialAcquisition-ReviewState-2.md): GO, zero open P0–P3.

Acceptance covers the revised COMPLETE Credential lifecycle and Connection seam
in root dev-docs/GwzCredentialAcquisitionAmendment.md revision2. It is not source
implementation, native-provider, actual SSH, placement/reuse or application
integration acceptance. Production source still equals the earlier portable
kernel; the new acquisition API/first explicit-file source is the next authorized
implementation chunk.

The public acquisition is one custom Send/Unpin Future:
`context.acquire(selection, purpose, control).await -> Result<PreparedCredential, AuthError>`.
It internally admits/retains potentially blocking source work before effects,
then selects immutable material and publishes once under the revocation guard.
Explicit-key exact bytes exist before every pool lookup. There is no public
wait/Acquired/select chain and no fabricated physical peer for acquisition.
Cancellation reports immediately; unfinished work remains charged/observed until
actual disposal. Completed-Future cancel is truthful observation, not revocation
of an already transferred prepared credential. Re-poll cannot transfer twice.

Two correction rounds, zero architectural roots. Round0 Code found an undefined
closing report. The operator also requested fewer caller chores; direct Future
replaced the intermediate public chain. Round1 BOTH axes independently found the
same post-completion cancel omission, a local state rule. Round2 closes it on
both axes. Reports are verbatim. Source-provider conformance/native qualification
remains separate from generic kernel and design evidence.

The operator's SDAX reference was assessed read-only in the task plan. No SDAX,
Tokio, new runtime or general supervisor is adopted by this verdict. Application,
Connection, core, Git, transport, CLI/Python and SSPI sources remain reference-only.
