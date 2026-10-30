//! Deliberately measures host behaviour without supplying a substitute AI, damage model or arena filter.
use crate::transaction::{Pair, Point, World};
use mod_api_stable::{BuffDurationV1, BuffV1, SkillSlotV1, StableMatchHook, StableSim};
use serde_json::{json, Value};
use std::collections::BTreeMap;

include!(concat!(env!("OUT_DIR"), "/config.rs"));
const ARM_TICK: usize = 300;
const DONE: &str = "ctp_probe_started";
const DODGE: &str = "ctp_probe_dodge";

pub struct Probe;
struct Adapter<'a, 'b>(&'a mut StableSim<'b>);
impl World for Adapter<'_, '_> {
    fn position(&self, id: usize) -> Option<Point> {
        self.0.get_entity(id).map(|e| e.pos())
    }
    fn alive(&self, id: usize) -> bool {
        self.0.get_entity(id).is_some_and(|e| e.is_alive())
    }
    fn teleport(&mut self, id: usize, point: Point) -> bool {
        self.0.entity_set_pos(id, point.0, point.1)
    }
}

fn champions(sim: &StableSim<'_>) -> Vec<usize> {
    let mut ids: Vec<_> = (0..sim.entity_count())
        .filter_map(|i| sim.entity_at(i))
        .filter(|e| e.is_champion())
        .map(|e| e.id())
        .collect();
    ids.sort_unstable();
    ids
}
fn has(sim: &StableSim<'_>, id: usize, name: &str) -> bool {
    sim.get_entity(id).is_some_and(|e| {
        (0..e.buff_count()).any(|i| e.buff_at(i).is_some_and(|b| b.name() == name))
    })
}
fn inside(point: Point) -> bool {
    (ORIGIN.0..=ORIGIN.0 + SIDE).contains(&point.0)
        && (ORIGIN.1..=ORIGIN.1 + SIDE).contains(&point.1)
}
fn starts() -> [Point; 2] {
    [
        (ORIGIN.0 + (SIDE - SEPARATION) / 2, ORIGIN.1 + SIDE / 2),
        (ORIGIN.0 + (SIDE + SEPARATION) / 2, ORIGIN.1 + SIDE / 2),
    ]
}
#[cfg(not(test))]
fn mode() -> &'static str {
    if cfg!(feature = "off-map-probe") {
        "off-map"
    } else if cfg!(feature = "dodge-probe") {
        "dodge-candidate"
    } else {
        "shield-observation"
    }
}

/// Files are diagnostic only. Instance identity never affects any simulation command.
/// At most ~100 samples plus bounded events per simulation; failed I/O changes no decisions.
fn record(sim: &StableSim<'_>, event: &str, detail: Value) {
    #[cfg(not(test))]
    {
        use std::{fs::OpenOptions, io::Write};
        if sim.tick() > ARM_TICK + DURATION + 120 {
            return;
        }
        let Some(base) = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_owned()))
        else {
            return;
        };
        let dir = base.join("mods/tfm2_contender_probe/logs");
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        let origin = sim.sim_origin();
        let name = format!(
            "{}-{}-{}-{:x}.jsonl",
            mode(),
            sim.seed(),
            origin.map_or(0, |o| o.match_id),
            sim.instance()
        );
        let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join(name))
        else {
            return;
        };
        // Replay seeks may revisit the same ticks. Bound the file as well as the capture interval.
        if file.metadata().map_or(true, |m| m.len() >= 2 * 1024 * 1024) {
            return;
        }
        let row = json!({"schema":1,"scope":"host-probe-not-champion-acceptance","mode":mode(),"tick":sim.tick(),"seed":sim.seed(),"origin":origin.map(|o|json!({"kind":o.kind,"match":o.match_id,"replay":o.replay_id,"set":o.set_index})),"event":event,"detail":detail});
        if let Ok(bytes) = serde_json::to_vec(&row) {
            if bytes.len() > 65_536 {
                let omitted = format!(
                    "{{\"schema\":1,\"tick\":{},\"event\":\"oversized-observation-omitted\"}}\n",
                    sim.tick()
                );
                let _ = file.write_all(omitted.as_bytes());
                return;
            }
            let _ = file.write_all(&bytes);
            let _ = file.write_all(b"\n");
        }
    }
    #[cfg(test)]
    let _ = (sim, event, detail);
}

