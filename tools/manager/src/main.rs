//! TFM2 Mod Manager: one exe for everything around the mods.
//!   1 Update everything: git pull (and switch to main when this folder is behind it), build the native DLL when Rust
//!     is installed, install every mod into the game with a backup, then check the install byte for byte.
//!   2 Check the game is up to date (changes nothing).
//!   3 Start the editor.
//!   4 Show the logs (the game's log.log, gundam_log / isliid_log / levi_log, the last manager log).
//!   5 Build the native DLL only.   6 Choose the game folder.
//! Arguments for scripts: --update --check --editor --build --logs --game <dir> --yes
//! Every line is also written to logs/manager-<time>.txt, and every failure says WHAT happened, WHY and HOW to fix it.
mod core;

use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static LOG: Mutex<Option<fs::File>> = Mutex::new(None);

/// Print a line and write it to the manager log.
fn say(line: &str) {
    println!("{line}");
    if let Ok(mut g) = LOG.lock() {
        if let Some(f) = g.as_mut() { let _ = writeln!(f, "{line}"); }
    }
}

/// A failure the user can act on.
struct Problem { what: String, why: String, how: String }

fn problem(what: impl Into<String>, why: impl Into<String>, how: impl Into<String>) -> Problem {
    Problem { what: what.into(), why: why.into(), how: how.into() }
}

fn report(p: &Problem) {
    say("");
    say("  !! PROBLEM");
    say(&format!("  WHAT: {}", p.what));
    say(&format!("  WHY:  {}", p.why));
    for (k, l) in p.how.lines().enumerate() { say(&format!("  {} {l}", if k == 0 { "HOW: " } else { "     " })); }
}

struct App { repo: PathBuf, game: Option<PathBuf>, yes: bool, log_path: PathBuf }

fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) }

/// Run a program, echoing and logging its output; Ok(exit code) or Err when it couldn't start.
fn run(program: &str, args: &[&str], dir: &Path) -> io::Result<i32> {
    let mut child = Command::new(program).args(args).current_dir(dir)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let err = child.stderr.take().map(|e| std::thread::spawn(move || {
        BufReader::new(e).lines().map_while(Result::ok).collect::<Vec<_>>()
    }));
    if let Some(out) = child.stdout.take() {
        for l in BufReader::new(out).lines().map_while(Result::ok) { say(&format!("    {l}")); }
    }
    if let Some(h) = err { for l in h.join().unwrap_or_default() { say(&format!("    {l}")); } }
    Ok(child.wait()?.code().unwrap_or(-1))
}

/// Run a program quietly and return its stdout (None when it isn't installed or fails).
fn output(program: &str, args: &[&str], dir: &Path) -> Option<String> {
    let o = Command::new(program).args(args).current_dir(dir).stderr(Stdio::null()).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn ask(app: &App, q: &str) -> bool {
    if app.yes { say(&format!("{q} [y/n] y (--yes)")); return true; }
    print!("{q} [y/n] ");
    let _ = io::stdout().flush();
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    let yes = s.trim().eq_ignore_ascii_case("y") || s.trim().eq_ignore_ascii_case("yes");
    if let Ok(mut g) = LOG.lock() { if let Some(f) = g.as_mut() { let _ = writeln!(f, "{q} [y/n] {}", s.trim()); } }
    yes
}

fn read_line(prompt: &str) -> String {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    s.trim().trim_matches('"').to_string()
}

// ------------------------------------------------------------------ the game folder

fn cfg_path(repo: &Path) -> PathBuf { repo.join("manager.cfg") }

fn saved_game(repo: &Path) -> Option<PathBuf> {
    fs::read_to_string(cfg_path(repo)).ok()?.lines().find_map(|l| l.strip_prefix("game=")).map(PathBuf::from)
}

fn save_game(repo: &Path, game: &Path) {
    let _ = fs::write(cfg_path(repo), format!("game={}\n", game.display()));
}

/// Steam's install folders: the registry (Windows), then the usual places.
fn steam_roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if cfg!(windows) {
        for (key, val) in [("HKCU\\Software\\Valve\\Steam", "SteamPath"), ("HKLM\\SOFTWARE\\WOW6432Node\\Valve\\Steam", "InstallPath"),
                           ("HKLM\\SOFTWARE\\Valve\\Steam", "InstallPath")] {
            if let Some(o) = output("reg", &["query", key, "/v", val], Path::new(".")) {
                if let Some(i) = o.find("REG_SZ") { out.push(PathBuf::from(o[i + 6..].trim().replace('/', "\\"))); }
            }
        }
        for d in ["C:\\Program Files (x86)\\Steam", "C:\\Program Files\\Steam"] { out.push(PathBuf::from(d)); }
    } else if let Some(h) = std::env::var_os("HOME") {
        out.push(PathBuf::from(&h).join(".steam/steam"));
        out.push(PathBuf::from(&h).join(".local/share/Steam"));
    }
    out
}

