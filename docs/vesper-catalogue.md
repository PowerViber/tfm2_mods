# Vesper, the Rift Gauntlets: catalogue

Generated from `Claude outputs/vesper/catalogue.json`. Everything here is available at every mastery rank; rank only changes how well the athlete plans with it.

## Gauntlet moves

| Move | Orbs | What happens |
|---|---|---|
| Rift Swipe | 2 | Swipes the air like pulling a mask off and tears the dimension: a Slash rift opens along the swipe, everyone on the line takes the chosen dimension's swipe effect, enemies take 25 + 50% AD |
| Shatter | 0 | Crushes every pure orb she holds: every open rift rips wide, banishes and damages the enemies inside (never allies). Past the stress cap the left gauntlet breaks |
| Overload | 0 | Forces an extra orb into a 6th to 9th socket: +12 stress now and +1 per 20 ticks while it is held charged |
| Vent | 0 | Drops every overload socket before it cracks the gauntlet |
| Awaken | 0 | At 100% the Hexcore awakens for 6 s: hexorbs at full power, pure + hex orbs fuse, her rifts cannot be broken |
| Overclock | 1 | While awakened, feeds a pure orb into the Hexcore: +20 corruption, +5% hexorb power. At 100 corruption the right gauntlet breaks |

## Hexcore tech modules

Each athlete's Hexcore carries 6 of these 8 (derived from the athlete, editable in the editor).

| Module | Effect | a | b |
|---|---|---|---|
| Plating | Shields the nearest ally for a per mille of max HP | 60 | 180 |
| Coil | Allies nearby gain +a% move speed for b ticks | 25 | 90 |
| Stasis | Enemies nearby are slowed a% for b ticks | 30 | 60 |
| Magnet | Enemies nearby are pulled a% toward the point | 35 | 0 |
| Mender | Allies nearby heal a per mille of max HP | 30 | 0 |
| Scanner | Enemies nearby take +a% damage for b ticks | 12 | 90 |
| Spark Arc | Arcs a + b% AD to the two nearest enemies | 20 | 25 |
| Anchor Bolt | The nearest enemy is rooted a ticks | 24 | 0 |

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
| Tear | 1 | 300 | r18000 | Round rift: chain node and portal |
| Slash | 1 | 240 | r6000, length 60000 | A 60000-unit line: enemies crossing it are thrown back; fires fan volleys |
| Sinkhole | 1 | 240 | r28000 | Ground ellipse: slows enemies 30% and swallows anyone inside for 60 ticks |
| Sky Rift | 2 | 180 | r24000 | Opens overhead: rains volleys and drops allies from above |
| Mirror Pane | 1 | 300 | r6000, length 40000 | A glass segment that bounces shots and divers |
| Twin Rift | 2 | 300 | r16000 | Two rifts born linked: instant portal pair, swaps fighters |
| Wandering Rift | 1 | 240 | r16000 | Moves 480 units per tick along the path Vesper draws |
| Anchor Rift | 1 | 360 | r14000 | Stuck to a unit (ally, enemy) or tethered to Vesper at a safe spot |
| Seed Rift | 1 | 480 | r14000 | Hidden for 120 ticks, then a trap for the first enemy to step on it |
| Maw | 3 | 90 | r40000 | Huge and brief: swallows everyone inside, then caves in by itself |

## Techniques