fn snapshot(sim: &StableSim<'_>, ids: &[usize]) -> Value {
    Value::Array(ids.iter().filter_map(|&id| sim.get_entity(id)).map(|e| {
        let cooldowns: Vec<_> = [SkillSlotV1::Skill, SkillSlotV1::Skill2, SkillSlotV1::Ult].into_iter().map(|s| e.cooldown(s).map(|c| json!({"remaining":c.remaining,"charges":c.charges}))).collect();
        json!({"id":e.id(),"name":e.name(),"team":e.team(),"alive":e.is_alive(),"position":e.pos(),"hp":e.hp(),"shield":e.shield(),"targetable":e.is_targetable(),"cc":(0..e.cc_count()).filter_map(|i|e.cc_at(i)).map(|c|json!({"kind":c.kind,"remaining":c.tick})).collect::<Vec<_>>(),"cooldowns":cooldowns})
    }).collect())
}

fn ledger(sim: &StableSim<'_>, ids: &[usize]) -> BTreeMap<String, Pair> {
    let mut result = BTreeMap::new();
    for &id in ids {
        if let Some(e) = sim.get_entity(id) {
            for i in 0..e.buff_count() {
                if let Some(b) = e.buff_at(i) {
                    if let Some(pair) = Pair::parse(b.name()).filter(|p| p.ids.contains(&id)) {
                        result.insert(b.name().to_owned(), pair);
                    }
                }
            }
        }
    }
    result
}

fn off_map(sim: &mut StableSim<'_>, ids: &[usize]) {
    let existing = ledger(sim, ids);
    for (marker, pair) in &existing {
        let must_return = sim.is_end()
            || sim.tick() >= pair.end_tick
            || pair.ids.iter().any(|&id| {
                sim.get_entity(id)
                    .is_none_or(|e| !e.is_alive() || !inside(e.pos()))
            });
        if must_return {
            let returned = pair.restore(&mut Adapter(sim));
            if returned {
                for &id in &pair.ids {
                    sim.entity_remove_buff(id, marker);
                }
            }
            if returned || sim.tick() % 60 == 0 {
                record(
                    sim,
                    "return",
                    json!({"pair":pair,"exactReturn":returned,"state":snapshot(sim,&pair.ids)}),
                );
            }
        }
    }
    if sim.tick() != ARM_TICK || !existing.is_empty() || ids.iter().any(|&id| has(sim, id, DONE)) {
        return;
    }
    let first = ids.iter().copied().find(|&id| {
        sim.get_entity(id)
            .is_some_and(|e| e.is_alive() && e.team() == 0 && !has(sim, id, DONE))
    });
    let second = ids.iter().copied().find(|&id| {
        sim.get_entity(id)
            .is_some_and(|e| e.is_alive() && e.team() == 1 && !has(sim, id, DONE))
    });
    let Some(pair) = first
        .zip(second)
        .and_then(|(a, b)| Pair::capture(&Adapter(sim), [a, b], ARM_TICK + DURATION))
    else {
        record(sim, "no-pair", json!({}));
        return;
    };
    let marker = pair.marker();
    if marker.len() > mod_api_stable::BUFF_NAME_CAP {
        record(sim, "ledger-rejected", json!({"reason":"marker capacity"}));
        return;
    }
    for &id in &pair.ids {
        sim.add_buff(id, &BuffV1::named(DONE));
        sim.add_buff(id, &BuffV1::named(&marker));
    }
    if !pair.ids.iter().all(|&id| has(sim, id, &marker)) {
        for &id in &pair.ids {
            sim.entity_remove_buff(id, &marker);
        }
        record(
            sim,
            "ledger-rejected",
            json!({"reason":"host did not retain both copies"}),
        );
        return;
    }
    record(
        sim,
        "before-transfer",
        json!({"pair":pair,"state":snapshot(sim,&pair.ids)}),
    );
    let entered = pair.enter(&mut Adapter(sim), starts());
    if !entered && pair.restore(&mut Adapter(sim)) {
        for &id in &pair.ids {
            sim.entity_remove_buff(id, &marker);
        }
    }
    record(
        sim,
        "transfer",
        json!({"pair":pair,"starts":starts(),"acceptedAndReadBack":entered,"state":snapshot(sim,&pair.ids)}),
    );
}