fn find_game(app: &App, arg: Option<PathBuf>) -> Option<PathBuf> {
    let mut cands: Vec<PathBuf> = arg.into_iter().collect();
    cands.extend(saved_game(&app.repo));
    cands.extend(core::game_candidates(&steam_roots()));
    cands.into_iter().find(|p| core::is_game_dir(p))
}

fn choose_game(app: &mut App) {
    say("Teamfight Manager 2's folder holds bundle.game_data.");
    say("In Steam: right-click the game > Manage > Browse local files, then copy the address bar.");
    loop {
        let s = read_line("Paste the game folder (Enter to cancel): ");
        if s.is_empty() { return; }
        let p = PathBuf::from(&s);
        if core::is_game_dir(&p) { save_game(&app.repo, &p); say(&format!("Game folder saved: {}", p.display())); app.game = Some(p); return; }
        say(&format!("No bundle.game_data in \"{s}\": that isn't the game folder. Try again."));
    }
}

fn need_game(app: &mut App) -> Result<PathBuf, Problem> {
    if let Some(g) = app.game.clone() { return Ok(g); }
    if !app.yes { choose_game(app); }
    app.game.clone().ok_or_else(|| problem(
        "The Teamfight Manager 2 folder wasn't found.",
        "It isn't in a Steam library this tool knows, and none was chosen.",
        "Choose 6 (Choose the game folder) and paste the folder that holds bundle.game_data,\nor run with --game \"D:\\SteamLibrary\\steamapps\\common\\Teamfight Manager 2\"."))
}

// ------------------------------------------------------------------ steps

fn game_running() -> bool {
    cfg!(windows) && output("tasklist", &["/fo", "csv", "/nh"], Path::new("."))
        .is_some_and(|o| o.to_ascii_lowercase().contains("teamfightmanager2"))
}

