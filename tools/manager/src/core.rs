//! The manager's file logic, free of processes and prompts so it can be tested anywhere: finding the repo and the
//! game, installing the mods, checking the install, mods.json, and scanning the game log.
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The custom champions' ids: a folder other than `tfm2_custom` holding one of these is an old duplicate.
pub const CUSTOM_IDS: [&str; 4] = ["tfm2_isliid_emperor", "tfm2_gundam_aegis_zero", "tfm2_levi_levi", "tfm2_custom_minato"];
/// Files older versions left behind that a newer one replaced (relative to the game's mods folder); an install deletes
/// them. Round 100: Isliid's combined sheets that are now split
/// per rank / per sword (the game would still load the big ones if they stayed).
pub const STALE_FILES: [&str; 12] = [
    "tfm2_custom/vfx/engraving_colors#sheet.png",
    "tfm2_custom/vfx/engraving_colors#anim.fanim",
    "tfm2_custom/vfx/aura_fields8#sheet.png",
    "tfm2_custom/vfx/aura_fields8#anim.fanim",
    "tfm2_custom/vfx/auras8#sheet.png",
    "tfm2_custom/vfx/auras8#anim.fanim",
    "tfm2_custom/vfx/falls#sheet.png",
    "tfm2_custom/vfx/falls#anim.fanim",
    "tfm2_custom/vfx/orbit8#sheet.png",
    "tfm2_custom/vfx/orbit8#anim.fanim",
    "tfm2_custom/vfx/swords_comet#sheet.png",
    "tfm2_custom/vfx/swords_comet#anim.fanim",
];
/// The mods that must be installed and enabled for the custom champions.
pub const REQUIRED: [&str; 2] = ["tfm2_custom", "tfm2_custom_ai"];
/// The Steam app id of Teamfight Manager 2.
pub const STEAM_APP_ID: &str = "3009300";

/// The repo: the first folder at or above `start` that holds both `mods` and `editor`.
pub fn find_repo(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(d) = dir {
        if d.join("mods").is_dir() && d.join("editor").is_dir() { return Some(d.to_path_buf()); }
        dir = d.parent();
    }
    None
}

/// A game folder holds bundle.game_data.
pub fn is_game_dir(p: &Path) -> bool { p.join("bundle.game_data").is_file() }

/// The library paths listed in Steam's libraryfolders.vdf.
pub fn vdf_paths(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("\"path\"") {
            if let Some(v) = rest.trim().strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
                out.push(v.replace("\\\\", "\\"));
            }
        }
    }
    out
}

/// The install folder name in appmanifest_3009300.acf.
pub fn acf_installdir(text: &str) -> Option<String> {
    text.lines().map(str::trim).find_map(|t| t.strip_prefix("\"installdir\""))
        .and_then(|r| r.trim().strip_prefix('"').and_then(|r| r.strip_suffix('"')).map(str::to_string))
}

/// The game folders to try under the given Steam roots (each root and its libraries).
pub fn game_candidates(steam_roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut libs: Vec<PathBuf> = Vec::new();
    for r in steam_roots {
        if !libs.contains(r) { libs.push(r.clone()); }
        if let Ok(t) = fs::read_to_string(r.join("steamapps").join("libraryfolders.vdf")) {
            for p in vdf_paths(&t) { let p = PathBuf::from(p); if !libs.contains(&p) { libs.push(p); } }
        }
    }
    let mut out = Vec::new();
    for lib in libs {
        let apps = lib.join("steamapps");
        if let Some(dir) = fs::read_to_string(apps.join(format!("appmanifest_{STEAM_APP_ID}.acf"))).ok().as_deref().and_then(acf_installdir) {
            out.push(apps.join("common").join(dir));
        }
        for name in ["Teamfight Manager2", "Teamfight Manager 2"] { out.push(apps.join("common").join(name)); }
    }
    out
}