impl StableMatchHook for Probe {
    fn on_match_tick(&self, sim: &mut StableSim<'_>, _: u64) {
        let ids = champions(sim);
        if cfg!(feature = "off-map-probe") {
            off_map(sim, &ids);
        }
        if sim.tick() == ARM_TICK
            && !cfg!(feature = "off-map-probe")
            && !ids.iter().any(|&id| has(sim, id, DONE))
        {
            if let Some(id) = ids.iter().copied().find(|&id| {
                sim.get_entity(id)
                    .is_some_and(|e| e.is_alive() && e.team() == 0 && !has(sim, id, DONE))
            }) {
                sim.add_buff(id, &BuffV1::named(DONE));
                record(sim, "before-candidate", snapshot(sim, &[id]));
                if cfg!(feature = "dodge-probe") {
                    // Candidate only: whether banish/100% reduction excludes Fixed/DoT and all CC is UNPROVEN.
                    let mut b = BuffV1::named(DODGE);
                    b.duration_kind = BuffDurationV1::Time.code();
                    b.duration_tick = DODGE_TICKS;
                    b.damaged_reduce = 100;
                    b.cc_immune = true;
                    sim.add_buff(id, &b);
                    let accepted = sim.entity_banish(id, id, DODGE_TICKS, "", "");
                    record(
                        sim,
                        "candidate-banish",
                        json!({"accepted":accepted,"state":snapshot(sim,&[id])}),
                    );
                } else {
                    let accepted = sim.entity_add_shield(id, 1000, 90);
                    record(
                        sim,
                        "candidate-shield",
                        json!({"accepted":accepted,"state":snapshot(sim,&[id])}),
                    );
                }
            }
        }
        if cfg!(feature = "dodge-probe") && sim.tick() == ARM_TICK + DODGE_TICKS {
            for &id in &ids {
                sim.entity_remove_buff(id, DODGE);
            } // Only our stat buff; never clear unrelated CC/shields.
            record(sim, "dodge-window-ended", snapshot(sim, &ids));
        }
        if (ARM_TICK..=ARM_TICK + DURATION + 6).contains(&sim.tick())
            && (sim.tick() % 6 == 0
                || (cfg!(feature = "dodge-probe") && sim.tick() <= ARM_TICK + DODGE_TICKS + 1))
        {
            record(sim, "sample", snapshot(sim, &ids));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starts_follow_canonical_geometry() {
        let p = starts();
        assert_eq!(p[1].0 - p[0].0, SEPARATION);
        assert!(p.into_iter().all(inside));
        assert!(ORIGIN.0 > 960_000);
    }
    #[test]
    fn off_map_is_not_accepted_as_original_return_point() {
        assert!(Pair::parse("ctp1:1:2:1176000:240000:1224000:240000:780").is_none());
    }
    #[test]
    fn shared_dimensions_are_the_approved_values() {
        assert_eq!(
            (SIDE, SEPARATION, DURATION, DODGE_TICKS),
            (160_000, 48_000, 480, 12)
        );
    }
    #[test]
    fn marker_capacity_is_checked() {
        let p = Pair {
            ids: [10, 20],
            entries: [(123456, 234567), (654321, 765432)],
            end_tick: 780,
        };
        assert!(p.marker().len() <= mod_api_stable::BUFF_NAME_CAP);
        assert!(p.marker().starts_with(crate::transaction::PREFIX));
    }
}

#[cfg(all(test, feature = "off-map-probe"))]
#[path = "host_tests.rs"]
mod host_tests;