/// git pull; offers to switch to main when this folder is on a branch behind it (rounds 90-91 lived on a branch while
/// main stayed at round 89, so every update installed the old version).
fn pull(app: &App) -> Result<(), Problem> {
    let r = &app.repo;
    if output("git", &["--version"], r).is_none() {
        say("git isn't installed: using the files already in this folder.");
        say("  (To get new versions: download the repository again from GitHub as a zip, or install Git for Windows.)");
        return Ok(());
    }
    if output("git", &["rev-parse", "--is-inside-work-tree"], r).is_none() {
        say("This folder isn't a git clone (a zip download?): using the files already here.");
        return Ok(());
    }
    // The Build step overwrites the repo's DLL; that's a build output, so drop it (it's rebuilt after the pull) instead
    // of letting git refuse the pull with "your local changes would be overwritten".
    let changed = core::changed_build_outputs(&output("git", &["status", "--porcelain"], r).unwrap_or_default());
    if !changed.is_empty() {
        say(&format!("Resetting the locally built {} to the repository's copy (it's rebuilt after the update).", changed.join(", ")));
        let mut args = vec!["checkout", "--"];
        args.extend(changed.iter().copied());
        let _ = run("git", &args, r);
    }
    let branch = output("git", &["rev-parse", "--abbrev-ref", "HEAD"], r).unwrap_or_default();
    say(&format!("Repository branch: {branch}"));
    say("Fetching ...");
    let _ = run("git", &["fetch", "origin"], r);
    let behind_main = output("git", &["rev-list", "--count", "HEAD..origin/main"], r).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    if branch != "main" && behind_main > 0 {
        say(&format!("This folder is on '{branch}', which is {behind_main} commit(s) behind main (the latest release)."));
        if ask(app, "Switch this folder to main?") {
            if run("git", &["checkout", "main"], r).ok() != Some(0) {
                return Err(problem("git couldn't switch to main.", "Files changed in this folder would be overwritten.",
                    "Open a terminal here and run: git stash   then: git checkout main\nor download main again from GitHub."));
            }
        }
    }
    free_running_exe(r);
    say("Pulling ...");
    match run("git", &["pull", "--ff-only"], r) {
        Ok(0) => {}
        Ok(_) => report(&problem("git pull didn't finish.",
            "No internet, or files in this folder were changed, so git won't overwrite them.",
            "Read the git message above. To drop local changes: git stash (keeps them aside) then pull again.\nContinuing with the files already in this folder.")),
        Err(e) => report(&problem("git couldn't run.", e.to_string(), "Reinstall Git for Windows. Continuing with the files here.")),
    }
    if let Some(head) = output("git", &["log", "-1", "--format=%h %s"], r) { say(&format!("Now at: {head}")); }
    Ok(())
}

/// Windows can't overwrite a running exe, so a pull that updates this manager would fail half way. A running exe can
/// be renamed though: move it aside to OLD_EXE and put an identical copy back under its own name for git to replace.
/// The next run deletes the old one.
const OLD_EXE: &str = "TFM2 Mod Manager.old.exe";
fn free_running_exe(repo: &Path) {
    let Ok(me) = std::env::current_exe() else { return };
    if me.parent() != Some(repo) { return; }
    let old = repo.join(OLD_EXE);
    let _ = fs::remove_file(&old);
    if fs::rename(&me, &old).is_ok() && fs::copy(&old, &me).is_err() {
        let _ = fs::rename(&old, &me);   // couldn't copy: put it back as it was
    }
}

