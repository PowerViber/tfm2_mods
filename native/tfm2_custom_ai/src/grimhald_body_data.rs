//! Generated from rules/grimhald-art.json.
pub const WEAPON_PREFIX: &str = "grh_weapon_";
pub fn durations(name:&str)-> &'static [u64] { match name {
"idle"=> &[180, 180, 180, 180],
"run"=> &[80, 80, 80, 80, 80, 80],
"attack"=> &[80, 80, 80, 80],
"skill1"=> &[150, 150, 150, 150],
"skill2"=> &[80, 80, 80, 80],
"ult"=> &[100, 100, 100, 100],
"hit"=> &[140],
"dead"=> &[100, 100, 100, 100, 100, 100],
_=> &[180],
}}
