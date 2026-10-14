// Illustrative guide example, not a champion shipped in the repository.
// Addendum modules (each standalone, with tests): overlay facing, publishing danger zones for the game's AI, and
// an effect budget. See docs/champion-guide-addendum.md in the repository.
pub mod budget;
pub mod facing;
pub mod zones;
use mod_api_stable::{declare_stable_mod, AttackTypeV1, LogLevel,
    StableHost, StableMod, StablePassive, StableSim};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Fighter { id: usize, x: i64, y: i64, attack: usize }
#[derive(Clone, Debug, PartialEq, Eq)]
enum Cmd { Damage { target: usize, amount: usize } }
#[derive(Clone)]
struct Core { ready_at: usize, rng: u64 }

impl Core {
    fn new(seed: u64, owner: usize) -> Self {
        Self { ready_at: 0,
            rng: (seed ^ (owner as u64).wrapping_mul(0x9e3779b97f4a7c15)).max(1) }
    }
    fn roll(&mut self, n: u64) -> u64 {
        let mut x = self.rng;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.rng = x;
        x % n.max(1)
    }
    fn step(&mut self, tick: usize, me: &Fighter,
        enemies: &[Fighter], blocked: bool) -> Vec<Cmd> {
        if blocked || tick < self.ready_at { return vec![]; }
        let nearest = enemies.iter().filter_map(|e| {
            let dx = (e.x - me.x) as i128;
            let dy = (e.y - me.y) as i128;
            let d2 = dx * dx + dy * dy;
            (d2 <= 70_000i128 * 70_000).then_some((d2, e.id))
        }).min(); // Explicit entity-id tie break.
        let Some((_, target)) = nearest else { return vec![]; };
        self.ready_at = tick.saturating_add(300);
        let variance = self.roll(11) as usize; // 0..10, seeded.
        vec![Cmd::Damage { target,
            amount: 40usize.saturating_add(me.attack / 2).saturating_add(variance) }]
    }
}

#[derive(Clone, Default)]
struct Burst { core: Option<Core>, last_tick: Option<usize> }
impl StablePassive for Burst {
    fn clone_box(&self) -> Box<dyn StablePassive> { Box::new(self.clone()) }
    fn on_dead(&mut self, _: &mut StableSim<'_>, _: usize) {
        self.core = None; self.last_tick = None;
    }
    fn on_update(&mut self, sim: &mut StableSim<'_>, _: u64,
        _: usize, entity: usize) {
        let tick = sim.tick();
        if self.last_tick == Some(tick) { return; }
        let Some(e) = sim.get_entity(entity) else { return; };
        if !e.is_alive() { return; }
        let (x, y) = e.pos();
        let team = e.team();
        let blocked = e.cc_count() > 0; // Deliberately conservative prototype.
        let me = Fighter { id: entity, x: x as i64, y: y as i64,
            attack: e.stat().attack };
        let enemies = (0..sim.entity_count())
            .filter_map(|i| sim.entity_at(i))
            .filter(|t| t.is_champion() && t.is_alive() && t.is_targetable()
                && t.team() != team && sim.is_visible(team, t.id()))
            .map(|t| { let (x, y) = t.pos();
                Fighter { id: t.id(), x: x as i64, y: y as i64,
                    attack: t.stat().attack } }).collect::<Vec<_>>();
        self.last_tick = Some(tick);
        let core = self.core.get_or_insert_with(|| Core::new(sim.seed(), entity));
        for cmd in core.step(tick, &me, &enemies, blocked) {
            match cmd {
                Cmd::Damage { target, amount } =>
                    sim.deal_damage(entity, target, amount, 0, AttackTypeV1::Skill),
            }
        }
    }
}

fn init(host: &StableHost) -> StableMod {
    host.log(LogLevel::Info, "guide_rules 0.1.0 loaded");
    let mut m = StableMod::new("guide_rules");
    m.add_native_passive("guide_rules:burst", Burst::default());
    m
}
declare_stable_mod!(init, requires = 9);

#[cfg(test)]
mod tests {
    use super::*;
    fn f(id: usize, x: i64) -> Fighter { Fighter { id, x, y: 10_000, attack: 80 } }
    #[test]
    fn cooldown_and_target_tie() {
        let me = f(0, 100_000);
        let enemies = vec![f(2, 110_000), f(1, 90_000)];
        let mut c = Core::new(77, 0);
        assert!(matches!(c.step(0, &me, &enemies, false).as_slice(),
            [Cmd::Damage { target: 1, amount: 80..=90 }]));
        assert!(c.step(299, &me, &enemies, false).is_empty());
        assert_eq!(c.step(300, &me, &enemies, false).len(), 1);
    }
    #[test]
    fn blocked_or_empty_does_not_spend_rng_or_cooldown() {
        let mut c = Core::new(77, 0);
        let initial = c.clone();
        assert!(c.step(0, &f(0, 100_000), &[f(1, 110_000)], true).is_empty());
        assert!(c.step(1, &f(0, 100_000), &[], false).is_empty());
        assert_eq!((c.ready_at, c.rng), (initial.ready_at, initial.rng));
    }
    #[test]
    fn replay_and_unrelated_instance_are_independent() {
        let mut a = Core::new(77, 0);
        let mut b = a.clone();
        let mut unrelated = Core::new(91, 3);
        for t in 0..1800 {
            let _ = unrelated.step(t, &f(3, 100_000), &[f(4, 105_000)], false);
            assert_eq!(a.step(t, &f(0, 100_000), &[f(1, 110_000)], false),
                b.step(t, &f(0, 100_000), &[f(1, 110_000)], false));
        }
        assert_eq!((a.ready_at, a.rng), (b.ready_at, b.rng));
    }
}
