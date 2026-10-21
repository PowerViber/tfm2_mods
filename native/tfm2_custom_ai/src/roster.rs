//! The shared champion model: every champion as "champion – family – skills – basic attack".
//!
//! The data lives in `rules/champions.json` (generated into `roster_data.rs`). Each skill says what it is
//! (its [`Kind`]s: Dash, Projectile, Zone…) and what an athlete uses it for (its [`Intent`]s: Clear,
//! Engage, Escape…), so code can ask "can this champion escape?" or "which skills clear a wave?"
//! without knowing the kit. Casting stays in each kit: native kits own their actions and the engine owns
//! base-game casts, so [`Kit`] answers *what* and *whether ready*, never *how*.

use crate::roster_data::{CHAMPIONS, FAMILIES, FAMILY_RULES};
use mod_api_stable::{SkillSlotV1 as Slot, StableSim};

macro_rules! labels {
    ($name:ident: $($v:ident),+) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name { $($v),+ }
        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$v),+];
            pub fn bit(self) -> u16 { 1 << self as u16 }
            pub fn name(self) -> &'static str {
                match self { $($name::$v => stringify!($v)),+ }
            }
        }
    };
}
labels!(Kind: Dash, Blink, Projectile, Line, Aoe, Zone, Summon, Heal, Shield, Cc, Buff, Construct);
labels!(Intent: Clear, Engage, Escape, Protect, Objective, Poke, Vision);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Range { Melee, Ranged }

#[derive(Debug)]
pub struct BasicAttack { pub name: &'static str, pub range: i64, pub reach: Range }

#[derive(Debug)]
pub struct SkillDef { pub slot: Slot, pub name: &'static str, pub kinds: u16, pub intents: u16 }

impl SkillDef {
    pub fn is(&self, kind: Kind) -> bool { self.kinds & kind.bit() != 0 }
    pub fn serves(&self, intent: Intent) -> bool { self.intents & intent.bit() != 0 }
    pub fn kinds(&self) -> impl Iterator<Item = Kind> + '_ { Kind::ALL.iter().copied().filter(|k| self.is(*k)) }
    pub fn intents(&self) -> impl Iterator<Item = Intent> + '_ { Intent::ALL.iter().copied().filter(|i| self.serves(*i)) }
}

#[derive(Debug)]
pub struct FamilyDef { pub key: &'static str, pub name: &'static str, pub about: &'static str }

/// A rule that already applies across franchises (e.g. Sandevistan outruns Infinity). `to` is a family key
/// or `*` for everyone.
#[derive(Debug)]
pub struct FamilyRule { pub id: &'static str, pub from: &'static str, pub to: &'static str, pub rule: &'static str }

#[derive(Debug)]
pub struct ChampionDef {
    pub id: &'static str,
    pub name: &'static str,
    pub family: usize,
    pub role: &'static str,
    pub native: bool,
    pub basic: BasicAttack,
    pub skills: &'static [SkillDef],
}

impl ChampionDef {
    pub fn family(&self) -> &'static FamilyDef { &FAMILIES[self.family] }
    pub fn can(&self, intent: Intent) -> bool { self.skills.iter().any(|s| s.serves(intent)) }
    pub fn has(&self, kind: Kind) -> bool { self.skills.iter().any(|s| s.is(kind)) }
    pub fn skills_for(&self, intent: Intent) -> impl Iterator<Item = &'static SkillDef> {
        self.skills.iter().filter(move |s| s.serves(intent))
    }
    /// The last part of the id ("tfm2_valorant_omen" → "omen"), the key tactics presets use.
    pub fn short(&self) -> &'static str { self.id.rsplit('_').next().unwrap_or(self.id) }
    /// Rules this champion's family gives or receives.
    pub fn family_rules(&self) -> impl Iterator<Item = &'static FamilyRule> {
        let key = self.family().key;
        FAMILY_RULES.iter().filter(move |r| r.from == key || r.to == key || r.to == "*")
    }
    /// "Gojo – Jujutsu Kaisen – Blue [Projectile, Cc | Clear, Engage] – … – BA: Strike (melee)".
    pub fn line(&self) -> String {
        let mut out = format!("{} – {}", self.name, self.family().name);
        for s in self.skills {
            let kinds: Vec<_> = s.kinds().map(Kind::name).collect();
            let intents: Vec<_> = s.intents().map(Intent::name).collect();
            out.push_str(&format!(" – {} [{} | {}]", s.name, kinds.join(", "), intents.join(", ")));
        }
        let reach = if self.basic.reach == Range::Melee { "melee" } else { "ranged" };
        out.push_str(&format!(" – BA: {} ({reach})", self.basic.name));
        out
    }
}

/// The definition for a champion name as the sim reports it. Exact ids win; otherwise the longest id the
/// name starts with (variants such as "…_v2"), so a reworked kit keeps its champion's definition.
pub fn def(name: &str) -> Option<&'static ChampionDef> {
    CHAMPIONS.iter().find(|c| c.id == name).or_else(|| {
        CHAMPIONS
            .iter()
            .filter(|c| name.starts_with(c.id) && name.as_bytes().get(c.id.len()) == Some(&b'_'))
            .max_by_key(|c| c.id.len())
    })
}

pub fn can(name: &str, intent: Intent) -> bool { def(name).is_some_and(|c| c.can(intent)) }

