# Vesper, the Rift Marksman: catalogue

Generated from `Claude outputs/vesper/catalogue.json`. Everything here is available at every mastery rank; rank only changes how well the athlete plans with it.

## Rift types

| Rift | Charges | Life (ticks) | Size | What the shape does |
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

| Technique | Group | Rifts | Strain | Per-target cooldown | What happens |
|---|---|---|---|---|---|
| Chain Fire | offense | Tear, Sky Rift, Mirror Pane, Twin Rift, Anchor Rift, Wandering Rift | 0 | 0 | While a chain is live every basic attack travels through it |
| Crossfire | offense | Tear | 10 | 0 | One attack splits out of up to 3 exits at 3 targets, at 45% each, for 180 ticks |
| Bank Shot | offense | Mirror Pane | 6 | 0 | A Mirror Pane in the chain focuses the next link (x1.2) and lets shots pass walls |
| Rift Rain | offense | Sky Rift | 14 | 0 | 6 packets fall over the Sky Rift during 60 ticks: 25 + 25% AD each |
| Slash Volley | offense | Slash | 12 | 0 | 5 packets fan out along the Slash: 20 + 30% AD each |
| Implode | offense | Tear, Sinkhole, Sky Rift, Twin Rift, Seed Rift, Anchor Rift, Wandering Rift, Maw | 10 | 0 | Close a rift early: enemies are dragged to its centre and take 20 + 40% AD plus its dimension twice |
| Domino | offense | Tear, Twin Rift | 8 | 0 | Implode every linked rift in turn, 15 ticks apart |
| Overload Shot | offense | Tear, Twin Rift, Sky Rift, Anchor Rift | 0 | 0 | Spend strain: the next chain shot gains +1% per point spent (up to 60) |
| Rift Step | engage | Tear, Twin Rift | 10 | 240 | Vesper steps into a rift within 24000 and out of another |
| Ally Gate | engage | Tear, Twin Rift | 15 | 180 | Allies touching the entrance come out of the exit |
| Sky Drop | engage | Sky Rift | 18 | 180 | A Gate exit from above: the ally lands and stuns enemies within 20000 for 24 ticks |
| Kidnap | engage | Tear, Twin Rift | 18 | 600 | Swallow an enemy for 30 ticks and spit them out of the other rift |
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
| Escort | team | Anchor Rift | 8 | 0 | An Anchor Rift on an ally: chain fire comes out of the diving ally |
| Rotation Gate | team | Twin Rift | 20 | 180 | A long Twin gate that moves allies across the map toward the next fight |
| Seed Trap | team | Seed Rift | 10 | 600 | A hidden Seed on a predicted path: opens with its dimension and swallows the victim |
| Sweep | team | Wandering Rift | 12 | 0 | Drag a dimension across a whole line of enemies |
| Borrow | team | Tear | 14 | 0 | Open in a far atlas cell: the near rift takes on that dimension for 120 ticks |
| Stitch | team | Tear | 20 | 0 | Sew two Tears into one larger rift holding both dimensions and their bonus |

## Dimensions