/// Build the native DLL with Rust (the GNU toolchain, like native/build.bat) and put it in mods/tfm2_custom_ai.
fn build(app: &App, required: bool) -> Result<bool, Problem> {
    let dir = app.repo.join("native").join("tfm2_custom_ai");
    let rustup = output("rustup", &["--version"], &dir).is_some();
    if !rustup {
        if required {
            return Err(problem("Rust isn't installed, so the DLL can't be built.", "rustup / cargo weren't found.",
                "Install Rust from https://rustup.rs (default options), then build again.\nYou don't need Rust to play: Update uses the DLL that ships in mods\\tfm2_custom_ai."));
        }
        say("Rust isn't installed: using the shipped DLL (mods\\tfm2_custom_ai\\tfm2_custom_ai.dll).");
        return Ok(false);
    }
    let gnu = "stable-x86_64-pc-windows-gnu";
    if !output("rustup", &["toolchain", "list"], &dir).unwrap_or_default().contains(gnu) {
        say("Installing the Rust GNU toolchain (one time) ...");
        let _ = run("rustup", &["toolchain", "install", gnu, "--profile", "minimal"], &dir);
    }
    say("Building the native DLL ...");
    let code = run("rustup", &["run", gnu, "cargo", "build", "--release"], &dir)
        .map_err(|e| problem("cargo couldn't start.", e.to_string(), "Reinstall Rust from https://rustup.rs."))?;
    if code != 0 {
        return Err(problem("The native DLL didn't build.", "The compiler reported an error (above).",
            format!("Send the error lines above (or the log {}) to Claude.\nThe shipped DLL is unchanged, so Update still works.", app.log_path.display())));
    }
    let built = dir.join("target").join("release").join("tfm2_custom_ai.dll");
    let dest = app.repo.join("mods").join("tfm2_custom_ai");
    fs::copy(&built, dest.join("tfm2_custom_ai.dll")).and_then(|_| fs::copy(dir.join("mod.mod_info"), dest.join("mod.mod_info")))
        .map_err(|e| problem("The new DLL couldn't be copied into mods\\tfm2_custom_ai.", e.to_string(),
            "Close the editor and anything using the file, then try again."))?;
    say("Built and copied into mods\\tfm2_custom_ai.");
    // round 108: a stale native mod_info gets the champions' mod disabled by the game ("does not match requirement")
    let read = |p: PathBuf| fs::read_to_string(p).unwrap_or_default();
    let have = core::mod_version(&read(dest.join("mod.mod_info")));
    let need = core::requirement_of(&read(app.repo.join("mods").join("tfm2_custom").join("mod.mod_info")), "tfm2_custom_ai");
    if let (Some(have), Some(need)) = (have, need) {
        if core::older(&have, &need) {
            return Err(problem(format!("The native mod says version {have}, but the champions need {need} or newer."),
                "native\\tfm2_custom_ai\\mod.mod_info wasn't updated with the code, so the game would disable the champions.",
                "Pull again (choose Update). If it still happens, send this message to Claude."));
        }
    }
    Ok(true)
}

fn install(app: &mut App) -> Result<(), Problem> {
    let game = need_game(app)?;
    if game_running() {
        return Err(problem("Teamfight Manager 2 is running.", "The game locks the native DLL and its mod files while it runs.",
            "Close the game completely (check the tray / Task Manager), then choose Update again."));
    }
    let backup = app.repo.join("backups").join(format!("game_mods_{}", core::stamp(now())));
    say(&format!("Installing into {}", game.join("mods").display()));
    say(&format!("  (what was there goes to {})", backup.display()));
    let r = core::install(&app.repo, &game, &backup).map_err(|e| problem(
        "Copying into the game folder failed.", e.to_string(),
        "If the message says 'denied': close the game and the editor, or run this exe as administrator\n(right-click > Run as administrator), since the game is under Program Files.\nNothing is lost: the old files are in the backup folder."))?;
    for m in &r.mods { say(&format!("  updated {m}{}", if m == "tfm2_custom_ai" && r.kept_plans { " (your Map tab plans were kept)" } else { "" })); }
    for (f, id) in &r.removed { say(&format!("  removed {f} (an old copy of {id}; it's in the backup)")); }
    for f in &r.stale_deleted { say(&format!("  deleted old file {f}")); }
    say(&format!("  {} files copied", r.files));
    enable_mods(app, &game);
    Ok(())
}

/// Make sure mods.json enables the custom champions and not the old duplicates (with a backup), like the editor does.
fn enable_mods(app: &App, game: &Path) {
    let path = core::mods_json(game);
    let Ok(text) = fs::read_to_string(&path) else {
        say("  (no config\\game\\mods.json yet: start the game once, enable the mods in its Mod Manager)");
        return;
    };
    let on = core::enabled_mods(&text);
    let missing: Vec<&str> = core::REQUIRED.iter().copied().filter(|m| !on.iter().any(|e| e == m)).collect();
    let old: Vec<&str> = ["tfm2_isliid", "tfm2_gundam", "tfm2_levi"].into_iter().filter(|m| on.iter().any(|e| e == m)).collect();
    if missing.is_empty() && old.is_empty() { say("  mods.json: the custom champions are enabled"); return; }
    say(&format!("  mods.json: not enabled {missing:?}; old copies still enabled {old:?}"));
    if game_running() { say("  (the game is running: change it in the game's Mod Manager instead)"); return; }
    if ask(app, "Fix the enabled mods list in the game's mods.json?") {
        if let Some(new) = core::set_enabled(&text, &missing, &old) {
            let _ = fs::copy(&path, path.with_extension("json.bak"));
            match fs::write(&path, new) {
                Ok(_) => say("  mods.json updated (backup: mods.json.bak). The game asks once to accept the code mod."),
                Err(e) => report(&problem("mods.json couldn't be written.", e.to_string(), "Enable the mods in the game's Mod Manager instead.")),
            }
        }
    }
}

