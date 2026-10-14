# Vesper, the Rift Gauntlets: catalogue

Generated from `Claude outputs/vesper/catalogue.json`. Everything here is available at every mastery rank; rank only changes how well the athlete plans with it.

## Gauntlet moves

| Move | Orbs | What happens |
|---|---|---|
| Rift Swipe | 2 | Swipes the air like pulling a mask off and tears the dimension: a Slash rift opens along the swipe, everyone on the line takes the chosen dimension's swipe effect, enemies take 20 + 40% AD |
| Shatter | 0 | Crushes every pure orb she holds: every open rift rips wide, banishes and damages the enemies inside (never allies). Past the stress cap the left gauntlet breaks |
| Overload | 0 | Forces an extra orb into a 6th to 9th socket: +8 stress now and +1 per 30 ticks while it is held charged |
| Vent | 0 | Drops every overload socket before it cracks the gauntlet |
| Awaken | 0 | At 100% the Hexcore awakens for 6 s: hexorbs at full power, pure + hex orbs fuse, her rifts cannot be broken |
| Overclock | 1 | While awakened, feeds a pure orb into the Hexcore: +15 corruption, +5% hexorb power. At 100 corruption the right gauntlet breaks |

## Hexcore tech modules

Each athlete's Hexcore carries 6 of these 8 (derived from the athlete, editable in the editor).

| Module | Effect | a | b |
|---|---|---|---|
| Plating | Shields the lowest-health ally near the rift for a per mille of max HP | 48 | 180 |
| Coil | Allies nearby gain +a% move speed for b ticks | 25 | 90 |
| Stasis | Enemies nearby are slowed a% for b ticks | 30 | 60 |
| Magnet | Enemies nearby are pulled a% toward the point | 35 | 0 |
| Mender | Allies nearby heal a per mille of max HP | 24 | 0 |
| Scanner | Enemies nearby take +a% damage for b ticks | 10 | 90 |
| Spark Arc | Arcs a + b% AD to the two nearest enemies | 16 | 20 |
| Anchor Bolt | The nearest enemy is rooted a ticks | 19 | 0 |

| Hexcore completion | Hexorb slots |
|---|---|
| 70% | 2 |
| 75% | 3 |
| 80% | 4 |
| 85% | 5 |
| 90% | 6 |

## Rift types

| Rift | Orbs | Life (ticks) | Size | What the shape does |
|---|---|---|---|---|
| Tear | 1 | 300 | r15300 | Round rift: chain node and portal |
| Slash | 1 | 240 | r5100, length 51000 | A 51000-unit line: enemies crossing it are thrown back; fires fan volleys |
| Sinkhole | 1 | 240 | r23800 | Ground ellipse: slows enemies 30% and swallows anyone inside for 60 ticks |
| Sky Rift | 2 | 180 | r20400 | Opens overhead: rains volleys and drops allies from above |
| Mirror Pane | 1 | 300 | r5100, length 34000 | A glass segment that bounces shots and divers |
| Twin Rift | 2 | 300 | r13600 | Two rifts born linked: instant portal pair, swaps fighters |
| Wandering Rift | 1 | 240 | r13600 | Moves 480 units per tick along the path Vesper draws |
| Anchor Rift | 1 | 360 | r11900 | Stuck to a unit (ally, enemy) or tethered to Vesper at a safe spot |
| Seed Rift | 1 | 480 | r11900 | Hidden for 120 ticks, then a trap for the first enemy to step on it |
| Maw | 3 | 90 | r34000 | Huge and brief: swallows everyone inside, then caves in by itself |

## Techniques

| Technique | Group | Rifts | Stress | Per-target cooldown | What happens |
|---|---|---|---|---|---|
| Chain Fire | offense | Tear, Sky Rift, Mirror Pane, Twin Rift, Anchor Rift, Wandering Rift | 0 | 0 | While a chain is live, every rift punch travels through it and comes out of the last rift |
| Crossfire | offense | Tear | 8 | 0 | One rift punch splits out of up to 3 rifts at 3 targets, at 36% each, for 180 ticks |
| Bank Shot | offense | Mirror Pane | 5 | 0 | A Mirror Pane in the chain focuses the next link (x1.2) and lets punches pass walls |
| Rift Rain | offense | Sky Rift | 11 | 0 | 6 packets fall over the Sky Rift during 60 ticks: 20 + 20% AD each |
| Slash Volley | offense | Slash | 9 | 0 | 5 packets fan out along the Slash: 16 + 24% AD each |
| Implode | offense | Tear, Sinkhole, Sky Rift, Twin Rift, Seed Rift, Anchor Rift, Wandering Rift, Maw | 8 | 0 | Close a rift early: enemies are dragged to its centre and take 16 + 32% AD plus its dimension twice |
| Domino | offense | Tear, Twin Rift | 6 | 0 | Implode every linked rift in turn, 15 ticks apart |
| Overload Punch | offense | Tear, Twin Rift, Sky Rift, Anchor Rift | 0 | 0 | Spends stress: the next rift punch gains +1% per point spent (up to 60) |
| Rift Step | engage | Tear, Twin Rift | 8 | 240 | Vesper steps into a rift within 18000 and out of another |
| Ally Gate | engage | Tear, Twin Rift | 11 | 180 | Allies touching the entrance come out of the exit |
| Sky Drop | engage | Sky Rift | 14 | 180 | A Gate exit from above: the ally lands and stuns enemies within 17000 for 19 ticks |
| Kidnap | engage | Tear, Twin Rift | 14 | 600 | Blink in front of an enemy within 90000 and arm a stun field for 0.5 s; if they are still in it, both of you drop through to the kidnap rift (20 s cooldown) |
| Sinkhole Trap | engage | Sinkhole | 9 | 600 | Enemies inside for 60 ticks fall through to the chosen exit |
| Swap | engage | Twin Rift | 12 | 480 | The fighters nearest each twin trade places |
| Fling | engage | Wandering Rift | 11 | 240 | Enemies caught are carried along the path and dropped at its end |
| Hook | engage | Anchor Rift | 12 | 600 | Mark an enemy; reel them through to the exit later. Enemies tearing it off free them |
| Phase Out | defense | Tear, Twin Rift | 9 | 240 | Vesper hides in a rift for up to 45 ticks, then leaves through another |
| Rift Curtain | defense | Slash, Tear | 9 | 240 | A Slash, or three Tears in a line: enemies crossing are thrown back |
| Reflect Wall | defense | Mirror Pane | 8 | 240 | Divers bounce off a Mirror Pane toward where they came from |
| Rescue | defense | Tear, Twin Rift | 11 | 900 | A rift under an ally below 25% HP drops them at a safe rift |
| Pocket | defense | Maw, Tear | 9 | 720 | Hide an ally inside a rift for 60 ticks, then return them |
| Lifeline | defense | Anchor Rift | 6 | 1200 | An Anchor Rift tethered to Vesper at a safe spot: below 25% HP she is pulled to it |
| Escort | team | Anchor Rift | 6 | 0 | An Anchor Rift on an ally: rift punches come out of the diving ally |
| Rotation Gate | team | Twin Rift | 15 | 180 | A long Twin gate that moves allies across the map toward the next fight |
| Seed Trap | team | Seed Rift | 8 | 600 | A hidden Seed on a predicted path: opens with its dimension and swallows the victim |
| Sweep | team | Wandering Rift | 9 | 0 | Drag a dimension across a whole line of enemies |
| Infuse | team | Tear, Sinkhole, Sky Rift, Twin Rift, Anchor Rift, Seed Rift, Wandering Rift, Maw, Slash, Mirror Pane | 6 | 0 | Spends a hexorb on an open rift: its tech module pulses from that rift for the rest of its life |
| Stitch | team | Tear | 15 | 0 | Sew two Tears into one larger rift holding both orb dimensions and their bonus |

## Cooldowns

Every dimension has its own cooldown once she casts it, and its family rests 2 s. Both gauntlets share both. The rifts of one cast (both ends of a gate, every node one action opens) and a fused punch count as one cast. Pulses and fists already in flight finish normally.