| Technique | Group | Rifts | Stress | Per-target cooldown | What happens |
|---|---|---|---|---|---|
| Chain Fire | offense | Tear, Sky Rift, Mirror Pane, Twin Rift, Anchor Rift, Wandering Rift | 0 | 0 | While a chain is live, every rift punch travels through it and comes out of the last rift |
| Crossfire | offense | Tear | 10 | 0 | One rift punch splits out of up to 3 rifts at 3 targets, at 45% each, for 180 ticks |
| Bank Shot | offense | Mirror Pane | 6 | 0 | A Mirror Pane in the chain focuses the next link (x1.2) and lets punches pass walls |
| Rift Rain | offense | Sky Rift | 14 | 0 | 6 packets fall over the Sky Rift during 60 ticks: 25 + 25% AD each |
| Slash Volley | offense | Slash | 12 | 0 | 5 packets fan out along the Slash: 20 + 30% AD each |
| Implode | offense | Tear, Sinkhole, Sky Rift, Twin Rift, Seed Rift, Anchor Rift, Wandering Rift, Maw | 10 | 0 | Close a rift early: enemies are dragged to its centre and take 20 + 40% AD plus its dimension twice |
| Domino | offense | Tear, Twin Rift | 8 | 0 | Implode every linked rift in turn, 15 ticks apart |
| Overload Punch | offense | Tear, Twin Rift, Sky Rift, Anchor Rift | 0 | 0 | Spends stress: the next rift punch gains +1% per point spent (up to 60) |
| Rift Step | engage | Tear, Twin Rift | 10 | 240 | Vesper steps into a rift within 24000 and out of another |
| Ally Gate | engage | Tear, Twin Rift | 15 | 180 | Allies touching the entrance come out of the exit |
| Sky Drop | engage | Sky Rift | 18 | 180 | A Gate exit from above: the ally lands and stuns enemies within 20000 for 24 ticks |
| Kidnap | engage | Tear, Twin Rift | 18 | 600 | Blink in front of an enemy and arm a stun field for 0.5 s; if they are still in it, both of you drop through to the kidnap rift (20 s cooldown) |
| Sinkhole Trap | engage | Sinkhole | 12 | 600 | Enemies inside for 60 ticks fall through to the chosen exit |
| Swap | engage | Twin Rift | 16 | 480 | The fighters nearest each twin trade places |
| Fling | engage | Wandering Rift | 14 | 240 | Enemies caught are carried along the path and dropped at its end |
| Hook | engage | Anchor Rift | 16 | 600 | Mark an enemy; reel them through to the exit later. Enemies tearing it off free them |
| Phase Out | defense | Tear, Twin Rift | 12 | 240 | Vesper hides in a rift for up to 45 ticks, then leaves through another |
| Rift Curtain | defense | Slash, Tear | 12 | 240 | A Slash, or three Tears in a line: enemies crossing are thrown back |
| Reflect Wall | defense | Mirror Pane | 10 | 240 | Divers bounce off a Mirror Pane toward where they came from |
| Rescue | defense | Tear, Twin Rift | 15 | 900 | A rift under an ally below 25% HP drops them at a safe rift |
| Pocket | defense | Maw, Tear | 12 | 720 | Hide an ally inside a rift for 60 ticks, then return them |
| Lifeline | defense | Anchor Rift | 8 | 1200 | An Anchor Rift tethered to Vesper at a safe spot: below 25% HP she is pulled to it |
| Escort | team | Anchor Rift | 8 | 0 | An Anchor Rift on an ally: rift punches come out of the diving ally |
| Rotation Gate | team | Twin Rift | 20 | 180 | A long Twin gate that moves allies across the map toward the next fight |
| Seed Trap | team | Seed Rift | 10 | 600 | A hidden Seed on a predicted path: opens with its dimension and swallows the victim |
| Sweep | team | Wandering Rift | 12 | 0 | Drag a dimension across a whole line of enemies |
| Infuse | team | Tear, Sinkhole, Sky Rift, Twin Rift, Anchor Rift, Seed Rift, Wandering Rift, Maw, Slash, Mirror Pane | 8 | 0 | Spends a hexorb on an open rift: its tech module pulses from that rift for the rest of its life |
| Stitch | team | Tear | 20 | 0 | Sew two Tears into one larger rift holding both orb dimensions and their bonus |

## Dimensions: rift zone, punch and swipe

Any orb opens any of the 100 dimensions. The rift zone, the rift punch and the rift swipe each have their own behaviour (300 in all).