/// Check the install; true when everything matches.
fn check(app: &mut App) -> Result<bool, Problem> {
    let game = need_game(app)?;
    let c = core::check(&app.repo, &game);
    let v = |o: &Option<String>| o.clone().unwrap_or_else(|| "not installed".into());
    say(&format!("Game folder: {}", game.display()));
    say(&format!("  native (tfm2_custom_ai): repo {}  game {}", v(&c.repo_versions.0), v(&c.game_versions.0)));
    say(&format!("  champions (tfm2_custom): repo {}  game {}", v(&c.repo_versions.1), v(&c.game_versions.1)));
    say(&format!("  {} champion files compared, {} differ, {} missing; native DLL {}", c.checked, c.mismatched.len(), c.missing.len(),
        if c.dll_ok { "identical" } else { "DIFFERENT" }));
    let mut issues: Vec<String> = Vec::new();
    for f in c.mismatched.iter().take(12) { issues.push(format!("{f} differs from the repo")); }
    for f in c.missing.iter().take(12) { issues.push(format!("{f} is missing in the game")); }
    if c.mismatched.len() + c.missing.len() > 24 { issues.push("... and more files".into()); }
    if !c.dll_ok { issues.push("the native DLL in the game isn't the repo's (close the game, then Update)".into()); }
    for e in &c.data_errors { issues.push(format!("champion data the game won't load: {e}")); }
    if c.repo_versions != c.game_versions { issues.push("the installed versions aren't the repo's (Update)".into()); }
    for (f, id) in &c.duplicates { issues.push(format!("old copy {f} ({id}) still installed: it can override the new data (Update removes it)")); }
    for m in &c.disabled { issues.push(format!("{m} isn't enabled in config\\game\\mods.json (Update offers to enable it, or use the game's Mod Manager)")); }
    if !c.mods_json_found { say("  (no config\\game\\mods.json: start the game once and enable the mods in its Mod Manager)"); }
    if output("git", &["rev-parse", "--is-inside-work-tree"], &app.repo).is_some() {
        // round 108: compare with what's on GitHub now, including this folder's own branch (not only main)
        let _ = output("git", &["fetch", "--quiet", "origin"], &app.repo);
        if let Some(behind) = output("git", &["rev-list", "--count", "HEAD..origin/main"], &app.repo).and_then(|s| s.parse::<u32>().ok()) {
            if behind > 0 { issues.push(format!("this folder is {behind} commit(s) behind main: Update pulls / switches to main")); }
        }
        let branch = output("git", &["rev-parse", "--abbrev-ref", "HEAD"], &app.repo).unwrap_or_default();
        if branch != "main" {
            if let Some(behind) = output("git", &["rev-list", "--count", "HEAD..@{u}"], &app.repo).and_then(|s| s.parse::<u32>().ok()) {
                if behind > 0 { issues.push(format!("this folder is {behind} commit(s) behind its branch '{branch}': choose Update to pull them")); }
            }
        }
    }
    // the game's log only counts when the game ran after this install (an older log describes the old files)
    let modified = |p: &Path| fs::metadata(p).and_then(|m| m.modified()).ok();
    let installed_at = modified(&game.join("mods").join("tfm2_custom_ai").join("tfm2_custom_ai.dll"));
    if let Some(log) = newest_game_log(&game) {
        let fresh = matches!((modified(&log), installed_at), (Some(l), Some(i)) if l > i);
        let found = core::scan_game_log(&fs::read_to_string(&log).unwrap_or_default());
        if !fresh {
            say("  the game hasn't run since these files were installed: start it, play a match, then check again");
        } else {
            if let (Some(loaded), Some(repo)) = (found.native_loaded.as_ref(), c.repo_versions.0.as_ref()) {
                say(&format!("  the game's last run loaded native {loaded}"));
                if let Some(advice) = core::native_advice(loaded, repo) { issues.push(advice); }
            }
            if !found.load_errors.is_empty() { issues.push(format!("the game's log has {} load error(s): choose 4 (Show logs)", found.load_errors.len())); }
        }
    }
    say("");
    if issues.is_empty() && c.up_to_date() { say("  UP TO DATE: the game has exactly the repo's mods."); return Ok(true); }
    say("  NOT UP TO DATE:");
    for (k, i) in issues.iter().enumerate() { say(&format!("   {}. {i}", k + 1)); }
    Ok(false)
}