pub fn skills_for(name: &str, intent: Intent) -> impl Iterator<Item = &'static SkillDef> {
    def(name).into_iter().flat_map(move |c| c.skills_for(intent))
}

pub fn champions_with(kind: Kind) -> impl Iterator<Item = &'static ChampionDef> {
    CHAMPIONS.iter().filter(move |c| c.has(kind))
}

pub fn all() -> &'static [ChampionDef] { &CHAMPIONS }

/// Champions that change the map for everyone: builders, and zones that block sight.
pub fn map_changers() -> impl Iterator<Item = &'static ChampionDef> {
    CHAMPIONS.iter().filter(|c| {
        c.has(Kind::Construct) || c.skills.iter().any(|s| s.is(Kind::Zone) && s.serves(Intent::Vision))
    })
}

/// A champion's kit as seen from outside: what it has and whether a skill for an intent is ready now.
pub trait Kit {
    fn def(&self) -> &'static ChampionDef;

    /// The first skill serving `intent` whose slot has a charge (or no cooldown left) right now.
    fn ready(&self, sim: &StableSim<'_>, me: usize, intent: Intent) -> Option<&'static SkillDef> {
        let entity = sim.get_entity(me)?;
        self.def().skills_for(intent).find(|s| {
            entity.cooldown(s.slot).map_or(true, |cd| cd.charges > 0 || cd.remaining == 0)
        })
    }
}

/// The kit of any champion, read from its definition.
pub struct Generic(pub &'static ChampionDef);

impl Kit for Generic {
    fn def(&self) -> &'static ChampionDef { self.0 }
}

pub fn kit(name: &str) -> Option<Generic> { def(name).map(Generic) }

#[cfg(test)]
mod tests {
    use super::*;

    fn data_champions() -> Vec<String> {
        let mods = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mods");
        let mut out = Vec::new();
        for m in std::fs::read_dir(mods).unwrap().flatten() {
            let Ok(dir) = std::fs::read_dir(m.path().join("champion")) else { continue };
            for f in dir.flatten() {
                let name = f.file_name().to_string_lossy().into_owned();
                if let Some(id) = name.strip_suffix(".data_champion") {
                    out.push(id.to_owned());
                }
            }
        }
        out.sort();
        out
    }

    #[test]
    fn every_shipped_and_base_champion_has_a_definition() {
        let ids = data_champions();
        assert!(ids.len() >= 17, "{ids:?}");
        for id in &ids {
            let d = def(id).unwrap_or_else(|| panic!("{id} has no roster definition"));
            assert!(d.native, "{id}");
        }
        for name in ["android", "archer", "fighter", "wind_mage"] {
            assert!(def(name).is_some_and(|d| !d.native), "{name}");
        }
        assert_eq!(CHAMPIONS.len(), 77);
    }

    #[test]
    fn every_skill_has_a_kind_and_an_intent() {
        for c in all() {
            assert!(!c.skills.is_empty(), "{}", c.id);
            for s in c.skills {
                assert!(s.kinds != 0 && s.intents != 0, "{} {}", c.id, s.name);
            }
        }
    }

    #[test]
    fn generated_masks_use_the_enum_order() {
        let kinds: Vec<_> = Kind::ALL.iter().map(|k| k.name()).collect();
        let intents: Vec<_> = Intent::ALL.iter().map(|i| i.name()).collect();
        assert_eq!(kinds, crate::roster_data::KINDS);
        assert_eq!(intents, crate::roster_data::INTENTS);
    }

    #[test]
    fn variants_resolve_to_their_champion() {
        assert_eq!(def("tfm2_custom_gojo_v2").map(|d| d.name), Some("Gojo"));
        assert_eq!(def("tfm2_valorant_omen").map(|d| d.short()), Some("omen"));
        assert!(def("tfm2_custom").is_none());
        assert!(def("nobody").is_none());
    }

    #[test]
    fn queries_read_like_the_roster() {
        assert!(can("tfm2_valorant_omen", Intent::Escape));
        assert!(can("tfm2_valorant_omen", Intent::Vision));
        assert!(skills_for("tfm2_valorant_omen", Intent::Clear).any(|s| s.name == "Paranoia"));
        assert!(champions_with(Kind::Dash).any(|c| c.name == "Levi"));
        assert!(champions_with(Kind::Construct).all(|c| c.name == "Steve"));
        let line = def("tfm2_custom_gojo").unwrap().line();
        assert!(line.starts_with("Gojo – Jujutsu Kaisen – Blue [Projectile, Cc | Clear, Engage]"), "{line}");
        assert!(line.ends_with("BA: Strike (melee)"), "{line}");
    }

    #[test]
    fn map_changers_are_steve_and_omen() {
        let mut keys: Vec<_> = map_changers().map(|c| c.short()).collect();
        keys.sort();
        assert_eq!(keys, ["omen", "steve"]);
    }

    #[test]
    fn family_rules_reach_both_sides() {
        let david = def("tfm2_cyberpunk_david").unwrap();
        let gojo = def("tfm2_custom_gojo").unwrap();
        assert!(david.family_rules().any(|r| r.id == "sandevistan_vs_infinity"));
        assert!(gojo.family_rules().any(|r| r.id == "sandevistan_vs_infinity"));
        assert!(gojo.family_rules().any(|r| r.id == "parry"), "parry applies to everyone");
    }
}
