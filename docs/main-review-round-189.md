# Round 189: Contender 3.6, one shadow pool, two new passives, Shadow Grab and a Break knock-up (native 0.10.124, tfm2_custom 0.2.127)

**Reihan:**
> "his kit relys on his s2 for the damage, can we add 2 more passives?"

He chose **Footwork** and **Body Shots**, the "Stronger" option. He asked to remove the dodge-for-Flurry sacrifice and to merge Flurry charges and shadows into one pool of 10:
- the same 5-pip art, each pip worth 2 shadows;
- a dodge costs 1 shadow and a Flurry 2;
- one shadow returns every 4 s;
- each landed hit takes 0.1 s off the wait for every missing shadow.

Then:
> "add s3 as a long shadow grab and knock up ... using the same shadow as before but with a cd, at shadow break it knocks up the enemy 1s but with a cd on that enemy"

He put the grab in the third slot (the ult), with a 12 s cooldown and a 5 s per-enemy Break cooldown.

## Changes (`contender-3.6`)
- **One shadow pool:** 10 shadows shared by dodges (1 each) and Shadow Flurry (2 each).
  - Recharge is one shadow every 4 s.
  - Each landed non-Flurry hit (echo, counter, cross, basic, grab) shortens the wait by 6 ticks for each missing shadow, so he recovers faster the emptier he is.
  - **Removed:** separate Flurry charges, the recharge credits and the Lab's Exchange dodge (Shift+E).
- **Pips:** the 5 existing pips show `ceil(shadows / 2)`.
- **Every passive scales with Rhythm stacks (20 max):**
  - **Rhythm:** the target takes +1.5% damage from him per stack (+30% at 20).
  - **Footwork:** +1.5% attack speed and +0.5% move speed per stack on the enemy he's fighting.
  - **Body Shots:** the stacked enemy loses 1.5% Armour per stack (−30% at 20). This is a real debuff, so it helps his whole team.
  - **Shadow Break:** 4% of max HP + 20% Attack, and now a **1 s knock-up**, at most once every 5 s per enemy. A Break inside that window still deals its damage. Towers are never knocked up.
- **Shadow Grab replaces Shadow Link (the ult slot, still unlocked at level 5 by the engine):**
  - he throws the same shadow in a line up to 90,000 (24 ticks at full range), 8,000 wide;
  - the first enemy it touches (not a tower or a CC-immune unit) takes 20 + 60% Attack and gains a Rhythm stack;
  - it is dragged to 20,000 in front of him over 0.2 s, then knocked up for 1 s;
  - the cooldown is 12 s; the engine slot carries the same cooltime and range;
  - the grab doesn't refill shadows (Link did, but that would be too strong on a 12 s cooldown);
  - the area warning and the stun are gone.

## Checks
- **Native tests:** 1,419 passed, 19 ignored. New or rewritten tests:
  - the shared pool (a dodge costs 1, a Flurry 2, refused below 2);
  - the refund scales with missing shadows (none when full);
  - the pips show `ceil(n / 2)`;
  - Body Shots' Armour shred appears and disappears with the stacks;
  - the grab hits the first enemy in line, pulls, knocks up, misses cleanly at max range and respects its cooldown;
  - the Break knock-up's per-enemy cooldown;
  - every number comes from the `d::` constants.
- **Lab and Mod Manager:** 3 Lab transport tests and 15 Mod Manager tests pass.
- **Generators:** `generate-contender`, `generate-roster` and `generate-lab-targets` are current.
- **Builds:** the DLL contains 0.10.124, and `editor/contender.wasm` is rebuilt.
- **Untested here:** how often the game AI throws the grab and whether it lands can only be judged in a live match.
