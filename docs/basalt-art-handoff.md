# Basalt art handoff (for Codex)

Basalt, the Wallbreaker: an original stone-shaper brawler with a **long stone-linked whip**. His palette is basalt grey and ember orange, with dust. The kit is in `docs/basalt-guide.md`; the numbers are in `rules/basalt.json`.

## What exists now (placeholders)
- **Body:** Grimhald's sheet (`tools/generate-basalt.py` falls back to it while `mods/tfm2_custom/champions/tfm2_custom_basalt#sheet.png` is missing).
- **Effects:** every `bs_*` name is bound to an existing Grimhald or Kicker effect (`PLACEHOLDER` in the generator). Once `mods/tfm2_custom/vfx/basalt#sheet.png` exists, each name binds to the same tag on his own sheet.

## Requested
**Body:** idle, run, the whip crack (basic), a raise (stomp), a punch, a throw, the whip snare and swing, a jump and an air pose, the ult charge (holding a giant rock overhead, 3 s loop), and the ult punch.

**VFX tags:**

| Tag | Use |
|---|---|
| `block` | one stone block of a wall; a row of them draws a wall (redrawn every 6 to 28 ticks) |
| `rise` | a wall erupting |
| `break` | a wall shattering, or a chunk ripped off terrain |
| `dust` | a wall crumbling away, or a wall kick |
| `slam` | a heavy impact (thrown wall, the clap, the boulder) |
| `wall_slam` | an enemy slammed into a wall |
| `leap` / `land` | jump take-off and landing |
| `dash` | Air Dash, Whip Pull and whip snares |
| `rock` | a meteor or rock hit |

**Wanted later:** the boulder in flight, the meteor shower, and the whip itself (drawn between him and its target during pulls and swings).