fn newest_game_log(game: &Path) -> Option<PathBuf> {
    let mut cands = vec![game.join("log.log"), game.join("logs").join("log.log")];
    if let Some(a) = std::env::var_os("APPDATA") {
        let base = PathBuf::from(a).join("TeamSamoyed").join("TeamfightManager2");
        cands.push(base.join("log.log"));
        cands.push(base.join("data").join("log.log"));
    }
    cands.into_iter().filter(|p| p.is_file())
        .max_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok())
}

fn logs(app: &mut App) -> Result<(), Problem> {
    let game = need_game(app)?;
    match newest_game_log(&game) {
        Some(p) => {
            let t = fs::read_to_string(&p).unwrap_or_default();
            let s = core::scan_game_log(&t);
            say(&format!("== The game's log: {}", p.display()));
            say(&format!("  native mod loaded: {}", s.native_loaded.as_deref().unwrap_or("NOT FOUND (is tfm2_custom_ai enabled?)")));
            for l in &s.load_errors { say(&format!("  LOAD ERROR: {l}")); }
            if !s.load_errors.is_empty() {
                say("  HOW: a champion with a load error is skipped by the game. Update re-installs its files; if it");
                say("       stays, send these lines to Claude.");
            }
            for l in &s.panics { say(&format!("  PANIC: {l}")); }
            if !s.panics.is_empty() { say("  HOW: send these lines to Claude (and gundam_log / isliid_log below)."); }
        }
        None => say("== The game's log.log wasn't found (start a match once)."),
    }
    for name in ["gundam_log.txt", "isliid_log.txt", "levi_log.txt", "scribble_log.txt", "coder_log.txt"] {
        let p = game.join("mods").join("tfm2_custom_ai").join(name);
        if let Ok(t) = fs::read_to_string(&p) {
            say(&format!("== {name} (last 15 lines)"));
            for l in core::tail(&t, 15) { say(&format!("  {l}")); }
        }
    }
    let perf = game.join("mods").join("tfm2_custom_ai").join("perf_log.txt");
    match fs::read_to_string(&perf).ok().as_deref().and_then(core::read_perf) {
        Some((block, reading)) => {
            say("== perf_log.txt (the last 10 s of game measured)");
            for l in block { say(&format!("  {l}")); }
            say(&format!("  READING: {reading}."));
        }
        None if perf_on(&game) => say("== Performance log is ON: it fills while a match plays (restart the game after turning it on)."),
        None => {}
    }
    let dir = app.repo.join("logs");
    if let Some(prev) = fs::read_dir(&dir).into_iter().flatten().flatten().map(|e| e.path())
        .filter(|p| p != &app.log_path && p.extension().is_some_and(|x| x == "txt"))
        .max_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok()) {
        say(&format!("== The previous manager run: {}", prev.display()));
        for l in core::tail(&fs::read_to_string(&prev).unwrap_or_default(), 15) { say(&format!("  {l}")); }
    }
    Ok(())
}

