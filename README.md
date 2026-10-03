# Teamfight Manager 2 Mod & Tool Suite (`tfm2_mods`)

A complete modding, database editing, sprite authoring, and native code development suite for **[Teamfight Manager 2](https://store.steampowered.com/app/3009300/Teamfight_Manager_2/)**.

---

## What's in this Repository

| Folder | Description |
|---|---|
| **[`mods/`](mods/)** | **Playable mod packages** ready to drop into the game's `mods/` directory. Includes 11 custom champions (Gojo, Minato, DIO, Steve, Frieren, Fern, Stark, David Martinez, Darth Vader, V1, etc.) and native AI. |
| **[`editor/`](editor/)** | **TFM2 Database Editor & Skill Lab** — local web application for editing career saves, database rosters, and custom skills using node building blocks. |
| **[`native/`](native/)** | **Native Rust AI mod** (`tfm2_custom_ai`) and SDK (`mod-api-stable`) for advanced combat logic and custom mechanics compiled to `tfm2_custom_ai.dll`. |
| **[`Sprite kit/`](Sprite%20kit/)** | Authoring templates, frame guides, and extracted base champion sheets (48×56 px pixel art, anim JSONs, VFX). |
| **[`database/`](database/)** | Staging folder for exported `.tfm2db` databases to open and modify inside the editor. |
| **[`Claude outputs/`](Claude%20outputs/)** | Champion roster guide PDF, visual preview GIFs, sound effects, and development helper scripts. |

---

## Quick Start: Installing the Mods

1. Locate your **Teamfight Manager 2** install directory:
   - Default Steam location: `C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager2`
2. Copy the folders inside **[`mods/`](mods/)** directly into your game's `mods` folder:
   - Target: `Teamfight Manager2\mods\`
3. Start the game, go to the title screen → **Mods**, and enable the mods you wish to play with.
4. If enabling `tfm2_custom_ai`, accept the code-mod warning and restart the game.

### Included Mod Champions
- **Steve** (`tfm2_blockcraft`): Building, block combat, pickaxe abilities.
- **Minato & Sand Mage Rework** (`tfm2_custom`): Flying Raijin, Rasengan, Kurama Mode.
- **Custom Native AI** (`tfm2_custom_ai`): Dedicated C-ABI native AI decision routines for custom champions.
- **David Martinez** (`tfm2_cyberpunk`): Sandevistan dashes and high-speed attacks.
- **Frieren, Fern & Stark** (`tfm2_frieren`): Zoltraak, barrier magic, ghoul transformations, and cleave attacks.
- **Satoru Gojo** (`tfm2_jjk`): Limitless, Blue, Red, Hollow Purple, and Infinite Void domain expansion.
- **DIO** (`tfm2_jojo`): The World time stop, knife throws, and road roller rush.
- **Darth Vader** (`tfm2_starwars`): Lightsaber throws, Force choke, and deflect mechanics.
- **V1** (`tfm2_ultrakill`): Coin tossing, ricochet shots, and blood heal dashes.
- **Toon & Valorant Packs** (`tfm2_toon`, `tfm2_valorant`).

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