| Class | Cooldown | Main effects |
|---|---|---|
| Light utility | 10 s | SLOW, HASTE, STATUP, STATDOWN, RANGE, MARK, SCRAMBLE, CDUP, CDDOWN, BADR, HEALCUT, VENT |
| Damage | 14 s | AMPLIFY, DOT, TRUE, EXECUTE, ECHO, WILDECHO, PRISM, SPLASH, RICOCHET, LENS, DRAIN, ENTROPY, SHATTER, COINFLIP, DICE, UNSTABLE, KALEIDO, CHAIN, REFLECTDMG, VAMP, MOMENTUM, PRESSURE, EXPOSE |
| Strong control / support | 20 s | BANISH, STUN, ROOT, SILENCE, DISARM, STATICBURST, ENCASE, WALLSLAM, HEAL, SHIELD, SHELTER, UNDYING, CCIMMUNE, CLEANSE, ALLYINVIS, SELFINVIS, GHOST, SIPHON, TITHE, RECALLHP, WALL, CHILL |
| Teleport / displacement | 24 s | PULL, KNOCK, SHOVE, STORM, GRAB, SWAPSPACE, SELFSWAP, BLINK, FOLD, REWIND, TOEND, REFLECT, TETHER, WELL, REPEL |
| Signature (one spectacular dimension per family) | 36 s | |

## Dimensions: rift zone, punch and swipe

Any orb opens any of the 150 dimensions. The rift zone, the rift punch and the rift swipe each have their own behaviour (450 in all). The numbers carry the 0.10.48 scale: damage, hard control, heals and shields x0.8, radii x0.85, crowd-control immunity and Undying x0.75.