| ID | Dimension | Family | Rift zone | Punch (hole) | Swipe (tear line) |
|---|---|---|---|---|---|
| D00 | Void | Control | **BANISH** a=60 | **BANISH** a=30 + **EXECUTE** a=15 b=30 | **BANISH** a=18 + **TOEND** |
| D01 | Silent Realm | Control | **SILENCE** a=60 | **SILENCE** a=90 + **CDUP** a=30 b=180 | **SILENCE** a=45 + **DISARM** a=15 |
| D02 | Unarmed World | Control | **DISARM** a=60 | **DISARM** a=90 + **STATDOWN** b=20 c=120 | **DISARM** a=45 + **SLOW** b=25 c=60 |
| D03 | Frozen Moment | Control | **STUN** a=30 | **STUN** a=45 + **SHATTER** | **STUN** a=22 + **SLOW** a=40 c=60 |
| D04 | Anchor | Control | **ROOT** a=60 | **ROOT** a=90 + **AMPLIFY** a=10 b=60 | **ROOT** a=45 + **PULL** a=30 |
| D05 | Heavy World | Control | **SLOW** a=35 b=20 c=40 | **SLOW** a=52 b=30 c=60 + **ROOT** a=15 | **SLOW** a=26 b=15 c=30 + **CDUP** a=20 b=120 |
| D06 | Leaden Sky | Control | **SLOW** a=50 c=90 | **SLOW** a=75 c=135 + **STATDOWN** a=1 b=15 c=90 | **SLOW** a=37 c=67 + **STUN** a=8 |
| D07 | Lull | Control | **STUN** a=18 | **STUN** a=27 + **SILENCE** a=30 | **STUN** a=13 + **BADR** a=15 b=60 |
| D08 | Gloom Cage | Control | **ROOT** a=90 | **ROOT** a=135 + **DOT** a=10 b=120 | **ROOT** a=67 + **AMPLIFY** a=8 b=60 |
| D09 | Tar Pit | Control | **SLOW** a=25 b=10 c=240 | **SLOW** a=37 b=15 c=360 + **HEALCUT** a=25 b=120 | **SLOW** a=18 b=7 c=180 + **DOT** a=8 b=180 |
| D10 | Static Realm | Ruin | **AMPLIFY** a=15 b=40 | **AMPLIFY** a=25 b=60 + **RICOCHET** a=30 | **AMPLIFY** a=12 b=30 + **MARK** a=40 b=180 |
| D11 | Ember | Ruin | **DOT** a=20 b=180 | **DOT** a=30 b=270 + **CHAIN** a=30000 b=100 | **DOT** a=15 b=135 + **SLOW** a=15 c=60 |
| D12 | True Rift | Ruin | **TRUE** a=15 | **TRUE** a=22 + **SHATTER** | **TRUE** a=11 + **AMPLIFY** a=6 b=40 |
| D13 | Shatter | Ruin | **SHATTER** | **SHATTER** + **STATDOWN** a=1 b=20 c=120 | **SHATTER** + **HEALCUT** a=15 b=60 |
| D14 | Acid Rain | Ruin | **DOT** a=10 b=360 | **DOT** a=15 b=540 + **STATDOWN** a=2 b=20 c=180 | **DOT** a=7 b=270 + **SPLASH** a=20 b=20000 |
| D15 | Fracture | Ruin | **AMPLIFY** a=25 b=60 | **AMPLIFY** a=37 b=90 + **EXECUTE** a=25 b=35 | **AMPLIFY** a=18 b=45 + **SHATTER** |
| D16 | Brittle | Ruin | **STATDOWN** a=1 b=25 c=120 | **STATDOWN** a=1 b=37 c=180 + **SPLASH** a=30 b=25000 | **STATDOWN** a=1 b=18 c=90 + **SLOW** a=15 b=15 c=60 |
| D17 | Hex Ward | Ruin | **STATDOWN** a=2 b=25 c=120 | **STATDOWN** a=2 b=37 c=180 + **SILENCE** a=20 | **STATDOWN** a=2 b=18 c=90 + **TRUE** a=10 |
| D18 | Cinder | Ruin | **DOT** a=30 b=120 | **DOT** a=45 b=180 + **SPLASH** a=25 b=20000 | **DOT** a=22 b=90 + **HEALCUT** a=25 b=120 |
| D19 | Razor Light | Ruin | **TRUE** a=25 | **TRUE** a=37 + **RICOCHET** a=40 | **TRUE** a=18 + **LENS** a=15 |
| D20 | Mirror Realm | Motion | **REFLECT** | **REFLECT** + **STUN** a=12 | **REFLECT** + **SLOW** a=20 c=40 |
| D21 | Drift | Motion | **PULL** a=35 | **PULL** a=52 + **SLOW** a=30 c=40 | **PULL** a=26 + **AMPLIFY** a=8 b=40 |
| D22 | Gale | Motion | **KNOCK** a=20000 | **KNOCK** a=25000 + **WALLSLAM** a=30 | **KNOCK** a=15000 + **SLOW** a=20 c=30 |
| D23 | Crosswind | Motion | **SHOVE** a=15000 | **SHOVE** a=22500 + **DISARM** a=20 | **SHOVE** a=11250 + **SLOW** a=25 c=40 |
| D24 | Tether | Motion | **TETHER** | **GRAB** a=24000 + **ROOT** a=20 | **PULL** a=50 + **SLOW** a=20 c=60 |
| D25 | Swap Space | Motion | **SWAPSPACE** | **SELFSWAP** + **STUN** a=12 | **SWAPSPACE** + **SLOW** a=30 c=40 |
| D26 | Undertow | Motion | **PULL** a=60 | **PULL** a=90 + **ROOT** a=20 | **PULL** a=45 + **STUN** a=10 |
| D27 | Repulse | Motion | **KNOCK** a=32000 | **KNOCK** a=48000 + **STATDOWN** a=1 b=15 c=60 | **KNOCK** a=24000 + **SHELTER** a=15 b=60 |
| D28 | Whirl | Motion | **SHOVE** a=9000 | **SHOVE** a=13500 + **SPLASH** a=20 b=18000 | **SHOVE** a=6750 + **SILENCE** a=15 |
| D29 | Slipstream | Motion | **GRAB** a=20000 | **GRAB** a=30000 + **AMPLIFY** a=15 b=40 | **GRAB** a=15000 + **HASTE** a=20 c=90 |
| D30 | Shelter | Haven | **SHELTER** a=15 b=40 | **SHELTER** a=22 b=60 + **SLOW** a=15 c=40 | **SHELTER** a=11 b=30 + **CLEANSE** |
| D31 | Bloom | Haven | **HEAL** a=20 b=40 | **HEAL** a=30 b=60 + **HEALCUT** a=20 b=60 | **HEAL** a=15 b=30 + **HASTE** a=15 c=60 |
| D32 | Bulwark | Haven | **SHIELD** a=80 b=180 | **SHIELD** a=120 b=270 + **DISARM** a=15 | **SHIELD** a=60 b=135 + **BADR** a=20 b=90 |
| D33 | Last Breath | Haven | **UNDYING** a=60 | **UNDYING** a=90 + **SLOW** a=20 b=20 c=40 | **UNDYING** a=45 + **SHELTER** a=20 b=40 |
| D34 | Sanctum | Haven | **SHELTER** a=25 b=40 | **SHELTER** a=37 b=60 + **KNOCK** a=12000 | **SHELTER** a=18 b=30 + **CCIMMUNE** a=30 |
| D35 | Spring | Haven | **HEAL** a=40 | **HEAL** a=60 + **STUN** a=8 | **HEAL** a=30 + **HASTE** a=25 c=40 |
| D36 | Aegis | Haven | **SHIELD** a=150 b=60 | **SHIELD** a=225 b=90 + **KNOCK** a=18000 | **SHIELD** a=112 b=45 + **REFLECTDMG** a=10 b=60 |
| D37 | Mend | Haven | **HEAL** a=10 b=360 | **HEAL** a=15 b=540 + **DOT** a=8 b=120 | **HEAL** a=7 b=270 + **CDDOWN** a=10 b=120 |
| D38 | Ward | Haven | **STATUP** a=2 b=25 c=120 | **STATUP** a=2 b=37 c=180 + **STATDOWN** a=3 b=20 c=120 | **STATUP** a=2 b=18 c=90 + **SHIELD** a=40 b=120 |
| D39 | Iron Skin | Haven | **STATUP** a=1 b=25 c=120 | **STATUP** a=1 b=37 c=180 + **STATDOWN** a=1 b=20 c=120 | **STATUP** a=1 b=18 c=90 + **SHELTER** a=10 b=90 |
| D40 | Folded Map | Passage | **FOLD** | **BLINK** a=30000 + **HASTE** a=25 c=90 | **GHOST** a=120 + **HASTE** a=30 c=120 |
| D41 | Ghost Plane | Passage | **GHOST** a=180 | **GHOST** a=270 + **BLINK** a=20000 | **GHOST** a=135 + **HASTE** a=20 c=120 |
| D42 | Far Shore | Passage | **RANGE** a=15000 b=40 | **RANGE** a=22500 b=60 + **KNOCK** a=15000 | **RANGE** a=11250 b=30 + **HASTE** b=15 c=120 |
| D43 | Swift Plane | Passage | **HASTE** a=30 b=25 c=180 | **HASTE** a=45 b=37 c=270 + **SLOW** a=25 c=60 | **HASTE** a=22 b=18 c=135 + **GHOST** a=60 |
| D44 | Tailwind | Passage | **HASTE** a=45 c=90 | **HASTE** a=67 c=135 + **SHOVE** a=12000 | **HASTE** a=33 c=67 + **CLEANSE** |
| D45 | Long Reach | Passage | **RANGE** a=25000 b=60 | **RANGE** a=37500 b=90 + **GRAB** a=15000 | **RANGE** a=18750 b=45 + **STATUP** b=10 c=120 |
| D46 | Stride | Passage | **HASTE** a=20 c=360 | **HASTE** a=30 c=540 + **BLINK** a=15000 | **HASTE** a=15 c=270 + **SHELTER** a=10 b=60 |
| D47 | Skyway | Passage | **GHOST** a=90 | **GHOST** a=135 + **KNOCK** a=20000 | **GHOST** a=67 + **ALLYINVIS** a=30 |
| D48 | Sprint | Passage | **HASTE** a=60 c=40 | **HASTE** a=90 c=60 + **BLINK** a=25000 | **HASTE** a=45 c=30 + **BADR** a=15 b=60 |
| D49 | Courier Wind | Passage | **HASTE** a=15 b=15 c=600 | **HASTE** a=22 b=22 c=900 + **CDDOWN** a=10 b=120 | **HASTE** a=11 b=11 c=450 + **RANGE** a=10000 b=120 |
| D50 | Rewind | Time | **REWIND** a=120 | **REWIND** a=180 + **SILENCE** a=20 | **REWIND** a=120 + **RECALLHP** a=120 b=150 |
| D51 | Echo | Time | **ECHO** a=40 b=60 | **ECHO** a=60 b=90 + **CDUP** a=15 b=120 | **ECHO** a=30 b=45 + **SLOW** a=15 c=60 |
| D52 | Haste Clock | Time | **HASTE** b=35 c=120 | **HASTE** b=52 c=180 + **CDUP** a=20 b=120 | **HASTE** b=26 c=90 + **CDDOWN** a=15 b=180 |
| D53 | Slow Clock | Time | **SLOW** b=35 c=120 | **SLOW** b=52 c=180 + **CDUP** a=30 b=120 | **SLOW** b=26 c=90 + **DISARM** a=10 |
| D54 | Hourglass | Time | **CDUP** a=50 b=180 | **CDUP** a=75 b=270 + **SILENCE** a=15 | **CDUP** a=37 b=135 + **SLOW** a=20 c=90 |
| D55 | Quickening | Time | **CDDOWN** a=25 b=180 | **CDDOWN** a=37 b=270 + **CDUP** a=25 b=120 | **CDDOWN** a=18 b=135 + **HASTE** a=15 c=90 |
| D56 | Afterimage | Time | **ECHO** a=60 b=30 | **ECHO** a=90 b=45 + **SELFINVIS** a=20 | **ECHO** a=45 b=22 + **SHOVE** a=6000 |
| D57 | Deja Vu | Time | **REWIND** a=60 | **REWIND** a=90 + **STUN** a=15 | **REWIND** a=45 + **RECALLHP** a=60 b=80 |
| D58 | Long Second | Time | **SLOW** a=20 b=20 c=180 | **SLOW** a=30 b=30 c=270 + **ROOT** a=10 | **SLOW** a=15 b=15 c=135 + **CDUP** a=25 b=90 |
| D59 | Prime Time | Time | **STATUP** b=15 c=180 | **STATUP** b=22 c=270 + **VAMP** a=15 b=90 | **STATUP** b=11 c=135 + **CDDOWN** a=15 b=120 |
| D60 | Hidden Side | Veil | **SELFINVIS** a=40 | **SELFINVIS** a=60 + **BLINK** a=18000 | **SELFINVIS** a=30 + **SLOW** a=20 c=40 |
| D61 | Blind Fog | Veil | **STATDOWN** b=25 c=60 | **STATDOWN** b=37 c=90 + **DISARM** a=25 | **STATDOWN** b=18 c=45 + **ALLYINVIS** a=20 |
| D62 | Clear Sky | Veil | **CLEANSE** | **CLEANSE** + **STATDOWN** a=2 b=15 c=90 | **CLEANSE** + **CCIMMUNE** a=20 |
| D63 | Mist Veil | Veil | **ALLYINVIS** a=45 | **ALLYINVIS** a=67 + **SLOW** a=25 c=30 | **ALLYINVIS** a=33 + **HASTE** a=20 c=60 |
| D64 | Glare | Veil | **SLOW** b=25 c=60 | **SLOW** b=37 c=90 + **DISARM** a=15 | **SLOW** b=18 c=45 + **STATDOWN** b=15 c=60 |
| D65 | Smoke Screen | Veil | **BADR** a=30 b=60 | **BADR** a=45 b=90 + **SHOVE** a=8000 | **BADR** a=22 b=45 + **SELFINVIS** a=20 |
| D66 | Dazzle | Veil | **DISARM** a=30 | **DISARM** a=45 + **AMPLIFY** a=12 b=40 | **DISARM** a=22 + **SLOW** a=15 c=30 |
| D67 | Eclipse | Veil | **STATDOWN** a=3 b=25 c=120 | **STATDOWN** a=3 b=37 c=180 + **SILENCE** a=25 | **STATDOWN** a=3 b=18 c=90 + **HEALCUT** a=20 b=90 |
| D68 | Nightfall | Veil | **SELFINVIS** a=90 | **SELFINVIS** a=135 + **EXECUTE** a=30 b=30 | **SELFINVIS** a=67 + **DISARM** a=20 |
| D69 | Lantern | Veil | **STATUP** a=4 b=10 c=180 | **STATUP** a=4 b=15 c=270 + **MARK** a=30 b=180 | **STATUP** a=4 b=7 c=135 + **LENS** a=10 |
| D70 | Glass Wall | Matter | **WALL** | **ENCASE** a=45 b=30 + **MARK** a=20 b=120 | **WALL** + **SLOW** a=25 c=60 |
| D71 | Prism | Matter | **PRISM** a=3 b=40 | **PRISM** a=3 b=60 + **STATDOWN** a=1 b=10 c=60 | **PRISM** a=3 b=30 + **TRUE** a=8 |
| D72 | Mirror Shard | Matter | **REFLECTDMG** a=15 b=60 | **REFLECTDMG** a=22 b=90 + **KNOCK** a=10000 | **REFLECTDMG** a=11 b=45 + **SHIELD** a=30 b=90 |
| D73 | Thorn Glass | Matter | **REFLECTDMG** a=25 b=40 | **REFLECTDMG** a=37 b=60 + **DOT** a=10 b=60 | **REFLECTDMG** a=18 b=30 + **SLOW** a=20 c=40 |
| D74 | Bedrock | Matter | **CCIMMUNE** a=30 | **CCIMMUNE** a=45 + **ROOT** a=15 | **CCIMMUNE** a=22 + **SHELTER** a=15 b=60 |
| D75 | Quartz Lens | Matter | **LENS** a=20 | **LENS** a=30 + **SPLASH** a=20 b=15000 | **LENS** a=15 + **AMPLIFY** a=10 b=40 |
| D76 | Splinter | Matter | **PRISM** a=2 b=60 | **PRISM** a=2 b=90 + **DOT** a=6 b=120 | **PRISM** a=2 b=45 + **SHATTER** |
| D77 | Obsidian | Matter | **STATDOWN** a=1 b=35 c=60 | **STATDOWN** a=1 b=52 c=90 + **KNOCK** a=14000 | **STATDOWN** a=1 b=26 c=45 + **ROOT** a=15 |
| D78 | Crystal Skin | Matter | **SHIELD** a=60 b=240 | **SHIELD** a=90 b=360 + **ENCASE** a=20 b=20 | **SHIELD** a=45 b=180 + **REFLECTDMG** a=10 b=90 |
| D79 | Keystone | Matter | **CCIMMUNE** a=60 | **CCIMMUNE** a=90 + **STUN** a=14 | **CCIMMUNE** a=45 + **CLEANSE** |
| D80 | Hungry | Hunger | **HEALCUT** a=50 b=40 | **HEALCUT** a=75 b=60 + **DOT** a=12 b=120 | **HEALCUT** a=37 b=30 + **SLOW** a=15 c=40 |
| D81 | Siphon | Hunger | **SIPHON** a=10 | **SIPHON** a=15 + **HEALCUT** a=20 b=60 | **SIPHON** a=8 + **SLOW** a=15 c=60 |
| D82 | Tithe | Hunger | **TITHE** | **TITHE** + **HEALCUT** a=30 b=60 | **TITHE** + **SLOW** a=20 c=40 |
| D83 | Leech | Hunger | **VAMP** a=20 b=120 | **VAMP** a=30 b=180 + **HEALCUT** a=30 b=90 | **VAMP** a=15 b=90 + **DRAIN** a=8 b=90 |
| D84 | Famine | Hunger | **HEALCUT** a=80 b=60 | **HEALCUT** a=100 b=90 + **EXECUTE** a=20 b=30 | **HEALCUT** a=60 b=45 + **STATDOWN** a=4 b=50 c=120 |
| D85 | Drain | Hunger | **DRAIN** a=15 b=120 | **DRAIN** a=22 b=180 + **SLOW** a=20 c=60 | **DRAIN** a=11 b=90 + **HEALCUT** a=30 b=60 |
| D86 | Gnaw | Hunger | **DOT** a=15 b=240 | **DOT** a=22 b=360 + **STATDOWN** a=1 b=15 c=120 | **DOT** a=11 b=180 + **SPLASH** a=15 b=15000 |
| D87 | Marrow | Hunger | **STATDOWN** a=4 b=100 c=240 | **STATDOWN** a=4 b=150 c=360 + **DOT** a=10 b=120 | **STATDOWN** a=4 b=75 c=180 + **HEALCUT** a=20 b=120 |
| D88 | Starve | Hunger | **HEALCUT** a=30 b=360 | **HEALCUT** a=45 b=540 + **VAMP** a=10 b=120 | **HEALCUT** a=22 b=270 + **DOT** a=6 b=240 |
| D89 | Feast | Hunger | **SIPHON** a=20 | **SIPHON** a=30 + **EXECUTE** a=20 b=25 | **SIPHON** a=15 + **DOT** a=10 b=120 |
| D90 | Rift Storm | Chaos | **STORM** a=20000 | **STORM** a=30000 + **STATICBURST** a=20 | **STORM** a=15000 + **AMPLIFY** a=8 b=40 |
| D91 | Dice | Chaos | **DICE** | **DICE** + **AMPLIFY** a=5 b=30 | **DICE** + **SHOVE** a=6000 |
| D92 | Unstable | Chaos | **UNSTABLE** a=130 b=4 | **UNSTABLE** a=150 b=6 + **LENS** a=10 | **UNSTABLE** a=130 b=8 + **SHOVE** a=10000 |
| D93 | Wild Echo | Chaos | **WILDECHO** | **WILDECHO** + **SHOVE** a=9000 | **WILDECHO** + **STATICBURST** a=15 |
| D94 | Coinflip | Chaos | **COINFLIP** a=20 | **COINFLIP** a=30 + **SHOVE** a=5000 | **COINFLIP** a=20 + **STORM** a=6000 |
| D95 | Static Burst | Chaos | **STATICBURST** a=30 | **STATICBURST** a=45 + **CHAIN** a=25000 b=60 | **STATICBURST** a=22 + **SHOVE** a=7000 |
| D96 | Scramble | Chaos | **SCRAMBLE** | **SCRAMBLE** + **STUN** a=10 | **SCRAMBLE** + **DISARM** a=20 |
| D97 | Entropy | Chaos | **ENTROPY** a=40 | **ENTROPY** a=60 + **STORM** a=7000 | **ENTROPY** a=30 + **HEALCUT** a=15 b=60 |
| D98 | Kaleidoscope | Chaos | **KALEIDO** | **KALEIDO** + **ECHO** a=25 b=30 | **KALEIDO** + **STORM** a=8000 |
| D99 | Calm Eye | Chaos | **VENT** a=15 | **VENT** a=12 + **SLOW** a=15 b=15 c=60 | **VENT** a=8 + **CLEANSE** |

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
| UNDYING | A | Cannot drop below 1 HP for a (ticks); once per ally per 20 s |
| STATUP | A | +b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 crit chance) for c (ticks) |
| HASTE | A | +a% move speed and +b% attack speed for c (ticks) |
| RANGE | A | +a attack range for b (ticks) |
| GHOST | A | Walks through walls for a (ticks) |
| FOLD | A | Allies entering come out of the nearest other Passage rift |
| CLEANSE | A | Every crowd control is removed |
| ALLYINVIS | A | Invisible for a (ticks) |
| BADR | A | -a% damage from basic attacks for b (ticks) |
| REFLECTDMG | A | Reflects a% of damage taken for b (ticks) |
| CCIMMUNE | A | Immune to crowd control for a (ticks); once per ally per 20 s |
| CDDOWN | A | Skill cooldowns -a% for b (ticks) |
| SELFINVIS | S | Vesper is invisible while inside, lasting a (ticks) |
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
| RICOCHET | E | The nearest other enemy within 40000 takes a% of the hit |