/// The top-level "version" of a mod.mod_info (the first one in the file; dependencies come after it).
pub fn mod_version(text: &str) -> Option<String> {
    let i = text.find("\"version\"")?;
    let rest = &text[i + 9..];
    let start = rest.find('"')? + 1;
    let end = rest[start..].find('"')? + start;
    Some(rest[start..end].to_string())
}

/// The repo's mod folders that are real mods (they hold a mod.mod_info), sorted.
pub fn mod_folders(repo: &Path) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(repo.join("mods")).into_iter().flatten().flatten()
        .filter(|e| e.path().join("mod.mod_info").is_file())
        .filter_map(|e| e.file_name().into_string().ok()).collect();
    out.sort();
    out
}

/// Copy a folder recursively (files overwrite).
pub fn copy_dir(from: &Path, to: &Path) -> io::Result<usize> {
    fs::create_dir_all(to)?;
    let mut n = 0;
    for e in fs::read_dir(from)? {
        let e = e?;
        let target = to.join(e.file_name());
        if e.file_type()?.is_dir() { n += copy_dir(&e.path(), &target)?; } else { fs::copy(e.path(), &target)?; n += 1; }
    }
    Ok(n)
}

/// Every file under `dir`, relative to it, sorted.
pub fn files_under(dir: &Path) -> Vec<PathBuf> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
        for e in fs::read_dir(dir).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() { walk(base, &p, out); } else if let Ok(r) = p.strip_prefix(base) { out.push(r.to_path_buf()); }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// Installed folders (other than tfm2_custom) that hold an old copy of a custom champion, with the champion found.
pub fn stale_duplicates(game_mods: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for e in fs::read_dir(game_mods).into_iter().flatten().flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name == "tfm2_custom" || !e.path().is_dir() { continue; }
        if let Some(id) = CUSTOM_IDS.iter().find(|id| e.path().join("champion").join(format!("{id}.data_champion")).is_file()) {
            out.push((name, id.to_string()));
        }
    }
    out.sort();
    out
}

/// What an install did.
#[derive(Debug, Default)]
pub struct InstallReport {
    pub mods: Vec<String>,
    pub files: usize,
    pub kept_plans: bool,
    pub removed: Vec<(String, String)>,
    pub stale_deleted: Vec<String>,
}

/// Install every repo mod into `<game>/mods`, backing up what's there into `backup` first.
/// tfm2_custom_ai: the DLL and mod info always; the Map tab's plans (tactics.json + tactics.txt, as a pair) and the
/// map dump only when the game has none. Old duplicate champion folders move into the backup.
pub fn install(repo: &Path, game: &Path, backup: &Path) -> io::Result<InstallReport> {
    let mut r = InstallReport::default();
    let gm = game.join("mods");
    fs::create_dir_all(&gm)?;
    for name in mod_folders(repo) {
        let src = repo.join("mods").join(&name);
        let dst = gm.join(&name);
        if dst.is_dir() { copy_dir(&dst, &backup.join(&name))?; }
        if name == "tfm2_custom_ai" {
            fs::create_dir_all(&dst)?;
            let had_plans = dst.join("tactics.txt").exists() || dst.join("tactics.json").exists();
            r.kept_plans = had_plans;
            for e in fs::read_dir(&src)?.flatten() {
                let f = e.file_name().to_string_lossy().to_string();
                if e.path().is_dir() { continue; }
                let always = f == "tfm2_custom_ai.dll" || f == "mod.mod_info";
                let plan = f == "tactics.json" || f == "tactics.txt";
                let target = dst.join(&f);
                if always || (!target.exists() && !(plan && had_plans)) { fs::copy(e.path(), &target)?; r.files += 1; }
            }
        } else {
            r.files += copy_dir(&src, &dst)?;
        }
        r.mods.push(name);
    }
    for (name, id) in stale_duplicates(&gm) {
        copy_dir(&gm.join(&name), &backup.join(&name))?;
        fs::remove_dir_all(gm.join(&name))?;
        r.removed.push((name, id));
    }
    for f in STALE_FILES {
        let p = gm.join(f);
        if p.is_file() { fs::remove_file(&p)?; r.stale_deleted.push(f.to_string()); }
    }
    Ok(r)
}

