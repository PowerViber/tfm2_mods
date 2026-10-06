# Teamfight Manager 2 Mod & Tool Suite (`tfm2_mods`)

A complete modding, database editing, sprite authoring, and native code development suite for **[Teamfight Manager 2](https://store.steampowered.com/app/3009300/Teamfight_Manager_2/)**.

---

## What's in this Repository

| Folder | Description |
|---|---|
| **[`mods/`](mods/)** | **Playable mod packages** ready to drop into the game's `mods/` directory: 11 custom champions (Minato, Gojo, DIO, David Martinez, V1, Darth Vader, Frieren, Steve, Omen, Scribble, Levi) and the native rules mod they rely on. |
| **[`editor/`](editor/)** | **TFM2 Database Editor & Skill Lab** — local web application for editing career saves, database rosters, and custom skills using node building blocks. |
| **[`native/`](native/)** | **Native Rust AI mod** (`tfm2_custom_ai`) and SDK (`mod-api-stable`) for advanced combat logic and custom mechanics compiled to `tfm2_custom_ai.dll`. |
| **[`Sprite kit/`](Sprite%20kit/)** | Authoring templates, frame guides, and extracted base champion sheets (48×56 px pixel art, anim JSONs, VFX). |
| **[`database/`](database/)** | Staging folder for exported `.tfm2db` databases to open and modify inside the editor. |
| **[`Claude outputs/`](Claude%20outputs/)** | Champion roster guide PDF, visual preview GIFs, sound effects, and development helper scripts. |

---

## Quick Start: Installing the Mods

**Easiest:** close the game and the editor, then double-click **`Update game and editor.bat`** in this folder. It:
- pulls the latest version (if this is a git clone);
- finds the game (the default Steam folder, or asks you);
- backs up your installed mods to `backups\game_mods_<time>`;
- installs every mod. It keeps your Map tab plans (`tactics.txt`) and removes files from older versions (such as a separate `tfm2_levi`; Levi is in `tfm2_custom` now);
- checks the native DLL and offers to start the editor.

To point it at another game folder, run it from a command prompt with the folder as its argument: `"Update game and editor.bat" "D:\SteamLibrary\steamapps\common\Teamfight Manager2"`.

**By hand:**
1. Locate your **Teamfight Manager 2** install directory:
   - Default Steam location: `C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2`
2. Copy the folders inside **[`mods/`](mods/)** directly into your game's `mods` folder:
   - Target: `Teamfight Manager2\mods\`
3. Start the game, go to the title screen → **Mods**, and enable the mods you wish to play with.
4. If enabling `tfm2_custom_ai`, accept the code-mod warning and restart the game.

### Included Mod Champions
| Mod folder | Champion | Highlights |
|---|---|---|
| `tfm2_custom` | **Minato** | Flying Raijin kunai teleports, Rasengan, Kurama Mode, dodge stacks |
| `tfm2_jjk` | **Gojo** | Blue / Red flags, Hollow Purple, Infinity, Unlimited Void domain |
| `tfm2_jojo` | **DIO** | Stand Out / Stand In modes, knives, ZA WARUDO time stop |
| `tfm2_cyberpunk` | **David Martinez** | Sandevistan, cyberpsychosis meter, gravity ult |
| `tfm2_ultrakill` | **V1** | Coin ricochets, parry + shotgun, railgun |
| `tfm2_starwars` | **Darth Vader** | Saber throw, Force choke, rage |
| `tfm2_frieren` | **Frieren** | Zoltraak; summons Fern and Stark (Stark replaces the Necromancer's ghoul sprite while enabled) |
| `tfm2_blockcraft` | **Steve** | Pearl / TNT / golden apple, fishing rod, boat wall ult |
| `tfm2_valorant` | **Omen** | Smokes, Paranoia, Shadow Step, Buy Phase (restyles the Bombardier sprite for his shadows) |
| `tfm2_toon` | **Scribble** | 35-spell toon mage that learns per player |
| `tfm2_custom` | **Levi**, **Emperor Isliid**, **Aegis Zero** | Levi: cable flyer with mastery ranks. Isliid: seven swords that engrave formations, mastery sigils. Aegis Zero: Wings of Light |
| `tfm2_custom_ai` | *(native rules)* | Required by every champion above: their scripted mechanics, Flash, farming, Mod Power, Map tab plans |

Most kits only work fully with **`tfm2_custom_ai`** enabled (it shows in the Mod Manager as "Gojo & Minato rules (native)").

The **[`test mod (copy into mods)/`](test%20mod%20(copy%20into%20mods)/)** folder holds `tfm2_test_sandmage`, a small Skill Lab example (Sand Mage ult also stuns). It is not needed to play.

---

## Using the Database & Skill Lab Editor

1. Open **[`editor/`](editor/)**.
2. Double-click **`Start Editor.bat`**.
3. It launches a local web server at **http://localhost:7272** and detects your active saves:
   - `%APPDATA%\TeamSamoyed\TeamfightManager2\data\save_*.data` (career saves)
   - `%APPDATA%\TeamSamoyed\TeamfightManager2\data\custom_database.tfm2db` (custom game database)
   - `C:\Games\tfm2\database\*.tfm2db` (exported databases)
4. Use **Skill Lab** to create or modify custom champions from visual building blocks (projectiles, dashes, teleports, stuns, knock-ups, buffs, heals) and click **Save mod** to deploy directly to the game.

For detailed editor documentation, see **[`editor/README.md`](editor/README.md)**.

---

## Building Native AI from Source

If you want to modify or rebuild `tfm2_custom_ai.dll`:
1. Ensure Rust is installed ([https://rustup.rs](https://rustup.rs)).
2. Run **[`native/build.bat`](native/build.bat)**.
3. The script configures the `stable-x86_64-pc-windows-gnu` toolchain, compiles `tfm2_custom_ai.dll`, and automatically installs it into `Teamfight Manager2\mods\tfm2_custom_ai\`.

---

## Sprite Authoring & Animation

To create or edit champion sprites:
- See **[`Sprite kit/README.txt`](Sprite%20kit/README.txt)**.
- Use `NEW CHAMPION template.png` and `NEW CHAMPION template_guide.png` for reference dimensions and anchor alignments.

---

## Playtest Notes

**[`docs/playtest-notes.md`](docs/playtest-notes.md)** is the round-by-round design log: confirmed engine behaviour, every balance change, and the open "still to watch" list.
