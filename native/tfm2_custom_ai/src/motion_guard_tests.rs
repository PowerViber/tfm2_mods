//! Round 156: no new hand-slid motion. `entity_set_pos` draws as a teleport in the game, so anything the player
//! should *see* moving (a dash, a flight, a knockback) must go through `crate::motion`. This test counts the remaining
//! `entity_set_pos(` calls per source file against an allowlist of real teleports, unseen units and corrective snaps.
//! A new champion that slides units by hand fails here: use `motion::step` / `motion::glide` instead, or, for a real
//! teleport, add the file with its reason below.

/// (file, calls, why they are allowed)
const ALLOWED: &[(&str, usize, &str)] = &[
    ("batch2.rs", 5, "domain seal snap, DIO time-stop blinks, DIO's vanish-dash, Stark's leap (teleports with fx)"),
    ("coder.rs", 1, "blink function (a teleport)"),
    ("contender.rs", 1, "Slip under banish (unseen)"),
    ("contender_arena.rs", 2, "Main Event ring transfer and exact returns (teleports)"),
    ("flash.rs", 1, "Flash (a blink)"),
    ("frieren.rs", 1, "Stark summon placement"),
    ("gojo.rs", 1, "domain seal snap"),
    ("gojo_v2.rs", 1, "domain seal snap"),
    ("grimhald.rs", 1, "Iron Ghost walk while banished (unseen)"),
    ("grimhald_cage.rs", 1, "cage edge correction (a deliberate snap)"),
    ("grimhald_weapon.rs", 1, "a test string asserting the weapon layer never moves him"),
    ("gundam.rs", 1, "Wings of Light flight while banished (unseen) and its landing"),
    ("isliid.rs", 6, "small sword displacements (candidates for motion::step later)"),
    ("lib.rs", 12, "Minato's Flying Raijin teleports, Gojo domain snaps, place::move_unit (instant placement helper)"),
    ("magnus.rs", 2, "chess pieces (summons) and his board step"),
    ("raze.rs", 1, "hidden satchel and Boom Bot carriers (unseen units)"),
    ("scribble.rs", 7, "doors, holes, redraw, rewind and revive (teleports by design)"),
    ("steve.rs", 9, "boat ride and pearl (Steve's own movement model; boat to be reviewed)"),
    ("steve_craft.rs", 2, "pearl teleport and crafted placements"),
    ("unified_theory.rs", 1, "equation teleport"),
    ("valorant.rs", 4, "Omen's teleports and shadow units"),
    ("vesper.rs", 1, "rift punch exit (a teleport through a tear)"),
];

#[test]
fn every_entity_set_pos_is_a_known_teleport_or_snap() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut seen = vec![];
    for e in std::fs::read_dir(&dir).unwrap().flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".rs") || name.ends_with("tests.rs") || name == "motion.rs" {
            continue;
        }
        let text = std::fs::read_to_string(e.path()).unwrap();
        let n = text.matches("entity_set_pos(").count();
        if n == 0 {
            continue;
        }
        let allowed = ALLOWED.iter().find(|a| a.0 == name).map_or(0, |a| a.1);
        assert!(
            n <= allowed,
            "{name} has {n} entity_set_pos calls (allowed {allowed}). Anything shown moving must use crate::motion::step \
             or crate::motion::glide (a hand-set position draws as a teleport); a real teleport needs an ALLOWED entry"
        );
        seen.push(name);
    }
    // the five converted kits must stay converted
    for name in ["levi.rs", "rally.rs"] {
        assert!(!seen.contains(&name.to_string()), "{name} slides by hand again");
    }
}

#[test]
fn motion_steps_through_the_engine_unless_the_old_path_is_asked_for() {
    use crate::glue_tests::{with_sim, Host, Unit};
    use mod_api_stable::{CcKindV1, EntityKindV1};
    let host = Host::new(vec![Unit { kind: EntityKindV1::Champion, team: 0, pos: (200_000, 200_000), hp: 1000, name: "x", targetable: true }]);
    host.borrow_mut().tick = 5;
    crate::motion::set_legacy_for_test(false);
    with_sim(&host, |sim| assert!(crate::motion::step(sim, 0, (203_000, 204_000), crate::motion::Kind::SelfMove)));
    {
        let h = host.borrow();
        assert!(h.moves.is_empty(), "no hand-set position");
        let (c, _) = h.ccs[0].iter().find(|c| c.0.kind == CcKindV1::ForceMove.code()).expect("forced move");
        assert_eq!(c.name(), crate::motion::SELF);
        assert_eq!(c.speed, 5_000);
        assert_eq!((c.dx, c.dy), (600, 800));
        assert!(crate::motion::is_self_motion(c));
    }
    crate::motion::set_legacy_for_test(true);
    with_sim(&host, |sim| assert!(crate::motion::glide(sim, 0, (210_000, 200_000), 10, crate::motion::Kind::Knock)));
    assert_eq!(host.borrow().moves.last().map(|m| m.1), Some((210_000, 200_000)), "motion.flag: the old placement");
    crate::motion::set_legacy_for_test(false);
}
