# Contender Windows investigation

This is an independent diagnostic, not a replacement Contender controller or
a native realm implementation. No production DLL, game setting, career or
executable is modified by the discovery/build commands below.

Use Python 3.11+ on Windows. All command examples follow this repository's RTK
convention. Paths below illustrate the environment used for the recorded run.

## Reproduce the static evidence

```powershell
rtk proxy python tools/rea/setup_environment.py
rtk proxy C:/Users/arian/.cache/tfm2-rea/python/Scripts/python.exe tools/rea/realm_discovery.py --exe "C:/Program Files (x86)/Steam/steamapps/common/Teamfight Manager2/TeamfightManager2.exe" --sdk "C:/Program Files (x86)/Steam/steamapps/common/Teamfight Manager2/mod-sdk-stable/mod-api-stable" --out docs/contender-realms/windows --max-functions 224 --seed-byte-budget 8192 --seed-rva 0x16ad0d0 --seed-rva 0x33d2d60 --seed-rva 0x1e130a0 --seed-rva 0x170d240 --seed-rva 0x33d0a10 --seed-rva 0x18c0480 --seed-rva 0x1765840 --seed-rva 0x1a098a0
rtk proxy C:/Users/arian/.cache/tfm2-rea/python/Scripts/python.exe tools/rea/run_focused_ghidra.py --ghidra C:/Users/arian/.cache/tfm2-rea/ghidra_12.1.4_PUBLIC --java-home "C:/Program Files/Android/Android Studio/jbr" --exe "C:/Program Files (x86)/Steam/steamapps/common/Teamfight Manager2/TeamfightManager2.exe"
rtk proxy C:/Users/arian/.cache/tfm2-rea/python/Scripts/python.exe tools/rea/scan_engine_callers.py --exe "C:/Program Files (x86)/Steam/steamapps/common/Teamfight Manager2/TeamfightManager2.exe" --out docs/contender-realms/windows/damage-callers.json --targets 0x18c0480,0x1e130a0,0x16ad0d0,0x33d0a10
rtk proxy C:/Users/arian/.cache/tfm2-rea/python/Scripts/python.exe -m unittest discover -s tools/rea -p test_*.py
```

The RVAs are caller-asserted seeds for the executable digest in the report.
They are not portable addresses or hook instructions. Never apply them to a
different executable. Omit the additional seeds for a different build and
repeat recovery from its SDK table. `--pid <owned-test-pid>` optionally checks
the selected executable's loaded SDK table/code bytes using read-only access;
it does not establish that any function was called.

`RealmSeedAnalysis.java` deliberately uses `-noanalysis` and bounded decoded
function bodies. Seven public wrapper prototypes come from the installed SDK;
private Rust signatures remain inferred. Global Ghidra startup did not finish
within the available provider window, so the focused output does not claim a
whole-program call graph. The wrapper runner hashes the source executable
before/after and checks actual project-file removal.

## Observer build (separate from installation)

```powershell
rtk proxy cargo +stable-x86_64-pc-windows-gnu test --release --locked --manifest-path experiments/contender-realm-observer/Cargo.toml
rtk proxy cargo +stable-x86_64-pc-windows-gnu test --release --locked --manifest-path experiments/contender-realm-observer/Cargo.toml --features observe-runtime
rtk proxy python tools/rea/build_observer.py --repeat
rtk proxy python tools/rea/check_observer_package.py
```

The default build is inert. The feature build needs its own `realm_probe.flag`,
the exact executable digest and the expected live SDK-table RVA. It registers
only its own public match observer. It reads a small, version-specific host
prefix to report the internal trait table, public entity snapshots and game
stack RVAs. No trait slots are invoked directly or overwritten, no combat RNG
is consumed, and no AI/ability/interaction replacement is registered.

Sampling is bounded to eight records per simulation and 64 diagnostic labels.
Raw host-state pointer labels are diagnostic, not persistent simulation IDs.
Do not use them to share state across matches, owners or replays.

The package is under the experiment's ignored `target/packages` directory.
The build tool does not install it. Production native/champion versions stay
unchanged because this diagnostic adds no released champion behaviour.
The repository Mod Manager requires its whole production pack in the fixture;
it does not offer a generic single-mod verifier. The check copies the unchanged
baseline plus the observer into a temporary directory, compares their bytes,
and restores the worktree manager configuration. Its pass does not certify the
observer's private host-prefix interpretation or live compatibility.

## Disposable runtime harness

`experiments/contender-realm-observer/sandbox.js` was used through an owned REA
Frida spawn session against an exact disposable executable path. It redirects
that process's known user-data directories and guards direct opens of the real
TFM2 roaming directory. This **does instrument filesystem functions**; it is
not part of the read-only collector or a realm backend. It grants no network
permission, changes no persistent environment and installs no gameplay hook.

Keep this redirection alive until the owned test process exits; detaching it
while the game runs could redirect later writes back to the normal profile.
Never attach this harness to the user's ordinary running game. Empty private
profiles are used, not copies of real careers. Hash manifests remain private;
only aggregate integrity results belong in published evidence.

The game presents a native-code permission warning. The user must approve that
warning. Do not edit the approval list or load the observer through another
path to bypass it. The current run has not passed that warning and has no
match-observer output. The static/live-byte evidence is still usable.

The recorded disposable process has now been closed before detaching its
filesystem isolation. `close_disposable.py` performs exact image-path checks
on one bound process handle; it refuses the installed game. The aggregate
profile-integrity report confirms all 38 real user-data files unchanged.

See [the evidence ledger](../../docs/contender-realms/README.md) for capability
status, known gaps and the staged validation requirements.