| ID | Dimension | Family | Cooldown | Rift zone | Punch (hole) | Swipe (tear line) |
|---|---|---|---|---|---|---|
| D00 | Void | Control | 36 s (signature) | **BANISH** a=48 | **BANISH** a=24 + **EXECUTE** a=12 b=30 | **BANISH** a=14 + **TOEND** |
| D01 | Silent Realm | Control | 20 s | **SILENCE** a=48 | **SILENCE** a=72 + **CDUP** a=30 b=180 | **SILENCE** a=36 + **DISARM** a=12 |
| D02 | Unarmed World | Control | 20 s | **DISARM** a=48 | **DISARM** a=72 + **STATDOWN** b=20 c=120 | **DISARM** a=36 + **SLOW** b=25 c=60 |
| D03 | Frozen Moment | Control | 20 s | **STUN** a=24 | **STUN** a=36 + **SHATTER** | **STUN** a=18 + **SLOW** a=40 c=60 |
| D04 | Anchor | Control | 20 s | **ROOT** a=48 | **ROOT** a=72 + **AMPLIFY** a=8 b=60 | **ROOT** a=36 + **PULL** a=30 |
| D05 | Heavy World | Control | 10 s | **SLOW** a=35 b=20 c=40 | **SLOW** a=52 b=30 c=60 + **ROOT** a=12 | **SLOW** a=26 b=15 c=30 + **CDUP** a=20 b=120 |
| D06 | Leaden Sky | Control | 10 s | **SLOW** a=50 c=90 | **SLOW** a=75 c=135 + **STATDOWN** a=1 b=15 c=90 | **SLOW** a=37 c=67 + **STUN** a=6 |
| D07 | Lull | Control | 20 s | **STUN** a=14 | **STUN** a=22 + **SILENCE** a=24 | **STUN** a=10 + **BADR** a=15 b=60 |
| D08 | Gloom Cage | Control | 20 s | **ROOT** a=72 | **ROOT** a=108 + **DOT** a=8 b=120 | **ROOT** a=54 + **AMPLIFY** a=6 b=60 |
| D09 | Tar Pit | Control | 10 s | **SLOW** a=25 b=10 c=240 | **SLOW** a=37 b=15 c=360 + **HEALCUT** a=25 b=120 | **SLOW** a=18 b=7 c=180 + **DOT** a=6 b=180 |
| D10 | Static Realm | Ruin | 14 s | **AMPLIFY** a=12 b=40 | **AMPLIFY** a=20 b=60 + **RICOCHET** a=24 | **AMPLIFY** a=10 b=30 + **MARK** a=32 b=180 |
| D11 | Ember | Ruin | 14 s | **DOT** a=16 b=180 | **DOT** a=24 b=270 + **CHAIN** a=25500 b=100 | **DOT** a=12 b=135 + **SLOW** a=15 c=60 |
| D12 | True Rift | Ruin | 14 s | **TRUE** a=12 | **TRUE** a=18 + **SHATTER** | **TRUE** a=9 + **AMPLIFY** a=5 b=40 |
| D13 | Shatter | Ruin | 14 s | **SHATTER** | **SHATTER** + **STATDOWN** a=1 b=20 c=120 | **SHATTER** + **HEALCUT** a=15 b=60 |
| D14 | Acid Rain | Ruin | 14 s | **DOT** a=8 b=360 | **DOT** a=12 b=540 + **STATDOWN** a=2 b=20 c=180 | **DOT** a=6 b=270 + **SPLASH** a=16 b=17000 |
| D15 | Fracture | Ruin | 14 s | **AMPLIFY** a=20 b=60 | **AMPLIFY** a=30 b=90 + **EXECUTE** a=20 b=35 | **AMPLIFY** a=14 b=45 + **SHATTER** |
| D16 | Brittle | Ruin | 10 s | **STATDOWN** a=1 b=25 c=120 | **STATDOWN** a=1 b=37 c=180 + **SPLASH** a=24 b=21250 | **STATDOWN** a=1 b=18 c=90 + **SLOW** a=15 b=15 c=60 |
| D17 | Hex Ward | Ruin | 10 s | **STATDOWN** a=2 b=25 c=120 | **STATDOWN** a=2 b=37 c=180 + **SILENCE** a=16 | **STATDOWN** a=2 b=18 c=90 + **TRUE** a=8 |
| D18 | Cinder | Ruin | 14 s | **DOT** a=24 b=120 | **DOT** a=36 b=180 + **SPLASH** a=20 b=17000 | **DOT** a=18 b=90 + **HEALCUT** a=25 b=120 |
| D19 | Razor Light | Ruin | 36 s (signature) | **TRUE** a=20 | **TRUE** a=30 + **RICOCHET** a=32 | **TRUE** a=14 + **LENS** a=12 |
| D20 | Mirror Realm | Motion | 24 s | **REFLECT** | **REFLECT** + **STUN** a=10 | **REFLECT** + **SLOW** a=20 c=40 |
| D21 | Drift | Motion | 24 s | **PULL** a=35 | **PULL** a=52 + **SLOW** a=30 c=40 | **PULL** a=26 + **AMPLIFY** a=6 b=40 |
| D22 | Gale | Motion | 24 s | **KNOCK** a=20000 | **KNOCK** a=25000 + **WALLSLAM** a=24 | **KNOCK** a=15000 + **SLOW** a=20 c=30 |
| D23 | Crosswind | Motion | 24 s | **SHOVE** a=15000 | **SHOVE** a=22500 + **DISARM** a=16 | **SHOVE** a=11250 + **SLOW** a=25 c=40 |
| D24 | Tether | Motion | 24 s | **TETHER** | **GRAB** a=24000 + **ROOT** a=16 | **PULL** a=50 + **SLOW** a=20 c=60 |
| D25 | Swap Space | Motion | 36 s (signature) | **SWAPSPACE** | **SELFSWAP** + **STUN** a=10 | **SWAPSPACE** + **SLOW** a=30 c=40 |
| D26 | Undertow | Motion | 24 s | **PULL** a=60 | **PULL** a=90 + **ROOT** a=16 | **PULL** a=45 + **STUN** a=8 |
| D27 | Repulse | Motion | 24 s | **KNOCK** a=32000 | **KNOCK** a=48000 + **STATDOWN** a=1 b=15 c=60 | **KNOCK** a=24000 + **SHELTER** a=12 b=60 |
| D28 | Whirl | Motion | 24 s | **SHOVE** a=9000 | **SHOVE** a=13500 + **SPLASH** a=16 b=15300 | **SHOVE** a=6750 + **SILENCE** a=12 |
| D29 | Slipstream | Motion | 24 s | **GRAB** a=20000 | **GRAB** a=30000 + **AMPLIFY** a=12 b=40 | **GRAB** a=15000 + **HASTE** a=20 c=90 |
| D30 | Shelter | Haven | 20 s | **SHELTER** a=12 b=40 | **SHELTER** a=18 b=60 + **SLOW** a=15 c=40 | **SHELTER** a=9 b=30 + **CLEANSE** |
| D31 | Bloom | Haven | 20 s | **HEAL** a=16 b=40 | **HEAL** a=24 b=60 + **HEALCUT** a=20 b=60 | **HEAL** a=12 b=30 + **HASTE** a=15 c=60 |
| D32 | Bulwark | Haven | 20 s | **SHIELD** a=64 b=180 | **SHIELD** a=96 b=270 + **DISARM** a=12 | **SHIELD** a=48 b=135 + **BADR** a=20 b=90 |
| D33 | Last Breath | Haven | 36 s (signature) | **UNDYING** a=45 | **UNDYING** a=68 + **SLOW** a=20 b=20 c=40 | **UNDYING** a=34 + **SHELTER** a=16 b=40 |
| D34 | Sanctum | Haven | 20 s | **SHELTER** a=20 b=40 | **SHELTER** a=30 b=60 + **KNOCK** a=12000 | **SHELTER** a=14 b=30 + **CCIMMUNE** a=23 |
| D35 | Spring | Haven | 20 s | **HEAL** a=32 | **HEAL** a=48 + **STUN** a=6 | **HEAL** a=24 + **HASTE** a=25 c=40 |
| D36 | Aegis | Haven | 20 s | **SHIELD** a=120 b=60 | **SHIELD** a=180 b=90 + **KNOCK** a=18000 | **SHIELD** a=90 b=45 + **REFLECTDMG** a=8 b=60 |
| D37 | Mend | Haven | 20 s | **HEAL** a=8 b=360 | **HEAL** a=12 b=540 + **DOT** a=6 b=120 | **HEAL** a=6 b=270 + **CDDOWN** a=10 b=120 |
| D38 | Ward | Haven | 10 s | **STATUP** a=2 b=25 c=120 | **STATUP** a=2 b=37 c=180 + **STATDOWN** a=3 b=20 c=120 | **STATUP** a=2 b=18 c=90 + **SHIELD** a=32 b=120 |
| D39 | Iron Skin | Haven | 10 s | **STATUP** a=1 b=25 c=120 | **STATUP** a=1 b=37 c=180 + **STATDOWN** a=1 b=20 c=120 | **STATUP** a=1 b=18 c=90 + **SHELTER** a=8 b=90 |
| D40 | Folded Map | Passage | 36 s (signature) | **FOLD** | **BLINK** a=30000 + **HASTE** a=25 c=90 | **GHOST** a=120 + **HASTE** a=30 c=120 |
| D41 | Ghost Plane | Passage | 20 s | **GHOST** a=180 | **GHOST** a=270 + **BLINK** a=20000 | **GHOST** a=135 + **HASTE** a=20 c=120 |
| D42 | Far Shore | Passage | 10 s | **RANGE** a=15000 b=40 | **RANGE** a=22500 b=60 + **KNOCK** a=15000 | **RANGE** a=11250 b=30 + **HASTE** b=15 c=120 |
| D43 | Swift Plane | Passage | 10 s | **HASTE** a=30 b=25 c=180 | **HASTE** a=45 b=37 c=270 + **SLOW** a=25 c=60 | **HASTE** a=22 b=18 c=135 + **GHOST** a=60 |
| D44 | Tailwind | Passage | 10 s | **HASTE** a=45 c=90 | **HASTE** a=67 c=135 + **SHOVE** a=12000 | **HASTE** a=33 c=67 + **CLEANSE** |
| D45 | Long Reach | Passage | 10 s | **RANGE** a=25000 b=60 | **RANGE** a=37500 b=90 + **GRAB** a=15000 | **RANGE** a=18750 b=45 + **STATUP** b=10 c=120 |
| D46 | Stride | Passage | 10 s | **HASTE** a=20 c=360 | **HASTE** a=30 c=540 + **BLINK** a=15000 | **HASTE** a=15 c=270 + **SHELTER** a=8 b=60 |
| D47 | Skyway | Passage | 20 s | **GHOST** a=90 | **GHOST** a=135 + **KNOCK** a=20000 | **GHOST** a=67 + **ALLYINVIS** a=30 |
| D48 | Sprint | Passage | 10 s | **HASTE** a=60 c=40 | **HASTE** a=90 c=60 + **BLINK** a=25000 | **HASTE** a=45 c=30 + **BADR** a=15 b=60 |
| D49 | Courier Wind | Passage | 10 s | **HASTE** a=15 b=15 c=600 | **HASTE** a=22 b=22 c=900 + **CDDOWN** a=10 b=120 | **HASTE** a=11 b=11 c=450 + **RANGE** a=10000 b=120 |
| D50 | Rewind | Time | 36 s (signature) | **REWIND** a=120 | **REWIND** a=180 + **SILENCE** a=16 | **REWIND** a=120 + **RECALLHP** a=120 b=120 |
| D51 | Echo | Time | 14 s | **ECHO** a=32 b=60 | **ECHO** a=48 b=90 + **CDUP** a=15 b=120 | **ECHO** a=24 b=45 + **SLOW** a=15 c=60 |
| D52 | Haste Clock | Time | 10 s | **HASTE** b=35 c=120 | **HASTE** b=52 c=180 + **CDUP** a=20 b=120 | **HASTE** b=26 c=90 + **CDDOWN** a=15 b=180 |
| D53 | Slow Clock | Time | 10 s | **SLOW** b=35 c=120 | **SLOW** b=52 c=180 + **CDUP** a=30 b=120 | **SLOW** b=26 c=90 + **DISARM** a=8 |
| D54 | Hourglass | Time | 10 s | **CDUP** a=50 b=180 | **CDUP** a=75 b=270 + **SILENCE** a=12 | **CDUP** a=37 b=135 + **SLOW** a=20 c=90 |
| D55 | Quickening | Time | 10 s | **CDDOWN** a=25 b=180 | **CDDOWN** a=37 b=270 + **CDUP** a=25 b=120 | **CDDOWN** a=18 b=135 + **HASTE** a=15 c=90 |
| D56 | Afterimage | Time | 14 s | **ECHO** a=48 b=30 | **ECHO** a=72 b=45 + **SELFINVIS** a=20 | **ECHO** a=36 b=22 + **SHOVE** a=6000 |
| D57 | Deja Vu | Time | 24 s | **REWIND** a=60 | **REWIND** a=90 + **STUN** a=12 | **REWIND** a=45 + **RECALLHP** a=60 b=64 |
| D58 | Long Second | Time | 10 s | **SLOW** a=20 b=20 c=180 | **SLOW** a=30 b=30 c=270 + **ROOT** a=8 | **SLOW** a=15 b=15 c=135 + **CDUP** a=25 b=90 |
| D59 | Prime Time | Time | 10 s | **STATUP** b=15 c=180 | **STATUP** b=22 c=270 + **VAMP** a=12 b=90 | **STATUP** b=11 c=135 + **CDDOWN** a=15 b=120 |
| D60 | Hidden Side | Veil | 20 s | **SELFINVIS** a=40 | **SELFINVIS** a=60 + **BLINK** a=18000 | **SELFINVIS** a=30 + **SLOW** a=20 c=40 |
| D61 | Blind Fog | Veil | 10 s | **STATDOWN** b=25 c=60 | **STATDOWN** b=37 c=90 + **DISARM** a=20 | **STATDOWN** b=18 c=45 + **ALLYINVIS** a=20 |
| D62 | Clear Sky | Veil | 20 s | **CLEANSE** | **CLEANSE** + **STATDOWN** a=2 b=15 c=90 | **CLEANSE** + **CCIMMUNE** a=15 |
| D63 | Mist Veil | Veil | 20 s | **ALLYINVIS** a=45 | **ALLYINVIS** a=67 + **SLOW** a=25 c=30 | **ALLYINVIS** a=33 + **HASTE** a=20 c=60 |
| D64 | Glare | Veil | 10 s | **SLOW** b=25 c=60 | **SLOW** b=37 c=90 + **DISARM** a=12 | **SLOW** b=18 c=45 + **STATDOWN** b=15 c=60 |
| D65 | Smoke Screen | Veil | 10 s | **BADR** a=30 b=60 | **BADR** a=45 b=90 + **SHOVE** a=8000 | **BADR** a=22 b=45 + **SELFINVIS** a=20 |
| D66 | Dazzle | Veil | 20 s | **DISARM** a=24 | **DISARM** a=36 + **AMPLIFY** a=10 b=40 | **DISARM** a=18 + **SLOW** a=15 c=30 |
| D67 | Eclipse | Veil | 10 s | **STATDOWN** a=3 b=25 c=120 | **STATDOWN** a=3 b=37 c=180 + **SILENCE** a=20 | **STATDOWN** a=3 b=18 c=90 + **HEALCUT** a=20 b=90 |
| D68 | Nightfall | Veil | 36 s (signature) | **SELFINVIS** a=90 | **SELFINVIS** a=135 + **EXECUTE** a=24 b=30 | **SELFINVIS** a=67 + **DISARM** a=16 |
| D69 | Lantern | Veil | 10 s | **STATUP** a=4 b=10 c=180 | **STATUP** a=4 b=15 c=270 + **MARK** a=24 b=180 | **STATUP** a=4 b=7 c=135 + **LENS** a=8 |
| D70 | Glass Wall | Matter | 36 s (signature) | **WALL** | **ENCASE** a=36 b=30 + **MARK** a=16 b=120 | **WALL** + **SLOW** a=25 c=60 |
| D71 | Prism | Matter | 14 s | **PRISM** a=3 b=32 | **PRISM** a=3 b=48 + **STATDOWN** a=1 b=10 c=60 | **PRISM** a=3 b=24 + **TRUE** a=6 |
| D72 | Mirror Shard | Matter | 14 s | **REFLECTDMG** a=12 b=60 | **REFLECTDMG** a=18 b=90 + **KNOCK** a=10000 | **REFLECTDMG** a=9 b=45 + **SHIELD** a=24 b=90 |
| D73 | Thorn Glass | Matter | 14 s | **REFLECTDMG** a=20 b=40 | **REFLECTDMG** a=30 b=60 + **DOT** a=8 b=60 | **REFLECTDMG** a=14 b=30 + **SLOW** a=20 c=40 |
| D74 | Bedrock | Matter | 20 s | **CCIMMUNE** a=23 | **CCIMMUNE** a=34 + **ROOT** a=12 | **CCIMMUNE** a=17 + **SHELTER** a=12 b=60 |
| D75 | Quartz Lens | Matter | 14 s | **LENS** a=16 | **LENS** a=24 + **SPLASH** a=16 b=12750 | **LENS** a=12 + **AMPLIFY** a=8 b=40 |
| D76 | Splinter | Matter | 14 s | **PRISM** a=2 b=48 | **PRISM** a=2 b=72 + **DOT** a=5 b=120 | **PRISM** a=2 b=36 + **SHATTER** |
| D77 | Obsidian | Matter | 10 s | **STATDOWN** a=1 b=35 c=60 | **STATDOWN** a=1 b=52 c=90 + **KNOCK** a=14000 | **STATDOWN** a=1 b=26 c=45 + **ROOT** a=12 |
| D78 | Crystal Skin | Matter | 20 s | **SHIELD** a=48 b=240 | **SHIELD** a=72 b=360 + **ENCASE** a=16 b=20 | **SHIELD** a=36 b=180 + **REFLECTDMG** a=8 b=90 |
| D79 | Keystone | Matter | 20 s | **CCIMMUNE** a=45 | **CCIMMUNE** a=68 + **STUN** a=11 | **CCIMMUNE** a=34 + **CLEANSE** |
| D80 | Hungry | Hunger | 10 s | **HEALCUT** a=50 b=40 | **HEALCUT** a=75 b=60 + **DOT** a=10 b=120 | **HEALCUT** a=37 b=30 + **SLOW** a=15 c=40 |
| D81 | Siphon | Hunger | 20 s | **SIPHON** a=8 | **SIPHON** a=12 + **HEALCUT** a=20 b=60 | **SIPHON** a=6 + **SLOW** a=15 c=60 |
| D82 | Tithe | Hunger | 36 s (signature) | **TITHE** | **TITHE** + **HEALCUT** a=30 b=60 | **TITHE** + **SLOW** a=20 c=40 |
| D83 | Leech | Hunger | 14 s | **VAMP** a=16 b=120 | **VAMP** a=24 b=180 + **HEALCUT** a=30 b=90 | **VAMP** a=12 b=90 + **DRAIN** a=6 b=90 |
| D84 | Famine | Hunger | 10 s | **HEALCUT** a=80 b=60 | **HEALCUT** a=100 b=90 + **EXECUTE** a=16 b=30 | **HEALCUT** a=60 b=45 + **STATDOWN** a=4 b=50 c=120 |
| D85 | Drain | Hunger | 14 s | **DRAIN** a=12 b=120 | **DRAIN** a=18 b=180 + **SLOW** a=20 c=60 | **DRAIN** a=9 b=90 + **HEALCUT** a=30 b=60 |
| D86 | Gnaw | Hunger | 14 s | **DOT** a=12 b=240 | **DOT** a=18 b=360 + **STATDOWN** a=1 b=15 c=120 | **DOT** a=9 b=180 + **SPLASH** a=12 b=12750 |
| D87 | Marrow | Hunger | 10 s | **STATDOWN** a=4 b=100 c=240 | **STATDOWN** a=4 b=150 c=360 + **DOT** a=8 b=120 | **STATDOWN** a=4 b=75 c=180 + **HEALCUT** a=20 b=120 |
| D88 | Starve | Hunger | 10 s | **HEALCUT** a=30 b=360 | **HEALCUT** a=45 b=540 + **VAMP** a=8 b=120 | **HEALCUT** a=22 b=270 + **DOT** a=5 b=240 |
| D89 | Feast | Hunger | 20 s | **SIPHON** a=16 | **SIPHON** a=24 + **EXECUTE** a=16 b=25 | **SIPHON** a=12 + **DOT** a=8 b=120 |
| D90 | Rift Storm | Chaos | 24 s | **STORM** a=20000 | **STORM** a=30000 + **STATICBURST** a=16 | **STORM** a=15000 + **AMPLIFY** a=6 b=40 |
| D91 | Dice | Chaos | 14 s | **DICE** | **DICE** + **AMPLIFY** a=4 b=30 | **DICE** + **SHOVE** a=6000 |
| D92 | Unstable | Chaos | 14 s | **UNSTABLE** a=130 b=4 | **UNSTABLE** a=150 b=6 + **LENS** a=8 | **UNSTABLE** a=130 b=8 + **SHOVE** a=10000 |
| D93 | Wild Echo | Chaos | 14 s | **WILDECHO** | **WILDECHO** + **SHOVE** a=9000 | **WILDECHO** + **STATICBURST** a=12 |
| D94 | Coinflip | Chaos | 14 s | **COINFLIP** a=16 | **COINFLIP** a=24 + **SHOVE** a=5000 | **COINFLIP** a=16 + **STORM** a=6000 |
| D95 | Static Burst | Chaos | 20 s | **STATICBURST** a=24 | **STATICBURST** a=36 + **CHAIN** a=21250 b=60 | **STATICBURST** a=18 + **SHOVE** a=7000 |
| D96 | Scramble | Chaos | 10 s | **SCRAMBLE** | **SCRAMBLE** + **STUN** a=8 | **SCRAMBLE** + **DISARM** a=16 |
| D97 | Entropy | Chaos | 14 s | **ENTROPY** a=32 | **ENTROPY** a=48 + **STORM** a=7000 | **ENTROPY** a=24 + **HEALCUT** a=15 b=60 |
| D98 | Kaleidoscope | Chaos | 36 s (signature) | **KALEIDO** | **KALEIDO** + **ECHO** a=20 b=30 | **KALEIDO** + **STORM** a=8000 |
| D99 | Calm Eye | Chaos | 10 s | **VENT** a=15 | **VENT** a=12 + **SLOW** a=15 b=15 c=60 | **VENT** a=8 + **CLEANSE** |
| D100 | Event Horizon | Gravity | 36 s (signature) | **WELL** a=30 | **WELL** a=45 + **ROOT** a=19 | **WELL** a=22 + **STUN** a=10 |
| D101 | Sinkwell | Gravity | 24 s | **WELL** a=12 | **WELL** a=18 + **SLOW** a=20 c=60 | **WELL** a=9 + **AMPLIFY** a=8 b=60 |
| D102 | Heavy Pull | Gravity | 24 s | **WELL** a=20 | **WELL** a=30 + **DOT** a=12 b=120 | **WELL** a=15 + **SLOW** a=25 b=10 c=60 |
| D103 | Repulsor | Gravity | 24 s | **REPEL** a=9000 | **REPEL** a=14000 + **DISARM** a=19 | **REPEL** a=7000 + **SLOW** a=20 c=60 |
| D104 | Outward Tide | Gravity | 24 s | **REPEL** a=6000 | **REPEL** a=9000 + **STATDOWN** a=1 b=20 c=90 | **REPEL** a=4500 + **HEALCUT** a=30 b=60 |
| D105 | Crush Point | Gravity | 24 s | **WELL** a=10 | **SPLASH** a=30 b=13600 + **WELL** a=15 | **SPLASH** a=20 b=10200 + **REPEL** a=5000 |
| D106 | Orbit | Gravity | 24 s | **SHOVE** a=6000 | **GRAB** a=15000 + **WELL** a=10 | **SHOVE** a=4500 + **WELL** a=8 |
| D107 | Mass Shift | Gravity | 24 s | **PULL** a=25 | **KNOCK** a=16000 + **WELL** a=12 | **PULL** a=19 + **REPEL** a=3500 |
| D108 | Lens of Weight | Gravity | 14 s | **LENS** a=12 | **LENS** a=18 + **WELL** a=14 | **LENS** a=9 + **REPEL** a=5500 |
| D109 | Gravity Well | Gravity | 24 s | **WELL** a=25 | **WELL** a=38 + **SILENCE** a=14 | **WELL** a=19 + **DISARM** a=10 |
| D110 | Absolute Zero | Frost | 36 s (signature) | **CHILL** a=29 b=15 c=3 | **CHILL** a=43 b=22 c=2 + **SHATTER** | **CHILL** a=22 b=11 c=3 + **STUN** a=10 |
| D111 | Rime | Frost | 20 s | **CHILL** a=16 b=8 c=4 | **CHILL** a=24 b=12 c=3 + **SLOW** a=15 c=60 | **CHILL** a=12 b=6 c=4 + **DOT** a=6 b=120 |
| D112 | Hoarfrost | Frost | 20 s | **CHILL** a=19 b=10 c=4 | **CHILL** a=29 b=15 c=3 + **AMPLIFY** a=8 b=60 | **CHILL** a=14 b=8 c=4 + **HEALCUT** a=20 b=60 |
| D113 | Glacier | Frost | 20 s | **CHILL** a=24 b=6 c=5 | **CHILL** a=36 b=9 c=4 + **ROOT** a=16 | **CHILL** a=18 b=5 c=5 + **KNOCK** a=8000 |
| D114 | Frostbite | Frost | 20 s | **CHILL** a=13 b=12 c=3 | **CHILL** a=20 b=18 c=2 + **DOT** a=16 b=120 | **CHILL** a=10 b=9 c=3 + **EXECUTE** a=12 b=30 |
| D115 | Cold Snap | Frost | 20 s | **STUN** a=12 | **STUN** a=19 + **CHILL** a=19 b=10 c=3 | **STUN** a=10 + **CHILL** a=10 b=10 c=3 |
| D116 | Ice Mirror | Frost | 14 s | **REFLECTDMG** a=14 b=60 | **REFLECTDMG** a=20 b=60 + **CHILL** a=16 b=10 c=3 | **REFLECTDMG** a=10 b=60 + **CHILL** a=12 b=9 c=3 |
| D117 | Whiteout | Frost | 10 s | **SLOW** a=30 b=15 c=60 | **SLOW** a=45 b=22 c=60 + **CHILL** a=14 b=8 c=4 | **SLOW** a=22 b=11 c=60 + **SILENCE** a=10 |
| D118 | Permafrost | Frost | 20 s | **CHILL** a=34 b=5 c=6 | **CHILL** a=51 b=8 c=5 + **SILENCE** a=14 | **CHILL** a=26 b=4 c=6 + **DISARM** a=14 |
| D119 | Sleet | Frost | 20 s | **CHILL** a=11 b=7 c=5 | **CHILL** a=17 b=11 c=4 + **SHOVE** a=6000 | **CHILL** a=8 b=5 c=5 + **SPLASH** a=15 b=10200 |
| D120 | Terminal Velocity | Momentum | 36 s (signature) | **MOMENTUM** a=12 b=48 | **MOMENTUM** a=18 b=72 + **KNOCK** a=16000 | **MOMENTUM** a=9 b=36 + **TOEND** |
| D121 | Slipshot | Momentum | 14 s | **MOMENTUM** a=6 b=24 | **MOMENTUM** a=9 b=36 + **AMPLIFY** a=8 b=60 | **MOMENTUM** a=5 b=18 + **SLOW** a=15 c=60 |
| D122 | Rush | Momentum | 10 s | **HASTE** a=24 c=90 | **MOMENTUM** a=12 b=30 + **BLINK** a=9000 | **MOMENTUM** a=6 b=20 + **GRAB** a=10000 |
| D123 | Impact | Momentum | 14 s | **MOMENTUM** a=8 b=32 | **MOMENTUM** a=12 b=48 + **STUN** a=10 | **MOMENTUM** a=6 b=24 + **KNOCK** a=7000 |
| D124 | Slingshot | Momentum | 24 s | **BLINK** a=12000 | **BLINK** a=18000 + **MOMENTUM** a=15 b=45 | **PULL** a=15 + **MOMENTUM** a=8 b=30 |
| D125 | Afterburn | Momentum | 14 s | **DOT** a=16 b=120 | **DOT** a=24 b=120 + **MOMENTUM** a=10 b=40 | **DOT** a=12 b=120 + **MOMENTUM** a=5 b=25 |
| D126 | Slipstream Edge | Momentum | 10 s | **RANGE** a=12000 b=40 | **RICOCHET** a=24 + **MOMENTUM** a=8 b=32 | **SHOVE** a=5000 + **MOMENTUM** a=4 b=20 |
| D127 | Kinetic Mark | Momentum | 10 s | **MARK** a=24 b=120 | **MARK** a=30 b=120 + **MOMENTUM** a=10 b=30 | **MARK** a=16 b=120 + **MOMENTUM** a=6 b=22 |
| D128 | Overdrive | Momentum | 14 s | **MOMENTUM** a=10 b=40 | **MOMENTUM** a=15 b=60 + **SPLASH** a=20 b=13600 | **MOMENTUM** a=8 b=30 + **STATDOWN** b=15 c=90 |
| D129 | Breakneck | Momentum | 14 s | **MOMENTUM** a=14 b=28 | **MOMENTUM** a=21 b=42 + **DISARM** a=14 | **MOMENTUM** a=10 b=21 + **ROOT** a=10 |
| D130 | Supernova | Pressure | 36 s (signature) | **PRESSURE** a=16 c=6 | **PRESSURE** a=48 b=30 + **KNOCK** a=14000 | **PRESSURE** a=24 b=30 + **STUN** a=10 |
| D131 | Compressor | Pressure | 14 s | **PRESSURE** a=8 c=5 | **PRESSURE** a=24 b=24 + **SLOW** a=20 c=60 | **PRESSURE** a=12 b=24 + **DOT** a=8 b=120 |
| D132 | Pressure Cooker | Pressure | 14 s | **PRESSURE** a=10 c=4 | **PRESSURE** a=30 b=36 + **HEALCUT** a=30 b=90 | **PRESSURE** a=15 b=36 + **AMPLIFY** a=8 b=60 |
| D133 | Blast Door | Pressure | 14 s | **PRESSURE** a=6 c=8 | **PRESSURE** a=18 b=18 + **ROOT** a=14 | **PRESSURE** a=9 b=18 + **REPEL** a=5000 |
| D134 | Overpressure | Pressure | 14 s | **PRESSURE** a=12 c=3 | **PRESSURE** a=36 b=42 + **EXPOSE** a=16 b=180 | **PRESSURE** a=18 b=42 + **SILENCE** a=10 |
| D135 | Vacuum Seal | Pressure | 20 s | **SILENCE** a=38 | **SILENCE** a=29 + **PRESSURE** a=20 b=30 | **SILENCE** a=14 + **PRESSURE** a=10 b=30 |
| D136 | Bulkhead | Pressure | 20 s | **SHIELD** a=48 b=120 | **SHIELD** a=72 b=120 + **PRESSURE** a=14 b=30 | **SHIELD** a=36 b=120 + **PRESSURE** a=7 b=30 |
| D137 | Implosion Seed | Pressure | 24 s | **WELL** a=14 | **WELL** a=21 + **PRESSURE** a=26 b=24 | **WELL** a=11 + **PRESSURE** a=13 b=24 |
| D138 | Steam Vent | Pressure | 14 s | **PRESSURE** a=7 c=6 | **PRESSURE** a=21 b=30 + **STATDOWN** a=1 b=20 c=120 | **PRESSURE** a=11 b=30 + **SHOVE** a=5000 |
| D139 | Rupture | Pressure | 14 s | **PRESSURE** a=14 c=4 | **PRESSURE** a=42 b=48 + **DISARM** a=19 | **PRESSURE** a=21 b=48 + **HEALCUT** a=25 b=60 |
| D140 | Dawnbreak | Radiance | 36 s (signature) | **EXPOSE** a=40 b=240 | **EXPOSE** a=60 b=240 + **HEAL** a=16 | **EXPOSE** a=30 b=240 + **STUN** a=10 |
| D141 | Glint | Radiance | 14 s | **EXPOSE** a=16 b=180 | **EXPOSE** a=24 b=180 + **AMPLIFY** a=8 b=60 | **EXPOSE** a=12 b=180 + **SLOW** a=15 c=60 |
| D142 | Halo | Radiance | 20 s | **HEAL** a=12 b=120 | **HEAL** a=18 b=120 + **EXPOSE** a=14 b=180 | **HEAL** a=9 b=120 + **EXPOSE** a=10 b=180 |
| D143 | Searchlight | Radiance | 14 s | **EXPOSE** a=24 b=120 | **EXPOSE** a=36 b=120 + **SILENCE** a=14 | **EXPOSE** a=18 b=120 + **DISARM** a=10 |
| D144 | Sunfire | Radiance | 14 s | **EXPOSE** a=12 b=360 | **EXPOSE** a=18 b=360 + **DOT** a=12 b=180 | **EXPOSE** a=9 b=360 + **DOT** a=6 b=180 |
| D145 | Revelation | Radiance | 10 s | **STATDOWN** a=2 b=20 c=90 | **STATDOWN** a=2 b=30 c=90 + **EXPOSE** a=20 b=180 | **STATDOWN** a=2 b=15 c=90 + **EXPOSE** a=10 b=120 |
| D146 | Beacon | Radiance | 10 s | **STATUP** b=12 c=120 | **STATUP** b=18 c=120 + **EXPOSE** a=18 b=240 | **STATUP** b=9 c=120 + **EXPOSE** a=9 b=240 |
| D147 | Prism Light | Radiance | 14 s | **PRISM** a=2 b=32 | **PRISM** a=3 b=24 + **EXPOSE** a=16 b=120 | **TRUE** a=10 + **EXPOSE** a=8 b=180 |
| D148 | Corona | Radiance | 14 s | **EXPOSE** a=32 b=180 | **EXPOSE** a=48 b=180 + **SPLASH** a=15 b=13600 | **EXPOSE** a=24 b=180 + **KNOCK** a=6000 |
| D149 | Lumen | Radiance | 14 s | **EXPOSE** a=20 b=300 | **EXPOSE** a=30 b=300 + **HEALCUT** a=25 b=90 | **EXPOSE** a=15 b=300 + **AMPLIFY** a=6 b=60 |