## Family-pair bonuses (orb combos)

Two orbs of different families spent within a second of each other trigger their pair's bonus.

| Families | Bonus | Effect | Strength |
|---|---|---|---|
| Control + Control | Pure Control Lock | The bound target is rooted 20 ticks after the shot | 25 |
| Control + Ruin | Control-Ruin Execute Window | +10% damage when the target is already controlled | 10 |
| Control + Motion | Control-Motion Crush | A recently displaced target is stunned 12 ticks | 12 |
| Control + Haven | Control-Haven Sanctuary | Allies near the exit become immune to crowd control for 30 ticks | 30 |
| Control + Passage | Control-Passage Prison | The target is rooted 30 ticks and dragged to the exit | 30 |
| Control + Time | Control-Time Lock | The bound target is rooted 20 ticks after the shot | 20 |
| Control + Veil | Control-Veil Sanctuary | Allies near the exit become immune to crowd control for 30 ticks | 30 |
| Control + Matter | Control-Matter Prison | The target is rooted 30 ticks and dragged to the exit | 30 |
| Control + Hunger | Control-Hunger Execute Window | +10% damage when the target is already controlled | 10 |
| Control + Chaos | Control-Chaos Crush | A recently displaced target is stunned 12 ticks | 12 |
| Ruin + Ruin | Pure Ruin Overcharge | The shot jumps to a second enemy at 50% | 62 |
| Ruin + Motion | Ruin-Motion Shrapnel | A recently displaced target takes 3% of max HP | 30 |
| Ruin + Haven | Ruin-Haven Siphon | Allies near the exit heal 10% of the damage | 10 |
| Ruin + Passage | Ruin-Passage Leak | The exit dimension also pulses at the farthest rift | 1 |
| Ruin + Time | Ruin-Time Execute Window | +10% damage when the target is already controlled | 10 |
| Ruin + Veil | Ruin-Veil Siphon | Allies near the exit heal 10% of the damage | 10 |
| Ruin + Matter | Ruin-Matter Leak | The exit dimension also pulses at the farthest rift | 1 |
| Ruin + Hunger | Ruin-Hunger Overcharge | The shot jumps to a second enemy at 50% | 50 |
| Ruin + Chaos | Ruin-Chaos Shrapnel | A recently displaced target takes 3% of max HP | 30 |
| Motion + Motion | Pure Motion Vortex | Enemies near the target are dragged to it | 43 |
| Motion + Haven | Motion-Haven Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Motion + Passage | Motion-Passage Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Motion + Time | Motion-Time Crush | A recently displaced target is stunned 12 ticks | 12 |
| Motion + Veil | Motion-Veil Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Motion + Matter | Motion-Matter Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Motion + Hunger | Motion-Hunger Shrapnel | A recently displaced target takes 3% of max HP | 30 |
| Motion + Chaos | Motion-Chaos Vortex | Enemies near the target are dragged to it | 35 |
| Haven + Haven | Pure Haven Haven | Allies near the exit heal 3% of max HP | 37 |
| Haven + Passage | Haven-Passage Gift | Allies near the exit gain a 5% max HP shield | 50 |
| Haven + Time | Haven-Time Sanctuary | Allies near the exit become immune to crowd control for 30 ticks | 30 |
| Haven + Veil | Haven-Veil Haven | Allies near the exit heal 3% of max HP | 30 |
| Haven + Matter | Haven-Matter Gift | Allies near the exit gain a 5% max HP shield | 50 |
| Haven + Hunger | Haven-Hunger Siphon | Allies near the exit heal 10% of the damage | 10 |
| Haven + Chaos | Haven-Chaos Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Passage + Passage | Pure Passage Paradox | A mirrored copy of the shot fires 12 ticks later at 50% | 62 |
| Passage + Time | Passage-Time Prison | The target is rooted 30 ticks and dragged to the exit | 30 |
| Passage + Veil | Passage-Veil Gift | Allies near the exit gain a 5% max HP shield | 50 |
| Passage + Matter | Passage-Matter Paradox | A mirrored copy of the shot fires 12 ticks later at 50% | 50 |
| Passage + Hunger | Passage-Hunger Leak | The exit dimension also pulses at the farthest rift | 1 |
| Passage + Chaos | Passage-Chaos Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Time + Time | Pure Time Lock | The bound target is rooted 20 ticks after the shot | 25 |
| Time + Veil | Time-Veil Sanctuary | Allies near the exit become immune to crowd control for 30 ticks | 30 |
| Time + Matter | Time-Matter Prison | The target is rooted 30 ticks and dragged to the exit | 30 |
| Time + Hunger | Time-Hunger Execute Window | +10% damage when the target is already controlled | 10 |
| Time + Chaos | Time-Chaos Crush | A recently displaced target is stunned 12 ticks | 12 |
| Veil + Veil | Pure Veil Haven | Allies near the exit heal 3% of max HP | 37 |
| Veil + Matter | Veil-Matter Gift | Allies near the exit gain a 5% max HP shield | 50 |
| Veil + Hunger | Veil-Hunger Siphon | Allies near the exit heal 10% of the damage | 10 |
| Veil + Chaos | Veil-Chaos Escort | Allies near the exit gain +30% move speed for 60 ticks | 30 |
| Matter + Matter | Pure Matter Paradox | A mirrored copy of the shot fires 12 ticks later at 50% | 62 |
| Matter + Hunger | Matter-Hunger Leak | The exit dimension also pulses at the farthest rift | 1 |
| Matter + Chaos | Matter-Chaos Fold-throw | The target is thrown to the first rift of the chain | 1 |
| Hunger + Hunger | Pure Hunger Overcharge | The shot jumps to a second enemy at 50% | 62 |
| Hunger + Chaos | Hunger-Chaos Shrapnel | A recently displaced target takes 3% of max HP | 30 |
| Chaos + Chaos | Pure Chaos Vortex | Enemies near the target are dragged to it | 43 |

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
