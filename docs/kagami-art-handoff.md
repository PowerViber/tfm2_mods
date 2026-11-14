# Kagami art handoff (for Codex)

Kagami, the Mirror Priestess: a shrine priestess whose clone is her mirror image. She's a support. Her colours are moonlit silver, lavender and white, with a mirror-glass shimmer. The kit is in `docs/kagami-guide.md`; the numbers are in `rules/kagami.json`.

## What exists now (placeholders)
- **Body:** Thalassa's sheet (`tools/generate-kagami.py` falls back to it while `mods/tfm2_custom/champions/tfm2_custom_kagami#sheet.png` is missing).
- **Clone body:** the base `priest` unit (`CLONE_BODY` in `native/tfm2_custom_ai/src/kagami.rs`). It's a summoned unit the core glides around, so it never plays a walk.
- **Effects:** every `kg_*` name is bound to a Thalassa effect (`PLACEHOLDER` in the generator). Once `mods/tfm2_custom/vfx/kagami#sheet.png` exists, the generator binds each name to the same-named tag on her own sheet instead.

## Requested
**Body** (`tfm2_custom_kagami#sheet.png` / `#anim.fanim`): idle, run, attack (a mirror-shard flick), and a short cast pose.

**Clone:** either a translucent, mirrored tint of her body, or a body drawn as effects (the Thalassa twin pattern: `twin_idle_*` / `twin_run_*` by heading). If you choose effects, tell me and I'll hide the summoned unit and lease the frames.

**VFX sheet** (`mods/tfm2_custom/vfx/kagami#sheet.png` / `#anim.fanim`), one tag per effect:

| Tag | Use | Length |
|---|---|---|
| `shard` | her basic projectile (looping) | — |
| `shard_hit` | the clone's shard landing | 12 ticks |
| `thread` | the 0.5 s mirror thread between the two swap sides (drawn at the midpoint) | 30 ticks |
| `swap` | a flash at both ends of a swap | 24 ticks |
| `stun_burst` | the 50,000-radius stun where the clone lands | 30 ticks |
| `clone_in` | the clone appearing | 24 ticks |
| `heal` | Moonlit Mend on a target | 30 ticks |
| `charm` | Kindred Charm landing | 24 ticks |
| `angel` | Guardian Angel: wings flaring, with a knockback ring (50,000) | 36 ticks |
| `seal` | the seal cocoon (a cracked-mirror shell) | 150 ticks |
| `revive` | the shell shattering on revive | 30 ticks |

**Buffs:**
- `sealed`: on a sealed unit; it may be hidden by the banish, so the `seal` effect carries the look;
- `charm`: a small lavender sparkle;
- `revived`: a mirror-glass shimmer for 4 s.