### Primitives

| Primitive | Side | Meaning |
|---|---|---|
| BANISH | E | Vanish from the fight: untargetable and unable to act for a (ticks) |
| STUN | E | Stunned for a (ticks) |
| ROOT | E | Rooted in place for a (ticks) |
| SILENCE | E | Cannot use skills for a (ticks) |
| DISARM | E | Cannot basic attack for a (ticks) |
| SLOW | E | -a% move speed and -b% attack speed while inside, lasting c (ticks) |
| AMPLIFY | E | Takes +a% damage from everyone for b (ticks) |
| DOT | E | Burns a per mille of max HP per second for b (ticks) |
| TRUE | P | Packets passing through deal +a% of their damage as true damage |
| SHATTER | E | Every shield is stripped |
| STATDOWN | E | -b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 regen) for c (ticks) |
| REFLECT | E | Reflected to the far side of the rift, wall-clamped |
| PULL | E | Dragged a% of the way to the rift centre |
| KNOCK | E | Thrown a units away from the rift centre |
| SHOVE | E | Shoved a units in a seeded random direction |
| SWAPSPACE | X | The two fighters nearest the centre trade places |
| TETHER | E | Leaving the zone snaps them back to its edge once |
| GRAB | E | Dragged a units toward Vesper |
| REWIND | E | Snaps back to where it stood a ticks ago |
| CDUP | E | Skill cooldowns +a% for b (ticks) |
| HEALCUT | E | -a% healing received for b (ticks) |
| SHELTER | A | Takes -a% damage for b (ticks) |
| HEAL | A | Heals a per mille of max HP per second for b (ticks); b = 0 heals once |
| SHIELD | A | Shield of a per mille of max HP for b (ticks), inside the shared support cap |
| UNDYING | A | Cannot drop below 1 HP for a (ticks); once per ally per 25 s |
| STATUP | A | +b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 crit chance) for c (ticks) |
| HASTE | A | +a% move speed and +b% attack speed for c (ticks) |
| RANGE | A | +a attack range for b (ticks) |
| GHOST | A | Walks through walls for a (ticks); once per ally per 10 s |
| FOLD | A | Allies entering come out of the nearest other Passage rift |
| CLEANSE | A | Every crowd control is removed |
| ALLYINVIS | A | Invisible for a (ticks) |
| BADR | A | -a% damage from basic attacks for b (ticks) |
| REFLECTDMG | A | Reflects a% of damage taken for b (ticks) |
| CCIMMUNE | A | Immune to crowd control for a (ticks); once per ally per 25 s |
| CDDOWN | A | Skill cooldowns -a% for b (ticks) |
| SELFINVIS | S | Vesper is invisible while inside, lasting a (ticks); once per 4 s |
| VAMP | S | Vesper heals a% of the damage she deals for b (ticks) |
| TITHE | X | Enemy shields inside are stripped and Vesper gains a shield inside the support cap |
| ECHO | P | Packets passing through repeat at a% after b ticks |
| PRISM | P | Packets passing through split into a at b% each |
| LENS | P | Packets passing through gain +a% damage |
| WALL | X | A glass wall across the rift: enemies crossing it are thrown back |
| DICE | X | Each pulse rolls a dimension from another family |
| UNSTABLE | X | Imbues at a% strength but every packet adds b strain |
| VENT | X | Each pulse vents a strain |
| KALEIDO | X | Packets leaving through it roll a random family-pair bonus |
| SIPHON | X | Allies inside heal a% of the damage Vesper deals there |
| DRAIN | E | Burns a per mille of max HP per second for b (ticks) and heals Vesper the same |
| STATICBURST | E | Stunned for a seeded 0..a ticks |
| SCRAMBLE | X | Two enemies inside trade places |
| ENTROPY | E | Burns a seeded 0..a per mille of max HP per pulse |
| STORM | E | Shoved a units in a seeded random direction, twice |
| WILDECHO | P | Packets passing through repeat at a seeded 20..80% after 30 ticks |
| COINFLIP | X | Each pulse flips: enemies take +a% damage, or allies take -a% damage |
| CHAIN | E | The same effect lands on the nearest other enemy within a units at b% strength |
| WALLSLAM | E | Thrown into a wall: stunned a ticks |
| TOEND | E | Carried to the far end of the swipe |
| MARK | E | Marked for b ticks: the next rift punch arcs a% of its damage to every marked enemy |
| RECALLHP | A | Regains the HP lost in the last a ticks (up to b per mille of max HP) |
| SELFSWAP | E | Vesper and the target trade places |
| ENCASE | E | Encased in glass: rooted a ticks, takes -b% damage meanwhile |
| SPLASH | E | Enemies within b units take a% of the hit |
| EXECUTE | P | +a% damage when the target is below b% HP |
| BLINK | S | Vesper steps a units toward the target |
| RICOCHET | E | The nearest other enemy within 34000 takes a% of the hit |
| WELL | E | Pulled a% of the way to the rift centre on every pulse (draws a group together) |
| REPEL | E | Pushed a units out from the rift centre on every pulse (scatters a group) |
| CHILL | E | Chilled: -b% move speed per stack; at c stacks frozen (rooted a ticks) and the stacks clear. Stacks fade 1 s after the last chill, so stepping out sheds them |
| MOMENTUM | P | Packets passing through gain +a% damage per 10000 units Vesper moved in the last second, up to +b% |
| PRESSURE | E | Each pulse with an enemy inside stores a charge (up to c); when the rift closes or implodes, enemies inside take a per mille of max HP per charge. On a punch or swipe: a burst b ticks later, a per mille of max HP to every enemy near the target |
| EXPOSE | E | Exposed for b ticks: her next punch, swipe or implode on it detonates for a per mille of max HP |