fn perf_flag(game: &Path) -> PathBuf { game.join("mods").join("tfm2_custom_ai").join("perf.flag") }
fn perf_on(game: &Path) -> bool { perf_flag(game).is_file() }

/// Round 99: the native mod measures itself (time per hook, effects played, the game's freezes) into perf_log.txt
/// while perf.flag sits next to it. The game reads the flag when it starts.
fn toggle_perf(app: &mut App) -> Result<(), Problem> {
    let game = need_game(app)?;
    let flag = perf_flag(&game);
    let dir = flag.parent().map(Path::to_path_buf).unwrap_or_default();
    if perf_on(&game) {
        fs::remove_file(&flag).map_err(|e| problem("The performance log couldn't be turned off.", e.to_string(),
            "Close the game and choose 7 again."))?;
        say("Performance log OFF (restart the game). perf_log.txt stays until you delete it.");
    } else {
        if !dir.is_dir() {
            return Err(problem("The native mod isn't installed in the game.", dir.display().to_string(), "Choose 1 first."));
        }
        fs::write(&flag, "on\n").map_err(|e| problem("The performance log couldn't be turned on.", e.to_string(),
            "Check the game folder isn't read-only, then choose 7 again."))?;
        let _ = fs::remove_file(dir.join("perf_log.txt"));
        say("Performance log ON. Restart the game, play one match, then choose 4: it reads the log for you.");
        say("  (or send mods/tfm2_custom_ai/perf_log.txt from the game folder to Claude)");
    }
    Ok(())
}

fn editor(app: &mut App) -> Result<(), Problem> {
    let server = app.repo.join("editor").join("server.js");
    if output("node", &["--version"], &app.repo).is_none() {
        let page = app.repo.join("editor").join("index.html");
        if cfg!(windows) { let _ = Command::new("explorer").arg(&page).spawn(); }
        return Err(problem("Node.js isn't installed, so the editor opened in standalone mode.",
            "The editor's server (save detection, backups, the game's files) needs Node.js.",
            "Install Node.js LTS from https://nodejs.org, then choose 3 again."));
    }
    let mut cmd = Command::new("node");
    cmd.arg(&server).current_dir(app.repo.join("editor"));
    if let Some(g) = &app.game { cmd.arg("--game").arg(g); }
    #[cfg(windows)]
    { use std::os::windows::process::CommandExt; cmd.creation_flags(0x0000_0010); } // its own console window
    cmd.spawn().map_err(|e| problem("The editor couldn't start.", e.to_string(), "Reinstall Node.js from https://nodejs.org."))?;
    say("The editor is starting in its own window and opens http://localhost:7272 in the browser.");
    say("  If the browser says it can't connect: the editor's window shows why (e.g. port 7272 busy: an editor is");
    say("  already open, so use that one).");
    Ok(())
}

fn update(app: &mut App) -> Result<(), Problem> {
    say("== 1/4 Pull the latest version");
    pull(app)?;
    say("== 2/4 Build the native DLL");
    if let Err(p) = build(app, false) { report(&p); say("Continuing with the shipped DLL."); }
    say("== 3/4 Install into the game");
    install(app)?;
    say("== 4/4 Check");
    if check(app)? {
        say("Start the game. The first time after an update it may ask to accept the code mod: accept, then restart it.");
        say("After a match, choose 4: gundam_log / isliid_log should name the native version above.");
    }
    Ok(())
}

