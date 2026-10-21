# Gojo v3 release verification

Native **0.10.60**, JJK **0.1.31**, custom pack **0.2.61**. [Implementation and references](gojo-v3.md).

- 429 native tests passed, 4 existing ignored; 63 shared-core tests and seven migration tests passed.
- 33 browser checks, including every technique demonstration, current replay and frozen v1/v2 readers. Both facings/light-dark presentation samples pass.
- 58,350 complete native/WASM frames: 51,840 across 24 development and 12 holdout scenarios, plus 6,510 covering all ten legal demonstrations, including complete orb/Nuke flights. Independent sessions pass.
- 72 native Domain expansion frames, projectile/impact bounds, 530 animation renewal windows and 4,240 phase comparisons pass. Original body and legacy frame pixels match main. Generation is reproducible.
- Installed Gojo decoded art: **31,962,112 bytes (30.48 MiB)**; every atlas is at most 2048 pixels per side. Archived-v2 editor artwork is loaded only by its separate reader.
- Five Gojo core runs: median p95 **14.1 µs**, p99 **17.9 µs**. Magnus 128-piece p95: **1.001 ms**, below the 4 ms target. These exclude native host/rendering time. Fresh-browser load and canvas submission are recorded separately.
- Two independent release builds and packaged DLL have identical SHA-256. Mod Manager passes against an isolated full-package fixture.

| Thirty-second raw fixture, five seeds | Requested damage: v2 → v3 | Remaining Gojo HP: v2 → v3 |
|---|---:|---:|
| duel | 5561.2 → 5138.0 | 487.0 → 626.0 |
| team | 6018.4 → 6083.6 | 521.0 → 534.2 |
| farm | 4012.0 → 4222.0 | 11.0 → 101.6 |

The fixtures use explicit scene movement and raw shield bookkeeping, not engine armour, items or attributed match damage. Duels trade some requested damage for better remaining HP; the team fixture improves slightly in both. No unconditional buff or live-performance claim is made.

Packaged DLL SHA-256:

`3cbdaf717a7d6a3b7a1d0429f355714d30ef7ccb46431fe9998a52d04e59f27f`

WASM SHA-256:

`2a45e19d85deb765d96c215d69df67e9f6880caed0d4074fc41a00688f0feaeb`

Core/rules source fingerprint: `c233c4b15434e590ff2b16e8d97d896094af44da166d2c44395df2fa8f083891`.

**Live acceptance pending. Stock enemy-shot bending remains unsupported.** Original checkout DLL preserved: True. No game executable patch, Steam installation or career migration was performed. [Machine-readable delivery record](gojo-results/v3-delivery.json).