| ID | Dimension | Family | Effect | Numbers |
|---|---|---|---|---|
| D00 | Void | Control | BANISH: Vanish from the fight: untargetable and unable to act for a (ticks) | a=60 b=0 c=0 |
| D01 | Silent Realm | Control | SILENCE: Cannot use skills for a (ticks) | a=60 b=0 c=0 |
| D02 | Unarmed World | Control | DISARM: Cannot basic attack for a (ticks) | a=60 b=0 c=0 |
| D03 | Frozen Moment | Control | STUN: Stunned for a (ticks) | a=30 b=0 c=0 |
| D04 | Anchor | Control | ROOT: Rooted in place for a (ticks) | a=60 b=0 c=0 |
| D05 | Heavy World | Control | SLOW: -a% move speed and -b% attack speed while inside, lasting c (ticks) | a=35 b=20 c=40 |
| D06 | Leaden Sky | Control | SLOW: -a% move speed and -b% attack speed while inside, lasting c (ticks) | a=50 b=0 c=90 |
| D07 | Lull | Control | STUN: Stunned for a (ticks) | a=18 b=0 c=0 |
| D08 | Gloom Cage | Control | ROOT: Rooted in place for a (ticks) | a=90 b=0 c=0 |
| D09 | Tar Pit | Control | SLOW: -a% move speed and -b% attack speed while inside, lasting c (ticks) | a=25 b=10 c=240 |
| D10 | Static Realm | Ruin | AMPLIFY: Takes +a% damage from everyone for b (ticks) | a=15 b=40 c=0 |
| D11 | Ember | Ruin | DOT: Burns a per mille of max HP per second for b (ticks) | a=20 b=180 c=0 |
| D12 | True Rift | Ruin | TRUE: Packets passing through deal +a% of their damage as true damage | a=15 b=0 c=0 |
| D13 | Shatter | Ruin | SHATTER: Every shield is stripped | a=0 b=0 c=0 |
| D14 | Acid Rain | Ruin | DOT: Burns a per mille of max HP per second for b (ticks) | a=10 b=360 c=0 |
| D15 | Fracture | Ruin | AMPLIFY: Takes +a% damage from everyone for b (ticks) | a=25 b=60 c=0 |
| D16 | Brittle | Ruin | STATDOWN: -b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 regen) for c (ticks) | a=1 b=25 c=120 |
| D17 | Hex Ward | Ruin | STATDOWN: -b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 regen) for c (ticks) | a=2 b=25 c=120 |
| D18 | Cinder | Ruin | DOT: Burns a per mille of max HP per second for b (ticks) | a=30 b=120 c=0 |
| D19 | Razor Light | Ruin | TRUE: Packets passing through deal +a% of their damage as true damage | a=25 b=0 c=0 |
| D20 | Mirror Realm | Motion | REFLECT: Reflected to the far side of the rift, wall-clamped | a=0 b=0 c=0 |
| D21 | Drift | Motion | PULL: Dragged a% of the way to the rift centre | a=35 b=0 c=0 |
| D22 | Gale | Motion | KNOCK: Thrown a units away from the rift centre | a=20000 b=0 c=0 |
| D23 | Crosswind | Motion | SHOVE: Shoved a units in a seeded random direction | a=15000 b=0 c=0 |
| D24 | Tether | Motion | TETHER: Leaving the zone snaps them back to its edge once | a=0 b=0 c=0 |
| D25 | Swap Space | Motion | SWAPSPACE: The two fighters nearest the centre trade places | a=0 b=0 c=0 |
| D26 | Undertow | Motion | PULL: Dragged a% of the way to the rift centre | a=60 b=0 c=0 |
| D27 | Repulse | Motion | KNOCK: Thrown a units away from the rift centre | a=32000 b=0 c=0 |
| D28 | Whirl | Motion | SHOVE: Shoved a units in a seeded random direction | a=9000 b=0 c=0 |
| D29 | Slipstream | Motion | GRAB: Dragged a units toward Vesper | a=20000 b=0 c=0 |
| D30 | Shelter | Haven | SHELTER: Takes -a% damage for b (ticks) | a=15 b=40 c=0 |
| D31 | Bloom | Haven | HEAL: Heals a per mille of max HP per second for b (ticks); b = 0 heals once | a=20 b=40 c=0 |
| D32 | Bulwark | Haven | SHIELD: Shield of a per mille of max HP for b (ticks), inside the shared support cap | a=80 b=180 c=0 |
| D33 | Last Breath | Haven | UNDYING: Cannot drop below 1 HP for a (ticks); once per ally per 20 s | a=60 b=0 c=0 |
| D34 | Sanctum | Haven | SHELTER: Takes -a% damage for b (ticks) | a=25 b=40 c=0 |
| D35 | Spring | Haven | HEAL: Heals a per mille of max HP per second for b (ticks); b = 0 heals once | a=40 b=0 c=0 |
| D36 | Aegis | Haven | SHIELD: Shield of a per mille of max HP for b (ticks), inside the shared support cap | a=150 b=60 c=0 |
| D37 | Mend | Haven | HEAL: Heals a per mille of max HP per second for b (ticks); b = 0 heals once | a=10 b=360 c=0 |
| D38 | Ward | Haven | STATUP: +b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 crit chance) for c (ticks) | a=2 b=25 c=120 |
| D39 | Iron Skin | Haven | STATUP: +b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 crit chance) for c (ticks) | a=1 b=25 c=120 |
| D40 | Folded Map | Passage | FOLD: Allies entering come out of the nearest other Passage rift | a=0 b=0 c=0 |
| D41 | Ghost Plane | Passage | GHOST: Walks through walls for a (ticks) | a=180 b=0 c=0 |
| D42 | Far Shore | Passage | RANGE: +a attack range for b (ticks) | a=15000 b=40 c=0 |
| D43 | Swift Plane | Passage | HASTE: +a% move speed and +b% attack speed for c (ticks) | a=30 b=25 c=180 |
| D44 | Tailwind | Passage | HASTE: +a% move speed and +b% attack speed for c (ticks) | a=45 b=0 c=90 |
| D45 | Long Reach | Passage | RANGE: +a attack range for b (ticks) | a=25000 b=60 c=0 |
| D46 | Stride | Passage | HASTE: +a% move speed and +b% attack speed for c (ticks) | a=20 b=0 c=360 |
| D47 | Skyway | Passage | GHOST: Walks through walls for a (ticks) | a=90 b=0 c=0 |
| D48 | Sprint | Passage | HASTE: +a% move speed and +b% attack speed for c (ticks) | a=60 b=0 c=40 |
| D49 | Courier Wind | Passage | HASTE: +a% move speed and +b% attack speed for c (ticks) | a=15 b=15 c=600 |
| D50 | Rewind | Time | REWIND: Snaps back to where it stood a ticks ago | a=120 b=0 c=0 |
| D51 | Echo | Time | ECHO: Packets passing through repeat at a% after b ticks | a=40 b=60 c=0 |
| D52 | Haste Clock | Time | HASTE: +a% move speed and +b% attack speed for c (ticks) | a=0 b=35 c=120 |
| D53 | Slow Clock | Time | SLOW: -a% move speed and -b% attack speed while inside, lasting c (ticks) | a=0 b=35 c=120 |
| D54 | Hourglass | Time | CDUP: Skill cooldowns +a% for b (ticks) | a=50 b=180 c=0 |
| D55 | Quickening | Time | CDDOWN: Skill cooldowns -a% for b (ticks) | a=25 b=180 c=0 |
| D56 | Afterimage | Time | ECHO: Packets passing through repeat at a% after b ticks | a=60 b=30 c=0 |
| D57 | Deja Vu | Time | REWIND: Snaps back to where it stood a ticks ago | a=60 b=0 c=0 |
| D58 | Long Second | Time | SLOW: -a% move speed and -b% attack speed while inside, lasting c (ticks) | a=20 b=20 c=180 |
| D59 | Prime Time | Time | STATUP: +b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 crit chance) for c (ticks) | a=0 b=15 c=180 |
| D60 | Hidden Side | Veil | SELFINVIS: Vesper is invisible while inside, lasting a (ticks) | a=40 b=0 c=0 |
| D61 | Blind Fog | Veil | STATDOWN: -b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 regen) for c (ticks) | a=0 b=25 c=60 |
| D62 | Clear Sky | Veil | CLEANSE: Every crowd control is removed | a=0 b=0 c=0 |
| D63 | Mist Veil | Veil | ALLYINVIS: Invisible for a (ticks) | a=45 b=0 c=0 |
| D64 | Glare | Veil | SLOW: -a% move speed and -b% attack speed while inside, lasting c (ticks) | a=0 b=25 c=60 |
| D65 | Smoke Screen | Veil | BADR: -a% damage from basic attacks for b (ticks) | a=30 b=60 c=0 |
| D66 | Dazzle | Veil | DISARM: Cannot basic attack for a (ticks) | a=30 b=0 c=0 |
| D67 | Eclipse | Veil | STATDOWN: -b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 regen) for c (ticks) | a=3 b=25 c=120 |
| D68 | Nightfall | Veil | SELFINVIS: Vesper is invisible while inside, lasting a (ticks) | a=90 b=0 c=0 |
| D69 | Lantern | Veil | STATUP: +b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 crit chance) for c (ticks) | a=4 b=10 c=180 |
| D70 | Glass Wall | Matter | WALL: A glass wall across the rift: enemies crossing it are thrown back | a=0 b=0 c=0 |
| D71 | Prism | Matter | PRISM: Packets passing through split into a at b% each | a=3 b=40 c=0 |
| D72 | Mirror Shard | Matter | REFLECTDMG: Reflects a% of damage taken for b (ticks) | a=15 b=60 c=0 |
| D73 | Thorn Glass | Matter | REFLECTDMG: Reflects a% of damage taken for b (ticks) | a=25 b=40 c=0 |
| D74 | Bedrock | Matter | CCIMMUNE: Immune to crowd control for a (ticks); once per ally per 20 s | a=30 b=0 c=0 |
| D75 | Quartz Lens | Matter | LENS: Packets passing through gain +a% damage | a=20 b=0 c=0 |
| D76 | Splinter | Matter | PRISM: Packets passing through split into a at b% each | a=2 b=60 c=0 |
| D77 | Obsidian | Matter | STATDOWN: -b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 regen) for c (ticks) | a=1 b=35 c=60 |
| D78 | Crystal Skin | Matter | SHIELD: Shield of a per mille of max HP for b (ticks), inside the shared support cap | a=60 b=240 c=0 |
| D79 | Keystone | Matter | CCIMMUNE: Immune to crowd control for a (ticks); once per ally per 20 s | a=60 b=0 c=0 |
| D80 | Hungry | Hunger | HEALCUT: -a% healing received for b (ticks) | a=50 b=40 c=0 |
| D81 | Siphon | Hunger | SIPHON: Allies inside heal a% of the damage Vesper deals there | a=10 b=0 c=0 |
| D82 | Tithe | Hunger | TITHE: Enemy shields inside are stripped and Vesper gains a shield inside the support cap | a=0 b=0 c=0 |
| D83 | Leech | Hunger | VAMP: Vesper heals a% of the damage she deals for b (ticks) | a=20 b=120 c=0 |
| D84 | Famine | Hunger | HEALCUT: -a% healing received for b (ticks) | a=80 b=60 c=0 |
| D85 | Drain | Hunger | DRAIN: Burns a per mille of max HP per second for b (ticks) and heals Vesper the same | a=15 b=120 c=0 |
| D86 | Gnaw | Hunger | DOT: Burns a per mille of max HP per second for b (ticks) | a=15 b=240 c=0 |
| D87 | Marrow | Hunger | STATDOWN: -b% of stat a (0 attack, 1 armour, 2 magic resist, 3 magic power, 4 regen) for c (ticks) | a=4 b=100 c=240 |
| D88 | Starve | Hunger | HEALCUT: -a% healing received for b (ticks) | a=30 b=360 c=0 |
| D89 | Feast | Hunger | SIPHON: Allies inside heal a% of the damage Vesper deals there | a=20 b=0 c=0 |
| D90 | Rift Storm | Chaos | STORM: Shoved a units in a seeded random direction, twice | a=20000 b=0 c=0 |
| D91 | Dice | Chaos | DICE: Each pulse rolls a dimension from another family | a=0 b=0 c=0 |
| D92 | Unstable | Chaos | UNSTABLE: Imbues at a% strength but every packet adds b strain | a=130 b=4 c=0 |
| D93 | Wild Echo | Chaos | WILDECHO: Packets passing through repeat at a seeded 20..80% after 30 ticks | a=0 b=0 c=0 |
| D94 | Coinflip | Chaos | COINFLIP: Each pulse flips: enemies take +a% damage, or allies take -a% damage | a=20 b=0 c=0 |
| D95 | Static Burst | Chaos | STATICBURST: Stunned for a seeded 0..a ticks | a=30 b=0 c=0 |
| D96 | Scramble | Chaos | SCRAMBLE: Two enemies inside trade places | a=0 b=0 c=0 |
| D97 | Entropy | Chaos | ENTROPY: Burns a seeded 0..a per mille of max HP per pulse | a=40 b=0 c=0 |
| D98 | Kaleidoscope | Chaos | KALEIDO: Packets leaving through it roll a random family-pair bonus | a=0 b=0 c=0 |
| D99 | Calm Eye | Chaos | VENT: Each pulse vents a strain | a=15 b=0 c=0 |

## Family-pair bonuses (exit twists)

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

0. **Spark**: Reflex chaos: picks rift types and techniques almost at random; ignores strain and enemy paths
1. **Tear**: Leads targets, including opening delays, for chain, Kidnap, Rescue and Seed placement
2. **Rift**: Safety: watches strain, keeps rifts off enemy paths, checks every exit, faces Mirror Panes correctly
3. **Gate**: Geometry: chain solver, Bank Shot and Slash angles, Crossfire splits, Implode and Domino timing on 2+ enemies
4. **Wanderer**: Survival: keeps a spare charge, sets a Lifeline, escapes dives, Pockets or Rescues allies
5. **Cartographer**: Team engage: Gates and Sky Drops timed with the frontliner, Escort, Kidnap and Hook into the team, Rotation Gates
6. **Riftwalker**: Priority and atlas: Kidnaps the carry not the tank, chokepoint Sinkholes and Curtains, Borrow and Stitch
7. **Worldbreaker**: Full sequences, 5 steps deep: spine upkeep, drift foresight, strain riding into Overload Shots

The #1 athlete of the Top 10 is shown as **Between Worlds** (rank 7 brain, visuals only).
