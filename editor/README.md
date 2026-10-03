# TFM2 Database Editor

A local web app for editing Teamfight Manager 2 career saves and `.tfm2db` databases.

## Start it

Double-click **`Start Editor.bat`**. It opens http://localhost:7272 in your browser and lists your files automatically:

| Where | What |
|---|---|
| `%APPDATA%\TeamSamoyed\TeamfightManager2\data\save_*.data` | Your career saves (the autosave lives here) |
| `%APPDATA%\TeamSamoyed\TeamfightManager2\data\custom_database.tfm2db` | The custom database the game uses for new careers |
| `C:\Games\tfm2\database\*.tfm2db` | Databases you exported from the game |

You need Node.js (https://nodejs.org) for this. Without Node, the .bat opens `index.html` directly. You then pick files yourself, and saves either go straight back to the file (Chrome/Edge) or are downloaded.

The editor finds the Steam copy of the game on its own (through Steam's library list), so it can put Skill Lab mods in the game's `mods` folder. The console window shows which game folder it found. If it's wrong, start it with the path: `"Start Editor.bat" --game "D:\SteamLibrary\steamapps\common\Teamfight Manager 2"`.

## What you can edit

- **Players:** name, age, 8 skills, 4 play-style traits, 5 position proficiencies, 11 hidden attributes (potential, stamina, condition, stress…), languages, contract (team, dates, salary, transfer fee) and appearance
- **Teams:** name, logo reference, stadium name, manager name and three money fields
- **Bulk edit:** set, add, multiply or clamp any stat for every player that matches the current search and filters (for example, "+5 potential for all supports on my team")
- **Champions:** base stats, growth per level and the numbers of every ability (damage, ratios, cooldowns, ranges, durations…) for all 64 champions, including the data-driven ones (Crossbowman, Nightmare, Alchemist, Sand Mage). **Patch preview** lists your buffs and nerfs before you save.
- **Skill Lab:** rebuild what a champion's skills *do* out of building blocks — projectiles, dashes, teleports, stuns, knock-ups, pulls, shields, heals, zones, buffs, damage over time — or create a brand-new champion. See below.

## Skill Lab (custom skills)

Skill Lab uses the game's official mod system ([TeamfightManager2Mod](https://github.com/teamsamoyed/TeamfightManager2Mod)). Nothing is written into your save. Instead it creates a mod in the game's install folder, in `mods\tfm2_custom\` (for Steam, usually `C:\Program Files (x86)\Steam\steamapps\common\Teamfight Manager 2\mods\`):

- `champion\<id>.data_champion` holds each reworked or new champion. Using an existing champion's id replaces that champion while the mod is enabled.
- `text\champion.i18n` holds the tooltips, and `mod.override_info` merges them into the game's text.
- The previous version of the mod is backed up in `editor\backups\mods\` before every save.

How to use it:

1. Open the **Skill Lab** tab (it also works without opening a save). Pick a champion under "Rework a champion…".
2. Change blocks. For example, switch *Airborne* to *Stun* with the block's type menu, or use **Swap effect type…** to change every one at once. Add *Teleport* under a projectile's "When it ends" so the champion blinks to where the projectile landed. You can also start from a **Quick recipe**.
   - **New champion from a preset…** loads a complete custom champion (for example Minato: Flying Raijin, Rasengan and Kurama Mode) that you can then tweak.
   - **Start over (remove all)** empties the mod; reworked champions go back to normal after you save.
3. Click **Save mod**. In the game, go to the title screen → **Mods** → enable "My custom skills" → restart the game.

Built-in champions (like Lancer) are rebuilt from their numbers as standard blocks. Their unique hand-coded mechanics can't be copied, so they're replaced. Crossbowman, Nightmare, Alchemist and Sand Mage start from their exact data. Disabling the mod brings every original champion back.

## Career save vs. exported database

- To change **player stats, team money or champions in the career you're playing**, open the career save (`save_*.data`) and save it with the game closed.
- Importing an exported `.tfm2db` into a running career only brings over **Team Info** (names, logos) and **Game Numbers** (champions, leagues). Player stats and money are **not** imported, which is why only names changed when you tried it.
- A save stores two copies of the champion numbers: the current numbers, and the base numbers that the automatic balance patches are measured against. With "Keep my numbers through future auto-patches" on (the default), the editor writes both, so the next auto-patch starts from your numbers.

## Safety

- **Close the game before saving.** If you don't, its autosave will overwrite your edits. The editor warns you if the game is running.
- Before every overwrite, the previous file is copied to `editor\backups\`.
- "Save as…" writes a copy and leaves the original untouched.
- After you save an exported database (`.tfm2db` from the database folder), import it in the game's database menu.

## Notes / limitations

- Changing a player's contract team only rewrites the contract. The team's lineup may not follow, so do real transfers in the game.
- Language IDs are shown as numbers because the game doesn't store their names in the save.
- One of the three team money fields is your bank balance. Compare them with what the game shows before you edit.
- Old news items and match history keep the old names after you rename a player or team.
- You can't add or delete players or teams here. The game's built-in database editor can do that.
- Champion edits replace the numbers in the save, but they aren't added to the in-game patch history, so they won't show up as a patch note.
- After a game update, `server.js` re-reads the champion data (`gamedata.js`) from `bundle.game_data` automatically. You can also run `node extract-gamedata.js`.

File format: a `TFM2` header (kind, timestamp, gzip length, CRC32), then a preview block, then a gzip-compressed Rust bincode payload. Credit to [eminyilmazz/tfm2-real-teams-and-rosters](https://github.com/eminyilmazz/tfm2-real-teams-and-rosters) for the container layout.
