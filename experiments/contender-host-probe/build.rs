use std::{env, fs, path::PathBuf};

fn main() {
    let rules = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../rules/contender.json");
    println!("cargo:rerun-if-changed={}", rules.display());
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(rules).expect("canonical Contender rules")).unwrap();
    let number = |p: &str| value.pointer(p).and_then(|v| v.as_u64()).expect(p);
    let out = format!("pub const SIDE: u64 = {};\npub const ORIGIN: (u64,u64) = ({},{});\npub const SEPARATION: u64 = {};\npub const DURATION: usize = {};\npub const DODGE_TICKS: usize = {};\n", number("/arena/interiorSide"), number("/arena/prototypeOrigin/0"), number("/arena/prototypeOrigin/1"), number("/arena/startingSeparation"), number("/ultimate/durationTicks"), number("/slip/durationTicks"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out_dir.join("config.rs"), out).unwrap();
    // The original battlefield's edge; transaction.rs compiles without host.rs in tests.
    fs::write(
        out_dir.join("map.rs"),
        format!(
            "pub const MAP_LIMIT: u64 = {};\n",
            number("/arena/mapLimit")
        ),
    )
    .unwrap();
}