fn header(app: &App) {
    let commit = output("git", &["log", "-1", "--format=%h %s"], &app.repo).unwrap_or_else(|| "(not a git clone)".into());
    let branch = output("git", &["rev-parse", "--abbrev-ref", "HEAD"], &app.repo).unwrap_or_default();
    say("");
    say("==============================  TFM2 Mod Manager  ==============================");
    say(&format!(" repo: {} [{branch}] {commit}", app.repo.display()));
    match &app.game {
        Some(g) => {
            let c = core::check(&app.repo, g);
            say(&format!(" game: {}", g.display()));
            say(&format!(" installed: native {}  champions {}",
                c.game_versions.0.as_deref().unwrap_or("-"), c.game_versions.1.as_deref().unwrap_or("-")));
        }
        None => say(" game: NOT FOUND (choose 6)"),
    }
    say(&format!(" log: {}", app.log_path.display()));
    say("================================================================================");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_default();
    let repo = core::find_repo(&exe_dir).or_else(|| std::env::current_dir().ok().and_then(|d| core::find_repo(&d)));
    let Some(repo) = repo else {
        println!("This exe must sit in the tfm2_mods folder (next to the 'mods' and 'editor' folders).");
        println!("HOW: move \"TFM2 Mod Manager.exe\" back into that folder and run it from there.");
        let _ = read_line("Press Enter to close.");
        std::process::exit(2);
    };
    let _ = fs::remove_file(repo.join(OLD_EXE));   // left by the last update (see free_running_exe)
    let _ = fs::create_dir_all(repo.join("logs"));
    let log_path = repo.join("logs").join(format!("manager-{}.txt", core::stamp(now())));
    if let Ok(mut g) = LOG.lock() { *g = fs::File::create(&log_path).ok(); }
    let flag = |f: &str| args.iter().any(|a| a == f);
    let game_arg = args.iter().position(|a| a == "--game").and_then(|i| args.get(i + 1)).map(PathBuf::from);
    let mut app = App { repo, game: None, yes: flag("--yes"), log_path };
    app.game = find_game(&app, game_arg.clone());
    if let (Some(a), Some(g)) = (game_arg, app.game.as_ref()) { if &a == g { save_game(&app.repo, g); } }
    header(&app);
    // in a sensible order whatever order they're given in: build, update, check, logs, editor
    let actions: Vec<&str> = ["--build", "--update", "--check", "--perf", "--logs", "--editor"].into_iter().filter(|f| flag(f)).collect();
    if !actions.is_empty() {
        let mut ok = true;
        for a in actions {
            let r = match a {
                "--build" => build(&app, true).map(|_| ()),
                "--update" => update(&mut app),
                "--check" => check(&mut app).map(|up| { if !up { ok = false; } }),
                "--editor" => editor(&mut app),
                "--perf" => toggle_perf(&mut app),
                _ => logs(&mut app),
            };
            if let Err(p) = r { report(&p); ok = false; }
        }
        say(&format!("Log saved: {}", app.log_path.display()));
        if !app.yes { let _ = read_line("Press Enter to close."); }
        std::process::exit(if ok { 0 } else { 1 });
    }
    loop {
        say("");
        say("  1  Update everything (pull, build if Rust is installed, install into the game, check)");
        say("  2  Check the game is up to date (changes nothing)");
        say("  3  Start the editor");
        say("  4  Show logs (game log errors, gundam / isliid / levi / coder logs, performance log, last run)");
        say("  5  Build the native DLL only");
        say("  6  Choose the game folder");
        say(&format!("  7  Performance log on/off (now {})", if app.game.as_deref().is_some_and(perf_on) { "ON" } else { "off" }));
        say("  0  Exit");
        let c = read_line("Choose: ");
        if let Ok(mut g) = LOG.lock() { if let Some(f) = g.as_mut() { let _ = writeln!(f, "Choose: {c}"); } }
        let r = match c.as_str() {
            "1" => update(&mut app),
            "2" => check(&mut app).map(|_| ()),
            "3" => editor(&mut app),
            "4" => logs(&mut app),
            "5" => build(&app, true).map(|_| ()),
            "6" => { choose_game(&mut app); Ok(()) }
            "7" => toggle_perf(&mut app),
            "0" | "" => break,
            _ => { say("Type a number from the list."); Ok(()) }
        };
        if let Err(p) = r { report(&p); }
        header(&app);
    }
    say(&format!("Log saved: {}", app.log_path.display()));
}
