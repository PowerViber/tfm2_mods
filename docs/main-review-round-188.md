# Round 188: Contender 3.5, Merge becomes a buff and Flurry becomes a real skill (native 0.10.123, tfm2_custom 0.2.126)

**Reihan:**
> "make S1 to be a buff for him, movespeed attspeed attdmg, in merged cant be CCed, if an enemy does damage to contender at shadow, it slows them"

On the Flurry: "i want it to be like a skill". He chose a whole Flurry at about 2.5× Attack, the "Medium" S1 buff, and keeping the cross. He also asked why Contender stays aggressive when low.

## What #103 (Codex, 3.4) had changed
- **Flurry:** 30 punches in 1.5 s at 2 + 2% Attack.
- **Reappearance:** in place; the last-hit teleport is removed.
- **Packaging:** the DLL is shipped with new versions, and the tests read the rules (good).
- **The problem:** a whole Flurry was about 150 at level 9, less than one basic attack.

## Changes (`contender-3.5`)
- **S1 Shadow Merge is a self-buff for 6 s:**
  - +20% move speed (was +10%);
  - **+30% attack speed** and **+15% Attack**;
  - **can't be crowd-controlled**;
  - **any enemy (not a tower) that damages him while merged is slowed 30% for 1.5 s**;
  - the cross is kept (12 + 15% Attack every second within 50,000).
- **Shadow Flurry:** each punch is 2 + **7%** Attack (was 2 + 2%), so a whole Flurry is about **2.5× Attack** (≈400 at level 9).
- **Less pointless aggression:**
  - his passive tells the input AI each tick whether a Flurry press can be honoured;
  - with no charge left (or mid-Flurry), the game AI's Flurry press becomes a basic attack instead of a wasted cast. This is the existing press-gating scheme already used by Aegis Zero and Isliid.
- **What this doesn't change:** the game's own AI still decides when to fight or retreat. If he still dives at low HP, the cause is the game AI's choices, and it needs a live check.

## Checks
- **Native tests:** 1,418 passed, 19 ignored. New tests:
  - the Merge buff values and CC immunity;
  - attackers slowed only while he's merged;
  - a Flurry press with no charge isn't wanted;
  - a full Flurry is 2.2–3.2× Attack.
- **Lab and Mod Manager:** 3 Lab transport tests and 15 Mod Manager tests pass.
- **Generators:** current.
- **Builds:** the DLL contains 0.10.123, and the wasm is rebuilt.