## Family-pair bonuses (orb combos)

Two orbs of different families spent within a second of each other trigger their pair's bonus.

| Families | Bonus | Effect | Strength |
|---|---|---|---|
| Control + Control | Pure Control Lock | The bound target is rooted 16 ticks after the shot | 20 |
| Control + Ruin | Control-Ruin Execute Window | +8% damage when the target is already controlled | 8 |
| Control + Motion | Control-Motion Crush | A recently displaced target is stunned 10 ticks | 10 |
| Control + Haven | Control-Haven Sanctuary | Allies near the exit become immune to crowd control for 22 ticks | 23 |
| Control + Passage | Control-Passage Prison | The target is rooted 24 ticks and dragged to the exit | 24 |
| Control + Time | Control-Time Lock | The bound target is rooted 16 ticks after the shot | 16 |
| Control + Veil | Control-Veil Sanctuary | Allies near the exit become immune to crowd control for 22 ticks | 23 |
| Control + Matter | Control-Matter Prison | The target is rooted 24 ticks and dragged to the exit | 24 |
| Control + Hunger | Control-Hunger Execute Window | +8% damage when the target is already controlled | 8 |
| Control + Chaos | Control-Chaos Crush | A recently displaced target is stunned 10 ticks | 10 |
| Control + Gravity | Control-Gravity Crush | A recently displaced target is stunned 10 ticks | 10 |
| Control + Frost | Control-Frost Lock | The bound target is rooted 16 ticks after the shot | 16 |
| Control + Momentum | Control-Momentum Execute Window | +8% damage when the target is already controlled | 8 |
| Control + Pressure | Control-Pressure Prison | The target is rooted 24 ticks and dragged to the exit | 24 |
| Control + Radiance | Control-Radiance Sanctuary | Allies near the exit become immune to crowd control for 22 ticks | 23 |
| Ruin + Ruin | Pure Ruin Overcharge | The shot jumps to a second enemy at 40% | 50 |
| Ruin + Motion | Ruin-Motion Shrapnel | A recently displaced target takes 2.4% of max HP | 24 |
| Ruin + Haven | Ruin-Haven Siphon | Allies near the exit heal 8% of the damage | 8 |
| Ruin + Passage | Ruin-Passage Leak | The exit dimension also pulses at the farthest rift | 1 |
| Ruin + Time | Ruin-Time Execute Window | +8% damage when the target is already controlled | 8 |
| Ruin + Veil | Ruin-Veil Siphon | Allies near the exit heal 8% of the damage | 8 |
| Ruin + Matter | Ruin-Matter Leak | The exit dimension also pulses at the farthest rift | 1 |
| Ruin + Hunger | Ruin-Hunger Overcharge | The shot jumps to a second enemy at 40% | 40 |
| Ruin + Chaos | Ruin-Chaos Shrapnel | A recently displaced target takes 2.4% of max HP | 24 |
| Ruin + Gravity | Ruin-Gravity Shrapnel | A recently displaced target takes 2.4% of max HP | 24 |
| Ruin + Frost | Ruin-Frost Execute Window | +8% damage when the target is already controlled | 8 |
| Ruin + Momentum | Ruin-Momentum Overcharge | The shot jumps to a second enemy at 40% | 40 |
| Ruin + Pressure | Ruin-Pressure Leak | The exit dimension also pulses at the farthest rift | 1 |
| Ruin + Radiance | Ruin-Radiance Siphon | Allies near the exit heal 8% of the damage | 8 |
| Motion + Motion | Pure Motion Vortex | Enemies near the target are dragged to it | 43 |
| Motion + Haven | Motion-Haven Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Motion + Passage | Motion-Passage Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Motion + Time | Motion-Time Crush | A recently displaced target is stunned 10 ticks | 10 |
| Motion + Veil | Motion-Veil Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Motion + Matter | Motion-Matter Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Motion + Hunger | Motion-Hunger Shrapnel | A recently displaced target takes 2.4% of max HP | 24 |
| Motion + Chaos | Motion-Chaos Vortex | Enemies near the target are dragged to it | 35 |
| Motion + Gravity | Motion-Gravity Vortex | Enemies near the target are dragged to it | 35 |
| Motion + Frost | Motion-Frost Crush | A recently displaced target is stunned 10 ticks | 10 |
| Motion + Momentum | Motion-Momentum Shrapnel | A recently displaced target takes 2.4% of max HP | 24 |
| Motion + Pressure | Motion-Pressure Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Motion + Radiance | Motion-Radiance Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Haven + Haven | Pure Haven Haven | Allies near the exit heal 2.4% of max HP | 30 |
| Haven + Passage | Haven-Passage Gift | Allies near the exit gain a 4% max HP shield | 40 |
| Haven + Time | Haven-Time Sanctuary | Allies near the exit become immune to crowd control for 22 ticks | 23 |
| Haven + Veil | Haven-Veil Haven | Allies near the exit heal 2.4% of max HP | 24 |
| Haven + Matter | Haven-Matter Gift | Allies near the exit gain a 4% max HP shield | 40 |
| Haven + Hunger | Haven-Hunger Siphon | Allies near the exit heal 8% of the damage | 8 |
| Haven + Chaos | Haven-Chaos Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Haven + Gravity | Haven-Gravity Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Haven + Frost | Haven-Frost Sanctuary | Allies near the exit become immune to crowd control for 22 ticks | 23 |
| Haven + Momentum | Haven-Momentum Siphon | Allies near the exit heal 8% of the damage | 8 |
| Haven + Pressure | Haven-Pressure Gift | Allies near the exit gain a 4% max HP shield | 40 |
| Haven + Radiance | Haven-Radiance Haven | Allies near the exit heal 2.4% of max HP | 24 |
| Passage + Passage | Pure Passage Paradox | A mirrored copy of the shot fires 12 ticks later at 40% | 50 |
| Passage + Time | Passage-Time Prison | The target is rooted 24 ticks and dragged to the exit | 24 |
| Passage + Veil | Passage-Veil Gift | Allies near the exit gain a 4% max HP shield | 40 |
| Passage + Matter | Passage-Matter Paradox | A mirrored copy of the shot fires 12 ticks later at 40% | 40 |
| Passage + Hunger | Passage-Hunger Leak | The exit dimension also pulses at the farthest rift | 1 |
| Passage + Chaos | Passage-Chaos Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Passage + Gravity | Passage-Gravity Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Passage + Frost | Passage-Frost Prison | The target is rooted 24 ticks and dragged to the exit | 24 |
| Passage + Momentum | Passage-Momentum Leak | The exit dimension also pulses at the farthest rift | 1 |
| Passage + Pressure | Passage-Pressure Paradox | A mirrored copy of the shot fires 12 ticks later at 40% | 40 |
| Passage + Radiance | Passage-Radiance Gift | Allies near the exit gain a 4% max HP shield | 40 |
| Time + Time | Pure Time Lock | The bound target is rooted 16 ticks after the shot | 20 |
| Time + Veil | Time-Veil Sanctuary | Allies near the exit become immune to crowd control for 22 ticks | 23 |
| Time + Matter | Time-Matter Prison | The target is rooted 24 ticks and dragged to the exit | 24 |
| Time + Hunger | Time-Hunger Execute Window | +8% damage when the target is already controlled | 8 |
| Time + Chaos | Time-Chaos Crush | A recently displaced target is stunned 10 ticks | 10 |
| Time + Gravity | Time-Gravity Crush | A recently displaced target is stunned 10 ticks | 10 |
| Time + Frost | Time-Frost Lock | The bound target is rooted 16 ticks after the shot | 16 |
| Time + Momentum | Time-Momentum Execute Window | +8% damage when the target is already controlled | 8 |
| Time + Pressure | Time-Pressure Prison | The target is rooted 24 ticks and dragged to the exit | 24 |
| Time + Radiance | Time-Radiance Sanctuary | Allies near the exit become immune to crowd control for 22 ticks | 23 |
| Veil + Veil | Pure Veil Haven | Allies near the exit heal 2.4% of max HP | 30 |
| Veil + Matter | Veil-Matter Gift | Allies near the exit gain a 4% max HP shield | 40 |
| Veil + Hunger | Veil-Hunger Siphon | Allies near the exit heal 8% of the damage | 8 |
| Veil + Chaos | Veil-Chaos Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Veil + Gravity | Veil-Gravity Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Veil + Frost | Veil-Frost Sanctuary | Allies near the exit become immune to crowd control for 22 ticks | 23 |
| Veil + Momentum | Veil-Momentum Siphon | Allies near the exit heal 8% of the damage | 8 |
| Veil + Pressure | Veil-Pressure Gift | Allies near the exit gain a 4% max HP shield | 40 |
| Veil + Radiance | Veil-Radiance Haven | Allies near the exit heal 2.4% of max HP | 24 |
| Matter + Matter | Pure Matter Paradox | A mirrored copy of the shot fires 12 ticks later at 40% | 50 |
| Matter + Hunger | Matter-Hunger Leak | The exit dimension also pulses at the farthest rift | 1 |
| Matter + Chaos | Matter-Chaos Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Matter + Gravity | Matter-Gravity Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Matter + Frost | Matter-Frost Prison | The target is rooted 24 ticks and dragged to the exit | 24 |
| Matter + Momentum | Matter-Momentum Leak | The exit dimension also pulses at the farthest rift | 1 |
| Matter + Pressure | Matter-Pressure Paradox | A mirrored copy of the shot fires 12 ticks later at 40% | 40 |
| Matter + Radiance | Matter-Radiance Gift | Allies near the exit gain a 4% max HP shield | 40 |
| Hunger + Hunger | Pure Hunger Overcharge | The shot jumps to a second enemy at 40% | 50 |
| Hunger + Chaos | Hunger-Chaos Shrapnel | A recently displaced target takes 2.4% of max HP | 24 |
| Hunger + Gravity | Hunger-Gravity Shrapnel | A recently displaced target takes 2.4% of max HP | 24 |
| Hunger + Frost | Hunger-Frost Execute Window | +8% damage when the target is already controlled | 8 |
| Hunger + Momentum | Hunger-Momentum Overcharge | The shot jumps to a second enemy at 40% | 40 |
| Hunger + Pressure | Hunger-Pressure Leak | The exit dimension also pulses at the farthest rift | 1 |
| Hunger + Radiance | Hunger-Radiance Siphon | Allies near the exit heal 8% of the damage | 8 |
| Chaos + Chaos | Pure Chaos Vortex | Enemies near the target are dragged to it | 43 |
| Chaos + Gravity | Chaos-Gravity Vortex | Enemies near the target are dragged to it | 35 |
| Chaos + Frost | Chaos-Frost Crush | A recently displaced target is stunned 10 ticks | 10 |
| Chaos + Momentum | Chaos-Momentum Shrapnel | A recently displaced target takes 2.4% of max HP | 24 |
| Chaos + Pressure | Chaos-Pressure Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Chaos + Radiance | Chaos-Radiance Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Gravity + Gravity | Pure Gravity Vortex | Enemies near the target are dragged to it | 43 |
| Gravity + Frost | Gravity-Frost Crush | A recently displaced target is stunned 10 ticks | 10 |
| Gravity + Momentum | Gravity-Momentum Shrapnel | A recently displaced target takes 2.4% of max HP | 24 |
| Gravity + Pressure | Gravity-Pressure Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Gravity + Radiance | Gravity-Radiance Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Frost + Frost | Pure Frost Lock | The bound target is rooted 16 ticks after the shot | 20 |
| Frost + Momentum | Frost-Momentum Execute Window | +8% damage when the target is already controlled | 8 |
| Frost + Pressure | Frost-Pressure Prison | The target is rooted 24 ticks and dragged to the exit | 24 |
| Frost + Radiance | Frost-Radiance Sanctuary | Allies near the exit become immune to crowd control for 22 ticks | 23 |
| Momentum + Momentum | Pure Momentum Overcharge | The shot jumps to a second enemy at 40% | 50 |
| Momentum + Pressure | Momentum-Pressure Leak | The exit dimension also pulses at the farthest rift | 1 |
| Momentum + Radiance | Momentum-Radiance Siphon | Allies near the exit heal 8% of the damage | 8 |
| Pressure + Pressure | Pure Pressure Paradox | A mirrored copy of the shot fires 12 ticks later at 40% | 50 |
| Pressure + Radiance | Pressure-Radiance Gift | Allies near the exit gain a 4% max HP shield | 40 |
| Radiance + Radiance | Pure Radiance Haven | Allies near the exit heal 2.4% of max HP | 30 |

## Ranks

0. **Spark**: Reflex chaos: random rift types, techniques and dimensions; overloads, shatters and overclocks on a whim
1. **Tear**: Leads targets, including opening delays, for chain, Kidnap, Rescue and Seed placement
2. **Rift**: Safety: watches stress, never overloads past 7 with enemies near, shatters only when the gauntlet holds, checks every exit
3. **Gate**: Geometry: chain solver, Bank Shot and swipe angles, Crossfire splits, Implode and Domino timing on 2+ enemies
4. **Wanderer**: Survival: keeps a spare orb, sets a Lifeline, vents overload when dived, Pockets or Rescues allies
5. **Cartographer**: Team engage: Gates and Sky Drops with the frontliner, Awakening timed with the team's engage, Kidnaps into the team
6. **Riftwalker**: Priority: picks every orb's dimension for its target (heal cut on healers, Shatter on shields), Infuse on the frontliner's rift
7. **Worldbreaker**: Full sequences, 5 steps deep: combo families for their twist, overload ridden into a 3-enemy Shatter, overclock stopped in time

The #1 athlete of the Top 10 is shown as **Between Worlds** (rank 7 brain, visuals only).