/// What a check found.
#[derive(Debug, Default)]
pub struct CheckReport {
    pub checked: usize,
    pub mismatched: Vec<String>,
    pub missing: Vec<String>,
    pub dll_ok: bool,
    pub repo_versions: (Option<String>, Option<String>),
    pub game_versions: (Option<String>, Option<String>),
    pub duplicates: Vec<(String, String)>,
    pub disabled: Vec<String>,
    pub mods_json_found: bool,
}

impl CheckReport {
    pub fn up_to_date(&self) -> bool {
        self.mismatched.is_empty() && self.missing.is_empty() && self.dll_ok && self.duplicates.is_empty()
            && self.disabled.is_empty() && self.repo_versions == self.game_versions
    }
}

/// Compare every file of the repo's tfm2_custom and the native DLL with the installed copies, byte for byte; read
/// both versions; list old duplicates and the required mods mods.json doesn't enable.
pub fn check(repo: &Path, game: &Path) -> CheckReport {
    let mut c = CheckReport::default();
    let (rm, gm) = (repo.join("mods"), game.join("mods"));
    for rel in files_under(&rm.join("tfm2_custom")) {
        c.checked += 1;
        let shown = format!("tfm2_custom/{}", rel.to_string_lossy().replace('\\', "/"));
        match (fs::read(rm.join("tfm2_custom").join(&rel)), fs::read(gm.join("tfm2_custom").join(&rel))) {
            (Ok(a), Ok(b)) => if a != b { c.mismatched.push(shown) },
            (Ok(_), Err(_)) => c.missing.push(shown),
            _ => {}
        }
    }
    let dll = |m: &Path| fs::read(m.join("tfm2_custom_ai").join("tfm2_custom_ai.dll")).ok();
    c.dll_ok = matches!((dll(&rm), dll(&gm)), (Some(a), Some(b)) if a == b);
    let ver = |m: &Path, name: &str| fs::read_to_string(m.join(name).join("mod.mod_info")).ok().as_deref().and_then(mod_version);
    c.repo_versions = (ver(&rm, "tfm2_custom_ai"), ver(&rm, "tfm2_custom"));
    c.game_versions = (ver(&gm, "tfm2_custom_ai"), ver(&gm, "tfm2_custom"));
    c.duplicates = stale_duplicates(&gm);
    if let Ok(t) = fs::read_to_string(mods_json(game)) {
        c.mods_json_found = true;
        let on = enabled_mods(&t);
        c.disabled = REQUIRED.iter().filter(|m| !on.iter().any(|e| e == *m)).map(|m| m.to_string()).collect();
    }
    c
}

/// The game's mod switch file.
pub fn mods_json(game: &Path) -> PathBuf { game.join("config").join("game").join("mods.json") }

/// The bounds of the enabled_mods array in mods.json (the `[` and the matching `]`).
fn enabled_span(text: &str) -> Option<(usize, usize)> {
    let k = text.find("\"enabled_mods\"")?;
    let open = k + text[k..].find('[')?;
    let close = open + text[open..].find(']')?;
    Some((open, close))
}

/// The mod ids in mods.json's enabled_mods.
pub fn enabled_mods(text: &str) -> Vec<String> {
    let Some((open, close)) = enabled_span(text) else { return Vec::new() };
    text[open + 1..close].split(',').map(|s| s.trim().trim_matches('"').to_string()).filter(|s| !s.is_empty()).collect()
}

/// mods.json with `add` enabled and `remove` dropped from enabled_mods (the rest of the file untouched).
pub fn set_enabled(text: &str, add: &[&str], remove: &[&str]) -> Option<String> {
    let (open, close) = enabled_span(text)?;
    let mut list = enabled_mods(text);
    list.retain(|m| !remove.contains(&m.as_str()));
    for a in add { if !list.iter().any(|m| m == a) { list.push(a.to_string()); } }
    let body = list.iter().map(|m| format!("\"{m}\"")).collect::<Vec<_>>().join(", ");
    Some(format!("{}[{}]{}", &text[..open], body, &text[close + 1..]))
}

