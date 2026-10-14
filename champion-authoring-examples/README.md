# Champion authoring examples

These are teaching examples accompanying Volume 2. They are not installed or shipped champions.

## Placement

Extract this folder as `champion-authoring-examples` directly inside your `tfm2_mods` checkout. The native example then resolves the existing SDK at `../../native/mod-api-stable`. The SDK is not bundled here.

## Native example

Run `cargo test --manifest-path champion-authoring-examples/native-example/Cargo.toml`.

The example registers `guide_rules:burst`, requires stable ABI 9, and uses a deterministic isolated core. Three tests verify cooldown/target ties, invalid/blocked behavior, cloned replay and unrelated-instance isolation. Its source was compile-checked; a Windows GNU release DLL was built in the research environment. It was not installed or tested in a Windows game match.

To build a supported platform, use the Rust target and linker for that platform. The package is named `guide_rules`, so its Windows DLL is `guide_rules.dll`. Add versioned native metadata and attach the passive as shown in Volume 2 before any game experiment. Do not enable this directly inside your normal career.

## Data example

`data-example/champion/guide_new_champion.data_champion` is the complete JSON prototype printed in Volume 2. It reuses a base Sand Mage sprite. JSON syntax and animation tags were checked; the installed game's loader was not run. Complete package metadata, localization and target game compatibility checks as described in the guide.

## Addendum modules (native example)

Three small modules, each with tests, carry the lessons Volume 2 is missing. The repository's
`docs/champion-guide-addendum.md` explains each one in full.

- `src/facing.rs`: the game mirrors only a champion's own body sprite. Buff overlays and view effects are never
  mirrored, so a champion drawn as overlays ships `<layer>_l` copies and picks them with DIO's facing rule.
- `src/zones.rs`: how to tell the game's AI that an area is dangerous. Publish a buff name, have the input AI
  parse it, and rewrite move orders only. Never rewrite casts or attacks: the planner panics.
- `src/budget.rs`: the effect token bucket. A low-priority reserve keeps opens, hits and casts visible, and the
  busiest second is bounded.

`cargo test` runs 8 tests (the 3 originals plus 5 for these modules).

## Before any game experiment

- **Versions agree, all three:** `native/tfm2_custom_ai/src/lib.rs` `VERSION`, `native/tfm2_custom_ai/mod.mod_info`
  and `mods/tfm2_custom_ai/mod.mod_info`. The champion pack's `mod.mod_info` must also require `>=` that version.
  A mismatch once disabled every champion. The test `every_version_agrees` checks it.
- **The data example needs its package:** a `mod.mod_info`, `mod.override_info` and `text/champion.i18n` entry, as
  in Volume 2. Its `category` and `tags` must come from the accepted lists, or the game skips the champion
  (the Manager's Check reports it).
- **Sand Mage is a stand-in.** Your own body art must face right (the game mirrors it). Anything you draw as a buff
  overlay will not turn around unless you ship `_l` copies.
- **Renaming or splitting a VFX sheet** leaves the old files in players' game folders. Add them to `STALE_FILES` in
  `tools/manager/src/core.rs` and rebuild `TFM2 Mod Manager.exe`.

## Validation boundaries

Source, compile and fixture checks do not establish live host behavior, game balance or frame rate. The two PDF volumes separate fresh checks, historical reports and proposed experiments.
