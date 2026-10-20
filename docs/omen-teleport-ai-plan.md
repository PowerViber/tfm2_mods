# Omen — teleport playmaking (agreed 2026-10-20)

This specification is parked separately from the Steve/Herobrine implementation.

## Agreed behavior

- Paranoia and Shadow Step retain contextual S2 selection but gain independent seven-second base cooldowns. This is an intentional utility buff enabling confirmed blind → Step sequences.
- Preserve damage, teleport ranges, channel times, smoke/economy rules and the existing 80-second ultimate cooldown. Weapon purchasing is outside this pass.
- Evaluate shoot → reposition, confirmed blind → Step, escaping visible incoming control, remote ganks, tower defence, retreat interception and major-objective reinforcement.
- Score useful firing access, visible enemy commitments, confirmed control, channel survival and a post-arrival action. Do not assume positional surprise against automatic targeting.
- Replace broad health-percentage exclusions with action-specific survival assessment. A cross-map ultimate plan must not depend on a nearby enemy causing the stock planner to cast.
- Keep a bounded continuation after landing; invalidate it when visibility, targets, geometry or fight conditions change. Do not retarget a committed destination.
- Native rules own accepted casts and cooldowns. Use inert action shells and the existing movement-only shared AI hook; never inject or hold stock attack/cast inputs. Remove synthetic enemy binds from the migrated controller.
- Use visible information only. Do not reward fake teleports unless opponents actually respond through existing observable mechanics. Preserve shade counterplay and cancellation.

## References reviewed

- [OpiumOmen — The ULTIMATE Aggressive Omen Guide](https://www.youtube.com/watch?v=bHUC3oKbWNs): retake examples from 0:27; repositioning from 2:00; information/audio discussion from 3:34. Selected frames/sections were reviewed, not every upload.
- [BcJ's tournament Step counterplay on VALORANT's channel](https://www.twitch.tv/valorant/clip/RealBloodyRamenTF2John-ueBQLakIz7JCqCGz): use visible warning timing to evaluate a legal escape; grant no new invulnerability.
- [Dignitas Omen guide](https://dignitas.gg/articles/stepping-out-of-the-shadows-taking-a-look-at-omen): coordinated entry, follow-up and rotations. Historical Valorant numeric values are not mod balance defaults.

## Verification and delivery

Record eligible opportunities, execution/rejection reasons, time to a useful post-landing attack, survival and fight/objective influence alongside teleport counts. Cover remote casts without local enemies, control/visibility loss, interrupted channels, shade spawn failure, independent owners, deterministic replay and native/editor parity. Publish a separate PR; no Omen runtime changes belong to the Steve PR.