/// What the game log says about the mods.
#[derive(Debug, Default, PartialEq)]
pub struct LogScan {
    pub native_loaded: Option<String>,
    pub load_errors: Vec<String>,
    pub panics: Vec<String>,
}

/// The lines of the game's log.log that matter: the native mod's "loaded" line (its version), data load errors and
/// panics (the last 10 of each).
pub fn scan_game_log(text: &str) -> LogScan {
    let mut s = LogScan::default();
    for line in text.lines() {
        let l = line.trim();
        if let Some(i) = l.find("tfm2_custom_ai ") {
            if let Some(j) = l[i..].find(" loaded") {
                s.native_loaded = Some(l[i + 15..i + j].trim().to_string());
            }
        }
        if l.contains("load error") { s.load_errors.push(l.to_string()); }
        if l.contains("panicked") { s.panics.push(l.to_string()); }
    }
    let tail = |v: &mut Vec<String>| { let n = v.len().saturating_sub(10); v.drain(..n); };
    tail(&mut s.load_errors);
    tail(&mut s.panics);
    s
}

/// The last `n` lines of a text.
pub fn tail(text: &str, n: usize) -> Vec<&str> {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(n)..].to_vec()
}

/// Round 99: the last 10-second block of perf_log.txt, and a one-line reading of it. The native mod writes a block
/// every 600 ticks: "game .. tick N: X ms in the mod ..", one line per hook, "effects played: E (R a second), at most P
/// on one tick", the top casters, and "longest gap between two match ticks: G ms (S over 100 ms)".
pub fn read_perf(text: &str) -> Option<(Vec<&str>, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().rposition(|l| l.starts_with("game ") && l.contains(" ms in the mod"))?;
    let block = lines[start..].to_vec();
    let num_after = |line: &str, key: &str| -> Option<f64> {
        let rest = &line[line.find(key)? + key.len()..];
        let n: String = rest.trim_start().chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        n.parse().ok()
    };
    let mod_ms = num_after(block[0], ": ").unwrap_or(0.0);
    let fx = block.iter().find_map(|l| num_after(l, "effects played: ").map(|n| (n, num_after(l, "at most ").unwrap_or(0.0))));
    let gap = block.iter().find_map(|l| num_after(l, "between two match ticks: ").map(|g| (g, num_after(l, "ms (").unwrap_or(0.0))));
    let mut reading = format!("the mod's own code took {:.0} ms of these 10 s ({:.1}%)", mod_ms, mod_ms / 100.0);
    if let Some((n, peak)) = fx { reading += &format!("; {:.0} effects ({:.0} a second, at most {peak:.0} on one tick)", n, n / 10.0); }
    match gap {
        Some((g, over)) if over > 0.0 && mod_ms < 1000.0 => reading += &format!(
            "; the game froze {over:.0} times (longest {g:.0} ms) while the mod's code stayed fast, so the time went \
             into the game itself (drawing and effects)"),
        Some((g, over)) if over > 0.0 => reading += &format!(
            "; the game froze {over:.0} times (longest {g:.0} ms) and the mod's code is a big part of it"),
        Some((g, _)) => reading += &format!("; no freezes (longest gap {g:.0} ms)"),
        None => {}
    }
    Some((block, reading))
}

/// Days since 1970-01-01 to (year, month, day) (Howard Hinnant's civil_from_days).
pub fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// A timestamp like 20261006_181500 (UTC) for backup and log names.
pub fn stamp(unix_secs: u64) -> String {
    let (y, m, d) = civil((unix_secs / 86_400) as i64);
    let s = unix_secs % 86_400;
    format!("{y:04}{m:02}{d:02}_{:02}{:02}{:02}", s / 3600, s / 60 % 60, s % 60)
}

