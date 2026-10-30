//! Experimental host measurements, NOT the Contender champion or an isolation implementation.
//! No experiment runs in the default build. Keep this crate outside the production mods tree.
use mod_api_stable::{declare_stable_mod, LogLevel, StableHost, StableMod};

#[cfg(any(
    all(feature = "off-map-probe", feature = "dodge-probe"),
    all(feature = "off-map-probe", feature = "shield-probe"),
    all(feature = "dodge-probe", feature = "shield-probe")
))]
compile_error!("Select exactly one probe, or none for the inert build.");

#[cfg(any(
    feature = "off-map-probe",
    feature = "dodge-probe",
    feature = "shield-probe"
))]
mod host;
#[cfg(any(
    test,
    feature = "off-map-probe",
    feature = "dodge-probe",
    feature = "shield-probe"
))]
mod transaction;

fn init(host: &StableHost) -> StableMod {
    let mut declaration = StableMod::new("tfm2_contender_probe");
    #[cfg(any(
        feature = "off-map-probe",
        feature = "dodge-probe",
        feature = "shield-probe"
    ))]
    {
        host.log(LogLevel::Warn, "CONTENDER HOST PROBE: experimental match mutations at tick 300. Isolated disposable matches only. No arena isolation or release approval.");
        declaration.set_match_hook(host::Probe);
    }
    #[cfg(not(any(
        feature = "off-map-probe",
        feature = "dodge-probe",
        feature = "shield-probe"
    )))]
    host.log(
        LogLevel::Warn,
        "Contender probe is INERT. No champion, hooks, artwork or balance changes registered.",
    );
    // The builder must be mutable only in experimental builds.
    let _ = &mut declaration;
    declaration
}

declare_stable_mod!(init, requires = 10);
