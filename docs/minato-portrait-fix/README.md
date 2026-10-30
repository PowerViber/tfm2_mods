# Minato blank roster portrait

The user's game screenshot showed an empty Minato card next to Monk. Baseline `777204f` (merged PR #52, native 0.10.78) reproduced the cause: Minato's actual `sprite` points to `minato_presenter_base`, whose `idle` and all other frames were transparent. Roster/draft screens do not run the match passive that adds his body layer. The editor's thumbnail alias showed a different, visible asset and masked this failure.

## Fix

- Keep the same asset path, but copy every approved normal Minato frame into it. Idle, movement, casts, hits and death now have a visible fallback. The champion combat definition and saved ID are unchanged; this repair does not require a career-definition migration.
- Append a separate one-pixel transparent `mn_body_hidden` animation. The native presenter requests that pose for one tick only while an alive match entity uses a normal/KCM body layer. This uses the supported `CcKindV1::Animation` visual channel, already used for Gojo's poses. It does not issue Stun, Bind, attack/skill blocks, movement, invisibility, banish, damage or stat commands, and never clears unrelated CC.
- Confirm the host accepted the named override before adding a body overlay; otherwise clear the overlay and retain the visible fallback. Death clears the overlay and lets the engine draw the normal corpse, avoiding a duplicate fixed-position corpse effect.
- Remove the editor-only alias. Tests now inspect the same base asset named in the game's data, instead of substituting the visible legacy alias.

No new appearance or effect designs were drawn. Existing normal/KCM source artwork, anchors, animation timing, combat values, AI and other champions are unchanged. Main's newer 0.10.78 changes are retained. Release versions: native **0.10.79**, custom **0.2.79**.

## Verification

`verify-minato-portrait.py` demonstrates the empty baseline idle and visible repaired idle, compares every fallback frame with the approved normal body, checks the separate hidden tag, exact unchanged champion data, allowed presenter host operations, absence of unrelated CC clearing or duplicate corpse effects, and the 32 MiB budget. It renders `portrait-before-after.png` directly from the real base animation. All 74 nonempty original frames are present. Minato's decoded presentation is **8,589,312 bytes**; the repaired atlas is **1024x341**.

Generation is byte-reproducible across the 11 Minato asset/rules/data files. The final native release suite passed **766 tests, zero failed, ten ignored**. The one-tick named-animation packet has a regression test alongside pose-clock/form/interruption tests. Skill Test/browser checks load both forms, visibly switch to KCM, preserve inspected combat fields, and use the actual base asset without an editor alias; no browser errors were recorded.

Two independent release builds match the packaged DLL SHA-256: `ea3874a7816ce9c1478b37ca18bd4428662e2748a1f124a73399c4a977b6e3f4`. Isolated Mod Manager verification passed: 836 files hash-compared, 723 champion files compared, zero differences/missing and an identical native DLL. Its “game” result refers only to the temporary fixture. See `package.json`, `mod-manager.json`, `native-tests.json` and `skill-test.json` for evidence.

The initial test run caught the newer main branch's internal native `VERSION` constant still saying 0.10.78 after metadata advanced. It was synchronized to 0.10.79 before the final tests and package. The package source-hash guard rejected the interrupted build rather than shipping it.

## REA and limits

REA recorded the installed PE x86-64 artifact's identity in `rea-evidence.json`. Ghidra was not configured, and this investigation did not decompile renderer or Animation-CC behavior. The root cause is directly demonstrated by the actual transparent sprite frames and the official data-champion sprite binding documentation; metadata inspection is not presented as runtime proof.

Actual-game roster, draft/HUD, both facings, normal/KCM transitions, immunity/control, death/respawn and invisibility/fog checks remain pending after installation. Browser/asset/native tests are not live-game acceptance. No installed game or real career was changed by this work.

## Reproduce

```powershell
rtk proxy python tools/verify-minato-portrait.py
rtk proxy cargo +stable-x86_64-pc-windows-gnu test --release --manifest-path native/tfm2_custom_ai/Cargo.toml --lib
rtk proxy python tools/package-original-bodies.py --base 777204f --out docs/minato-portrait-fix --fixture-prefix minato-portrait-manager-
```

The final command compares two independent DLL builds and checks a copied temporary game fixture only. Rebuild only Minato's canonical layers with `integrate_original_bodies.minato_layers()`; a full unrelated-artwork regeneration is unnecessary.