/// Files the Build step overwrites in the repo (the freshly built DLL and its mod_info). They're build outputs, so a
/// local change to them is dropped before pulling; otherwise git refuses the pull ("local changes would be overwritten").
pub const BUILD_OUTPUTS: [&str; 2] = ["mods/tfm2_custom_ai/tfm2_custom_ai.dll", "mods/tfm2_custom_ai/mod.mod_info"];

/// The build outputs that `git status --porcelain` lists as changed.
pub fn changed_build_outputs(porcelain: &str) -> Vec<&'static str> {
    BUILD_OUTPUTS.iter().copied().filter(|f| porcelain.lines()
        .any(|l| l.split_whitespace().last().map(|n| n.trim_matches('"')) == Some(*f))).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn build_outputs_reset_before_pull() {
        let st = " M mods/tfm2_custom_ai/tfm2_custom_ai.dll\n M editor/x.js\n?? logs/a.txt\n";
        assert_eq!(changed_build_outputs(st), vec!["mods/tfm2_custom_ai/tfm2_custom_ai.dll"]);
        assert_eq!(changed_build_outputs("MM mods/tfm2_custom_ai/mod.mod_info\n"), vec!["mods/tfm2_custom_ai/mod.mod_info"]);
        assert!(changed_build_outputs("").is_empty());
        // the manager trims git's output, so the first line can lose its leading space
        assert_eq!(changed_build_outputs("M mods/tfm2_custom_ai/tfm2_custom_ai.dll"), vec!["mods/tfm2_custom_ai/tfm2_custom_ai.dll"]);
    }

    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("tfm2_manager_test_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }
    fn write(p: &Path, text: &str) { fs::create_dir_all(p.parent().unwrap()).unwrap(); fs::write(p, text).unwrap(); }

    fn repo_and_game(name: &str) -> (PathBuf, PathBuf, PathBuf) {
        let root = tmp(name);
        let repo = root.join("My Repo (2)");
        let game = root.join("Program Files (x86)").join("Teamfight Manager2");
        write(&repo.join("editor/index.html"), "x");
        write(&repo.join("mods/tfm2_custom/mod.mod_info"), "{\n  \"version\": \"0.2.4\",\n  \"dependencies\": [{\"version\": \">=1\"}]\n}");
        write(&repo.join("mods/tfm2_custom/champion/tfm2_gundam_aegis_zero.data_champion"), "new aegis");
        write(&repo.join("mods/tfm2_custom/vfx/gundam#sheet.png"), "new sheet");
        write(&repo.join("mods/tfm2_custom_ai/mod.mod_info"), "{ \"version\": \"0.10.4\" }");
        write(&repo.join("mods/tfm2_custom_ai/tfm2_custom_ai.dll"), "new dll");
        write(&repo.join("mods/tfm2_custom_ai/tactics.json"), "repo plans");
        write(&repo.join("mods/tfm2_custom_ai/tactics.txt"), "repo plans");
        write(&repo.join("mods/tfm2_custom_ai/map_dump.json"), "dump");
        write(&repo.join("mods/tfm2_gundam/source/build.js"), "source only, no mod_info");
        write(&game.join("bundle.game_data"), "");
        (root, repo, game)
    }

    #[test]
    fn repo_root_found_from_exe_dir() {
        let (_r, repo, _g) = repo_and_game("root");
        let deep = repo.join("tools/manager/target/release");
        fs::create_dir_all(&deep).unwrap();
        assert_eq!(find_repo(&deep), Some(repo.clone()));
        assert_eq!(mod_folders(&repo), vec!["tfm2_custom".to_string(), "tfm2_custom_ai".to_string()]);
    }

    #[test]
    fn install_copies_and_keeps_plans() {
        let (root, repo, game) = repo_and_game("install");
        write(&game.join("mods/tfm2_custom_ai/tactics.txt"), "MY PLANS");
        write(&game.join("mods/tfm2_custom_ai/tfm2_custom_ai.dll"), "old dll");
        write(&game.join("mods/tfm2_custom/champion/my_own.data_champion"), "mine");
        write(&game.join("mods/tfm2_custom/vfx/engraving_colors#sheet.png"), "stale");
        let r = install(&repo, &game, &root.join("backup")).unwrap();
        assert!(r.kept_plans);
        assert_eq!(fs::read_to_string(game.join("mods/tfm2_custom_ai/tactics.txt")).unwrap(), "MY PLANS");
        assert!(!game.join("mods/tfm2_custom_ai/tactics.json").exists(), "plans only come as a pair");
        assert!(game.join("mods/tfm2_custom_ai/map_dump.json").exists());
        assert_eq!(fs::read_to_string(game.join("mods/tfm2_custom_ai/tfm2_custom_ai.dll")).unwrap(), "new dll");
        assert!(game.join("mods/tfm2_custom/champion/my_own.data_champion").exists(), "his own files stay");
        assert!(!game.join("mods/tfm2_custom/vfx/engraving_colors#sheet.png").exists());
        assert_eq!(fs::read_to_string(root.join("backup/tfm2_custom_ai/tfm2_custom_ai.dll")).unwrap(), "old dll");
        assert!(!game.join("mods/tfm2_gundam").exists(), "source-only folders aren't mods");
        // a fresh game gets the plans pair
        let fresh = root.join("fresh");
        write(&fresh.join("bundle.game_data"), "");
        let r = install(&repo, &fresh, &root.join("backup2")).unwrap();
        assert!(!r.kept_plans && fresh.join("mods/tfm2_custom_ai/tactics.json").exists() && fresh.join("mods/tfm2_custom_ai/tactics.txt").exists());
    }

    #[test]
    fn stale_duplicates_go_to_backup() {
        let (root, repo, game) = repo_and_game("dups");
        write(&game.join("mods/tfm2_gundam/champion/tfm2_gundam_aegis_zero.data_champion"), "old aegis");
        write(&game.join("mods/tfm2_gundam/mod.mod_info"), "{}");
        write(&game.join("mods/tfm2_jojo/champion/tfm2_jojo_dio.data_champion"), "dio");
        assert_eq!(stale_duplicates(&game.join("mods")), vec![("tfm2_gundam".to_string(), "tfm2_gundam_aegis_zero".to_string())]);
        let r = install(&repo, &game, &root.join("backup")).unwrap();
        assert_eq!(r.removed.len(), 1);
        assert!(!game.join("mods/tfm2_gundam").exists() && root.join("backup/tfm2_gundam/mod.mod_info").exists());
        assert!(game.join("mods/tfm2_jojo").exists(), "other mods are left alone");
    }

    #[test]
    fn check_reports_mismatch() {
        let (root, repo, game) = repo_and_game("check");
        write(&mods_json(&game), "{\n  \"enabled_mods\": [\"tfm2_custom\", \"tfm2_custom_ai\"],\n  \"x\": 1\n}");
        install(&repo, &game, &root.join("backup")).unwrap();
        let c = check(&repo, &game);
        assert!(c.up_to_date(), "{c:?}");
        assert_eq!(c.checked, 3);
        assert_eq!(c.game_versions, (Some("0.10.4".into()), Some("0.2.4".into())));
        write(&game.join("mods/tfm2_custom/vfx/gundam#sheet.png"), "OLD sheet");
        fs::remove_file(game.join("mods/tfm2_custom/champion/tfm2_gundam_aegis_zero.data_champion")).unwrap();
        write(&game.join("mods/tfm2_custom_ai/tfm2_custom_ai.dll"), "old dll");
        let c = check(&repo, &game);
        assert!(!c.up_to_date());
        assert_eq!(c.mismatched, vec!["tfm2_custom/vfx/gundam#sheet.png".to_string()]);
        assert_eq!(c.missing, vec!["tfm2_custom/champion/tfm2_gundam_aegis_zero.data_champion".to_string()]);
        assert!(!c.dll_ok);
    }

    #[test]
    fn vdf_and_manifest_parse() {
        let vdf = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"C:\\\\Program Files (x86)\\\\Steam\"\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n\t}\n}";
        assert_eq!(vdf_paths(vdf), vec!["C:\\Program Files (x86)\\Steam".to_string(), "D:\\SteamLibrary".to_string()]);
        assert_eq!(acf_installdir("\"AppState\"\n{\n\t\"appid\"\t\t\"3009300\"\n\t\"installdir\"\t\t\"Teamfight Manager 2\"\n}"),
            Some("Teamfight Manager 2".to_string()));
        let root = tmp("steam");
        write(&root.join("steamapps/libraryfolders.vdf"), &format!("\"path\" \"{}\"", root.join("lib2").display()));
        write(&root.join("lib2/steamapps/appmanifest_3009300.acf"), "\"installdir\" \"TFM2 Here\"");
        let c = game_candidates(&[root.clone()]);
        assert!(c.contains(&root.join("lib2/steamapps/common/TFM2 Here")));
        assert!(c.contains(&root.join("steamapps/common/Teamfight Manager2")));
    }

    #[test]
    fn mods_json_enable() {
        let t = "{\n  \"enabled_mods\": [\n    \"base\",\n    \"tfm2_levi\"\n  ],\n  \"other\": [1, 2]\n}";
        assert_eq!(enabled_mods(t), vec!["base".to_string(), "tfm2_levi".to_string()]);
        let out = set_enabled(t, &["tfm2_custom", "tfm2_custom_ai"], &["tfm2_levi"]).unwrap();
        assert_eq!(enabled_mods(&out), vec!["base".to_string(), "tfm2_custom".to_string(), "tfm2_custom_ai".to_string()]);
        assert!(out.ends_with("\"other\": [1, 2]\n}"), "the rest is untouched");
        assert_eq!(set_enabled("{}", &["x"], &[]), None);
    }

    #[test]
    fn log_scan_finds_load_errors() {
        let log = "[INFO] tfm2_custom_ai 0.10.4 loaded (game 1.2.3): Unlimited Void, ...\n\
                   [WARN] data_champion load error: invalid value: integer -60000 in tfm2_custom_minato\n\
                   thread 'sim' panicked at line_defense.rs:968\n";
        let s = scan_game_log(log);
        assert_eq!(s.native_loaded.as_deref(), Some("0.10.4"));
        assert_eq!(s.load_errors.len(), 1);
        assert_eq!(s.panics.len(), 1);
        assert_eq!(mod_version("{\n  \"name\": \"x\",\n  \"version\": \"0.2.4\",\n  \"dependencies\": [{\"version\": \">=0.10.4\"}]}"), Some("0.2.4".into()));
    }

    #[test]
    fn perf_log_is_read() {
        let log = "game 1 tick 600: 3.00 ms in the mod over the last 600 ticks (both simulations)\n  old\n\
                   game 1 tick 1200: 42.40 ms in the mod over the last 600 ticks (both simulations)\n\
                   \x20 isliid passive                   30.00 ms     600 calls     50.0 us/call\n\
                   \x20 effects played: 2400 (240 a second), at most 31 on one tick\n\
                   \x20   Isliid                               2100\n\
                   \x20 longest gap between two match ticks: 312 ms (9 over 100 ms)\n";
        let (block, reading) = read_perf(log).unwrap();
        assert!(block[0].contains("tick 1200") && block.len() == 5, "{block:?}");
        assert!(reading.contains("took 42 ms") && reading.contains("240 a second") && reading.contains("at most 31"), "{reading}");
        assert!(reading.contains("froze 9 times (longest 312 ms)") && reading.contains("game itself"), "{reading}");
        let calm = "game 1 tick 600: 2.00 ms in the mod\n  longest gap between two match ticks: 40 ms (0 over 100 ms)\n";
        assert!(read_perf(calm).unwrap().1.contains("no freezes"));
        assert!(read_perf("nothing yet").is_none());
    }

    #[test]
    fn stamps_are_dates() {
        assert_eq!(stamp(0), "19700101_000000");
        assert_eq!(stamp(1_791_312_900), "20261006_185500");
    }
}
