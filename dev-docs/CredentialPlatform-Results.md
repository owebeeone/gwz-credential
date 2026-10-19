# Standalone native platform results

Date: 2026-10-19. Source/test/CI checkpoint:
`f6df6be69d54927b3eed6140aa59f92bb2ab13a4`.
Production source equals the accepted portable kernel d6fc64d6. This checkpoint
adds public CI and the controlled-stall source-admission counterexample; no real
source provider is implemented or qualified by these results.

All platforms passed the same pinned gates: cargo +1.95.0 fmt --check;
cargo +1.95.0 test --offline --locked; cargo +1.95.0 clippy --offline --locked
--all-targets -- -D warnings; cargo +1.95.0 doc --offline --locked --no-deps.
The ordinary runner exit status is the gate; no count/inventory pins.

| Native host | Target/filesystem | Result |
| --- | --- | --- |
| Mac | aarch64-apple-darwin, external build target | All four gates PASS |
| dabeest Win11 | x86_64-pc-windows-msvc, isolated E: ReFS checkout | All four gates PASS |
| weftpi Debian13 | aarch64-unknown-linux-gnu, isolated /mnt/work checkout | All four gates PASS |

[Public CI39248862281](https://github.com/owebeeone/gwz-credential/actions/runs/39248862281)
also passed on hosted Ubuntu, macOS and Windows. Public build/test/CI needs only
this repository and its locked dependency; no private evidence or adjacent GWZ
member is needed.

Raw commands, toolchains, logs, initial failed discovery/unpinned attempts and
hashes are in the private
[credential qualification run](https://github.com/owebeeone/gwz-core-evidence/tree/main/campaigns/credential-qualification/runs/2026-10-19-portable-hosting)
(access required). Windows E: ReFS was explicitly selected by the operator;
Get-Volume confirmed it. The Windows shared booking was released by its owner.
No host-wide security, service, trust or agent configuration changed.

The first Mac invocation used the root toolchain through --manifest-path; tests
passed but newer clippy rejected the existing manual no-op waker. That invocation
was not a1.95.0 qualification gate. All Mac gates were then run with explicit
+1.95.0 and passed without source changes. Windows/Linux used the pin explicitly.

Public hosting is complete at
[owebeeone/gwz-credential](https://github.com/owebeeone/gwz-credential).
Accepted history was audited for private-key/token patterns and raw/build paths
before the initial public push. No package publication, release or tag occurred.

These are portable-library/source-counterexample results, distinct from native
file acquisition, SSH authentication, endpoint placement, persistent connection
reuse and CLI/Python qualification. The operator subsequently authorized an
acquisition-boundary redesign; its joint DRAFT is root
`dev-docs/GwzCredentialAcquisitionAmendment.md` at c9640db2. Production changes
remain paused until that revised joint boundary passes review.
