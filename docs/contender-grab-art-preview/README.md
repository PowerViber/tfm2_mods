# Shadow Grab artwork

This review follows the Shadow Grab cast from [Contender 3.6, PR #105](https://github.com/PowerViber/tfm2_mods/pull/105). [PR #106](https://github.com/PowerViber/tfm2_mods/pull/106) restored 3.5 while the artwork was prepared.

The preview pairs the existing flying shadow with four new violet effects. The reaching hand opens during flight. The latch identifies the single target, the ribbon tracks the pull, and the upward burst marks the knock-up. Both facings and a clean miss are available at 1× and 3×.

- [Transparent source sheet](shadow-grab-source.png)
- [Game-size comparison](shadow-grab-review.png)
- [Recorded 3.6 traces](shadow-grab-trace.json) from WASM at merge commit `c522b41`
- [Animated review page](../../editor/contender-grab-review.html)

Run `node tools/serve-contender-shadow-review.cjs 7319` from the repository root and open the [animated review](http://localhost:7319/editor/contender-grab-review.html).

The recording keeps the 3.6 cast preview playable on reverted main. Artwork approval is pending before the game-pack integration.
