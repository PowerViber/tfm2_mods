//! Creative catalogue, equipment and bounded owner-local redstone. Artwork is selected separately.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
pub const SPEED: u8 = 0;
pub const HASTE: u8 = 1;
pub const STRENGTH: u8 = 2;
pub const RESISTANCE: u8 = 3;
pub const POISON: u8 = 4;
pub const WEAKNESS: u8 = 5;
pub const IRON: u8 = 6;
pub const DIAMOND: u8 = 7;
pub const NETHERITE: u8 = 8;
pub const PROTECTION: u8 = 9;
pub const THORNS: u8 = 10;
pub const TOTEM: u8 = 11;
pub const REGEN: u8 = 12;
pub const SLOW: u8 = 13;
#[derive(Clone, Debug)]
pub struct Effect {
    pub target: usize,
    pub kind: u8,
    pub power: i64,
    pub end: usize,
    pub next: usize,
    pub interval: usize,
}
#[derive(Clone, Debug)]
pub struct Armour {
    pub target: usize,
    pub recipe: u8,
    pub end: usize,
    pub enchant: Option<u8>,
}
#[derive(Clone, Debug)]
pub struct Totem {
    pub target: usize,
    pub end: usize,
    pub shield: i64,
}
#[derive(Clone, Debug)]
pub struct Payload {
    pub node: u32,
    pub recipe: u8,
    pub target: Option<usize>,
    pub point: Point,
    pub hp: i64,
}
#[derive(Clone, Debug)]
pub struct Pulse {
    pub event: u64,
    pub node: u32,
    pub at: usize,
    pub path: Vec<u32>,
}
#[derive(Clone, Debug)]
pub struct Bounce {
    pub object: u32,
    pub target: usize,
    pub at: usize,
    pub point: Point,
    pub destination: Point,
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub effects: Vec<Effect>,
    pub armour: Vec<Armour>,
    pub totems: Vec<Totem>,
    pub totem_lock: BTreeMap<usize, usize>,
    pub links: BTreeMap<u32, Vec<u32>>,
    pub settings: BTreeMap<u32, (u8, usize)>,
    pub armed: BTreeSet<u32>,
    pub payloads: Vec<Payload>,
    pub pulses: Vec<Pulse>,
    pub seen: BTreeSet<(u64, u32)>,
    pub serial: u64,
    pub bounces: Vec<Bounce>,
    pub aura_clocks: BTreeMap<(u32, usize), usize>,
    pub aura_winners: BTreeMap<(usize, u8), (u32, i64)>,
    pub aura_restores: BTreeSet<(usize, u8)>,
}
impl State {
    pub fn clear(&mut self) {
        let locks = std::mem::take(&mut self.totem_lock);
        *self = Self::default();
        self.totem_lock = locks;
    }
    pub fn json(&self) -> String {
        let links = self
            .links
            .iter()
            .flat_map(|(a, bs)| bs.iter().map(move |b| format!("[{a},{b}]")))
            .collect::<Vec<_>>()
            .join(",");
        let effects = self
            .effects
            .iter()
            .map(|e| format!("[{},{},{},{},{}]", e.target, e.kind, e.power, e.end, e.next))
            .collect::<Vec<_>>()
            .join(",");
        let armour = self
            .armour
            .iter()
            .map(|e| {
                format!(
                    "[{},{},{},{}]",
                    e.target,
                    e.recipe,
                    e.end,
                    e.enchant.map_or(-1, |v| v as i64)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let payloads = self
            .payloads
            .iter()
            .map(|e| {
                format!(
                    "[{},{},{},{},{}]",
                    e.node,
                    e.recipe,
                    e.target.map_or(-1, |v| v as i64),
                    e.point.0,
                    e.point.1
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"links\":[{links}],\"effects\":[{effects}],\"armour\":[{armour}],\"payloads\":[{payloads}],\"armed\":{:?}}}",self.armed.iter().copied().collect::<Vec<_>>())
    }
}

fn ally<'a>(i: &'a Input, id: Option<usize>) -> Option<&'a Unit> {
    i.units.iter().find(|u| {
        Some(u.id) == id
            && u.team == i.me.team
            && u.id != i.me.id
            && u.champion
            && u.hp > 0
            && u.targetable
    })
}
fn friendly_recipe(r: u8) -> bool {
    matches!(r,23..=25|28..=32|40..=41)
}
fn physical(r: u8) -> bool {
    matches!(r,15..=19|33..=36|39)
}
fn edit_allowed(r: u8) -> bool {
    matches!(r,0..=2|15..=19|33..=34|37..=38)
}
fn network_recipe(r: u8) -> bool {
    matches!(r, 1 | 3 | 33..=36)
}
pub(super) fn hp_hit(u: &Unit, base: i64, hp: i64, permyriad: i64) -> i64 {
    let component = hp * permyriad / 10000;
    base + if u.epic {
        component.min(EPIC_HP_DAMAGE_CAP)
    } else {
        component
    }
}
impl Core {
    pub(super) fn refresh_aura_winners(&mut self, i: &Input) {
        let mut winners = BTreeMap::new();
        for o in self.objects.iter().filter(|o| {
            o.confirmed
                && o.hp > 0
                && i.tick < o.end
                && matches!(o.recipe, 19 | 26)
                && i.objects
                    .iter()
                    .find(|(id, _)| *id == o.id)
                    .is_none_or(|(_, hp)| *hp > 0)
        }) {
            for u in i.units.iter().filter(|u| {
                u.hp > 0
                    && u.targetable
                    && if o.recipe == 19 {
                        u.team == i.me.team
                            && u.id != i.me.id
                            && u.champion
                            && within(u.pos(), o.point, 80000)
                    } else {
                        u.team != i.me.team && !u.tower && within(u.pos(), o.point, 22000)
                    }
            }) {
                let (kind, score) = if o.recipe == 19 {
                    (REGEN, 8 + o.power_hp * 70 / 10000)
                } else {
                    (POISON, hp_hit(u, 3, o.power_hp, 35) * 2)
                };
                let winner = winners.entry((u.id, kind)).or_insert((o.id, score));
                if score > winner.1 || (score == winner.1 && o.id < winner.0) {
                    *winner = (o.id, score);
                }
            }
        }
        self.creative.aura_restores = self
            .creative
            .aura_winners
            .iter()
            .filter(|(key, (_, score))| winners.get(key).is_none_or(|(_, next)| next < score))
            .map(|(key, _)| *key)
            .collect();
        self.creative.aura_winners = winners;
    }
    fn rotation(&self, i: &Input, id: u32, heading: u8) -> Result<(Point, Point), &'static str> {
        let o = self
            .objects
            .iter()
            .find(|o| o.id == id && o.confirmed && o.hp > 0)
            .ok_or("Select a living owned construction")?;
        if heading >= 16 || !edit_allowed(o.recipe) || !within(i.me.pos(), o.point, 70000) {
            return Err("Construction cannot be rotated here");
        }
        let h = heading as usize * 16;
        let direction = (SIN[(h + 64) % 256], SIN[h]);
        if o.recipe == 0 {
            let a = (
                o.point.0 - direction.0 * 30000 / 10000,
                o.point.1 - direction.1 * 30000 / 10000,
            );
            let b = (
                o.point.0 + direction.0 * 30000 / 10000,
                o.point.1 + direction.1 * 30000 / 10000,
            );
            if !legal(a)
                || !legal(b)
                || !clear(a, b, i.wall)
                || i.units.iter().any(|u| {
                    u.team == i.me.team
                        && u.hp > 0
                        && !u.gadget
                        && distance_segment(u.pos(), a, b) < 12000
                })
            {
                return Err("Rotated wall would intersect terrain or an ally");
            }
        }
        Ok((o.point, direction))
    }
    pub(super) fn creative_candidates(&self, i: &Input, a: &mut Vec<(Action, i64)>) {
        if self.form == Form::Steve {
            if let Some(o) = self
                .objects
                .iter()
                .filter(|o| {
                    o.confirmed && o.hp > 0 && o.hp < o.max_hp && within(i.me.pos(), o.point, 40000)
                })
                .min_by_key(|o| (o.hp * 100 / o.max_hp.max(1), o.id))
            {
                a.push((Action::Repair(o.id), 120));
            }
            return;
        }
        let foe = i
            .units
            .iter()
            .filter(|u| {
                u.team != i.me.team
                    && u.hp > 0
                    && u.visible
                    && u.targetable
                    && !u.immune
                    && (u.champion || u.farmable)
            })
            .min_by_key(|u| (!u.champion, square(i.me.pos(), u.pos()), u.id));
        let friend = i
            .units
            .iter()
            .filter(|u| {
                u.team == i.me.team
                    && u.id != i.me.id
                    && u.hp > 0
                    && u.champion
                    && within(i.me.pos(), u.pos(), 60000)
            })
            .min_by_key(|u| (u.pct(), u.id));
        for r in 15..45u8 {
            if r == 42 {
                continue;
            }
            let friendly = friendly_recipe(r);
            let target = if friendly {
                friend.map(|u| u.id)
            } else if matches!(r, 43..=44) {
                foe.map(|u| u.id)
            } else {
                None
            };
            if friendly && friend.is_none() {
                continue;
            }
            if !friendly && foe.is_none() {
                continue;
            }
            let point = if friendly {
                friend.unwrap().pos()
            } else if r <= 19 {
                toward(i.me.pos(), foe.unwrap().pos(), 20000)
            } else {
                toward(i.me.pos(), foe.unwrap().pos(), RECIPES[r as usize].range)
            };
            let score = match r {
                15..=17 => 75,
                18..=19 => {
                    if friend.is_some_and(|u| u.pct() < 60) {
                        145
                    } else {
                        55
                    }
                }
                20..=22 | 26..=27 | 43..=44 => {
                    if foe.is_some_and(|u| u.champion) {
                        115
                    } else {
                        55
                    }
                }
                23..=24 => {
                    if friend.unwrap().pct() < 65 {
                        240 - friend.unwrap().pct() * 2
                    } else {
                        -1
                    }
                }
                25 => 70,
                28..=30 => 100 + (r as i64 - 28) * 5,
                31..=32 => {
                    if self
                        .creative
                        .armour
                        .iter()
                        .any(|a| Some(a.target) == target && a.end > i.tick)
                    {
                        115
                    } else {
                        -1
                    }
                }
                33 | 37..=39 => {
                    if foe.is_some_and(|u| u.champion) {
                        85
                    } else {
                        -1
                    }
                }
                34..=36 => -1,
                40 => {
                    if friend.unwrap().pct() < 50 {
                        90
                    } else {
                        -1
                    }
                }
                41 => {
                    if friend.unwrap().pct() < 35 {
                        300
                    } else {
                        -1
                    }
                }
                _ => -1,
            };
            if score > 0 {
                a.push((
                    Action::Recipe {
                        recipe: r,
                        target,
                        point,
                    },
                    score,
                ));
            }
        }
        for from in self
            .objects
            .iter()
            .filter(|o| o.confirmed && network_recipe(o.recipe))
        {
            if self.creative.armed.contains(&from.id) {
                continue;
            }
            if self
                .creative
                .links
                .get(&from.id)
                .is_some_and(|l| !l.is_empty())
            {
                a.push((Action::Arm(from.id), 80));
            }
        }
        if let Some(f) = foe.filter(|u| u.champion) {
            let plate = self.objects.iter().find(|o| o.recipe == 35 && o.confirmed);
            let dispenser = self.objects.iter().find(|o| o.recipe == 34 && o.confirmed);
            if i.me.pct() > 65 && self.available(i.tick) >= 2 {
                if plate.is_none() {
                    a.push((
                        Action::Recipe {
                            recipe: 35,
                            target: None,
                            point: toward(i.me.pos(), f.pos(), 30000),
                        },
                        125,
                    ));
                } else if dispenser.is_none() {
                    let p = plate.unwrap().point;
                    a.push((
                        Action::Recipe {
                            recipe: 34,
                            target: None,
                            point: (p.0 - 18000, p.1 + 12000),
                        },
                        125,
                    ));
                }
            }
            if let Some(d) = dispenser {
                if !self.creative.payloads.iter().any(|p| p.node == d.id) {
                    a.push((
                        Action::Load {
                            node: d.id,
                            recipe: 44,
                            target: Some(f.id),
                            point: f.pos(),
                        },
                        145,
                    ));
                } else if within(d.point, f.pos(), 90000)
                    && self
                        .creative
                        .payloads
                        .iter()
                        .any(|p| p.node == d.id && within(p.point, f.pos(), 18000))
                {
                    a.push((Action::Activate(d.id), 155));
                }
            }
            if let (Some(p), Some(d)) = (plate, dispenser) {
                if !self
                    .creative
                    .links
                    .get(&p.id)
                    .is_some_and(|v| v.contains(&d.id))
                {
                    a.push((
                        Action::Link {
                            from: p.id,
                            to: d.id,
                        },
                        180,
                    ));
                } else if !self.creative.armed.contains(&p.id)
                    && self.creative.payloads.iter().any(|v| v.node == d.id)
                {
                    a.push((Action::Arm(p.id), 190));
                }
            }
            for o in self.objects.iter().filter(|o| {
                o.confirmed
                    && edit_allowed(o.recipe)
                    && o.recipe != 0
                    && within(i.me.pos(), o.point, 70000)
            }) {
                if !within(o.point, f.pos(), 80000) && within(i.me.pos(), f.pos(), 80000) {
                    a.push((
                        Action::Recipe {
                            recipe: 42,
                            target: Some(o.id as usize),
                            point: toward(i.me.pos(), f.pos(), 25000),
                        },
                        95,
                    ));
                }
            }
        }
    }
    fn network_size(&self, from: u32, to: u32) -> usize {
        let mut stack = vec![from, to];
        let mut seen = BTreeSet::new();
        while let Some(id) = stack.pop() {
            if !seen.insert(id) {
                continue;
            }
            if let Some(next) = self.creative.links.get(&id) {
                stack.extend(next);
            }
            for (a, bs) in &self.creative.links {
                if bs.contains(&id) {
                    stack.push(*a);
                }
            }
        }
        seen.len()
    }
    fn cycle(&self, from: u32, to: u32) -> bool {
        let mut stack = vec![to];
        let mut seen = BTreeSet::new();
        while let Some(id) = stack.pop() {
            if id == from {
                return true;
            }
            if seen.insert(id) {
                if let Some(v) = self.creative.links.get(&id) {
                    stack.extend(v);
                }
            }
        }
        false
    }
    fn creative_valid(
        &self,
        i: &Input,
        r: u8,
        target: Option<usize>,
        point: Point,
        origin: Point,
    ) -> Result<(), &'static str> {
        let d = RECIPES.get(r as usize).ok_or("Unknown Creative recipe")?;
        if friendly_recipe(r) {
            let u = ally(i, target).ok_or("Needs another living allied champion")?;
            if !within(origin, u.pos(), d.range) {
                return Err("Ally beyond range");
            }
            if matches!(r, 31..=32)
                && u.gifted_armour_until <= i.tick
                && !self
                    .creative
                    .armour
                    .iter()
                    .any(|a| a.target == u.id && a.end > i.tick)
            {
                return Err("Enchantment requires active gifted armour");
            }
            if r == 41
                && (u.totem_locked
                    || self
                        .creative
                        .totem_lock
                        .get(&u.id)
                        .is_some_and(|&t| t > i.tick))
            {
                return Err("Recipient Totem lockout");
            }
        } else if r == 42 {
            let o = self
                .objects
                .iter()
                .find(|o| Some(o.id as usize) == target && o.confirmed && o.hp > 0)
                .ok_or("Select a living owned construction")?;
            if !edit_allowed(o.recipe) {
                return Err("Construction cannot be edited");
            }
            if !self.aimed(i, point, d.range) || Self::occupied(i, point) {
                return Err("Edited footprint unavailable");
            }
            if o.recipe == 0 {
                let delta = (point.0 - o.point.0, point.1 - o.point.1);
                if self
                    .walls
                    .iter()
                    .filter(|w| w.object == Some(o.id))
                    .flat_map(|w| w.path.windows(2))
                    .any(|p| {
                        !clear(
                            (p[0].0 + delta.0, p[0].1 + delta.1),
                            (p[1].0 + delta.0, p[1].1 + delta.1),
                            i.wall,
                        )
                    })
                {
                    return Err("Edited wall intersects terrain");
                }
            }
            for (a, bs) in &self.creative.links {
                for b in bs {
                    if *a == o.id || *b == o.id {
                        let other = if *a == o.id { *b } else { *a };
                        if self
                            .objects
                            .iter()
                            .find(|n| n.id == other)
                            .is_some_and(|n| !within(n.point, point, 32000))
                        {
                            return Err("Edit would break a redstone link");
                        }
                    }
                }
            }
        } else {
            if !legal(point)
                || !within(origin, point, d.range)
                || (i.wall)(point.0, point.1)
                || !clear(origin, point, i.wall)
            {
                return Err("Creative destination unavailable");
            }
            if d.cap > 0 && !self.capacity(r) {
                return Err("Creative construction capacity reached");
            }
            if physical(r) && Self::occupied(i, point) {
                return Err("Construction footprint occupied");
            }
            if r == 39
                && i.units.iter().any(|u| {
                    u.team == i.me.team
                        && u.hp > 0
                        && distance_segment(
                            u.pos(),
                            (point.0 - 30000, point.1),
                            (point.0 + 30000, point.1),
                        ) < 12000
                })
            {
                return Err("Obsidian would trap an ally");
            }
            if let Some(id) = target {
                if r != 42 && Self::enemy(i, id).is_none() {
                    return Err("Enemy target lost");
                }
            }
        }
        Ok(())
    }
    pub(super) fn creative_accept(&mut self, i: &Input, a: Action, out: &mut Vec<Cmd>) -> bool {
        let t = i.tick;
        let mut paid = vec![];
        let mut previous = None;
        let mut ticks = PREPARE_TICKS as usize;
        match &a {
            Action::Rotate { object, heading } => {
                if self.form != Form::Herobrine
                    || !i.unlocked[RECIPES[42].slot]
                    || t < self.cooldowns[42]
                {
                    return self.reject("Creative Edit locked or cooling down");
                }
                if let Err(why) = self.rotation(i, *object, *heading) {
                    return self.reject(why);
                }
                let Some(p) = self.spend(t, 1, false) else {
                    return self.reject("Creative Edit needs a charge");
                };
                paid = p;
                previous = Some((42, self.cooldowns[42]));
                self.cooldowns[42] = t + cd(RECIPES[42].cooldown, i.haste);
            }
            Action::Repair(id) => {
                if self.form != Form::Steve {
                    return self.reject("Repair requires Survival Steve");
                }
                if !i.unlocked[0] || t < self.repair_at {
                    return self.reject("Repair locked or cooling down");
                }
                if !self.objects.iter().any(|o| {
                    o.id == *id
                        && o.confirmed
                        && o.hp > 0
                        && o.hp < o.max_hp
                        && within(i.me.pos(), o.point, 40000)
                }) {
                    return self.reject("No damaged owned construction in range");
                }
                let Some(p) = self.spend(t, 1, false) else {
                    return self.reject("Repair needs a charge");
                };
                paid = p;
                self.repair_at = t + 480;
            }
            Action::Recipe {
                recipe: r,
                target,
                point,
            } => {
                if self.form != Form::Herobrine {
                    return self.reject("Creative recipe requires Herobrine");
                }
                let d = RECIPES[*r as usize];
                if !i.unlocked[d.slot] || t < self.cooldowns[d.group] {
                    return self.reject("Recipe locked or cooling down");
                }
                if let Err(why) = self.creative_valid(i, *r, *target, *point, i.me.pos()) {
                    return self.reject(why);
                }
                let Some(p) = self.spend(t, d.cost, false) else {
                    return self.reject("Insufficient Creative charges");
                };
                paid = p;
                previous = Some((d.group, self.cooldowns[d.group]));
                self.cooldowns[d.group] = t + cd(d.cooldown, i.haste);
                if (15..=19).contains(r) {
                    ticks = 60;
                }
            }
            Action::Load {
                node,
                recipe,
                target,
                point,
            } => {
                if self.form != Form::Herobrine {
                    return self.reject("Loading redstone requires Herobrine");
                }
                let Some(n) = self
                    .objects
                    .iter()
                    .find(|o| o.id == *node && o.recipe == 34 && o.confirmed)
                else {
                    return self.reject("Select an owned Dispenser");
                };
                if self.creative.payloads.iter().any(|p| p.node == *node) {
                    return self.reject("Dispenser already loaded");
                }
                if !matches!(*recipe,3..=5|12|20..=30|43..=44) {
                    return self.reject("Unsupported dispenser payload");
                }
                let d = RECIPES[*recipe as usize];
                if !i.unlocked[d.slot] || t < self.cooldowns[d.group] {
                    return self.reject("Payload locked or cooling down");
                }
                if *recipe >= 15 {
                    if let Err(why) = self.creative_valid(i, *recipe, *target, *point, n.point) {
                        return self.reject(why);
                    }
                } else if !legal(*point)
                    || !within(n.point, *point, d.range)
                    || !clear(n.point, *point, i.wall)
                {
                    return self.reject("Payload destination unavailable");
                }
                let Some(p) = self.spend(t, d.cost, false) else {
                    return self.reject("Payload needs its paid recipe charges");
                };
                paid = p;
                previous = Some((d.group, self.cooldowns[d.group]));
                self.cooldowns[d.group] = t + cd(d.cooldown, i.haste);
            }
            Action::Link { from, to } => {
                if self.form != Form::Herobrine {
                    return self.reject("Redstone editing requires Herobrine");
                }
                let nodes: Vec<_> = self
                    .objects
                    .iter()
                    .filter(|o| {
                        [from, to].contains(&&o.id) && o.confirmed && network_recipe(o.recipe)
                    })
                    .collect();
                if from == to || nodes.len() != 2 || !within(nodes[0].point, nodes[1].point, 32000)
                {
                    return self.reject("Link needs two owned components within 32000");
                }
                if self
                    .creative
                    .links
                    .get(from)
                    .is_some_and(|v| v.len() >= 2 || v.contains(to))
                    || self.network_size(*from, *to) > 4
                    || self.cycle(*from, *to)
                {
                    return self.reject("Redstone link is duplicate, cyclic or over capacity");
                }
            }
            Action::Configure { node, mode, delay } => {
                if self.form != Form::Herobrine
                    || *mode > 1
                    || ![15, 30, 60].contains(delay)
                    || !self
                        .objects
                        .iter()
                        .any(|o| o.id == *node && o.confirmed && matches!(o.recipe, 35 | 36))
                {
                    return self.reject("Configure an owned plate or repeater with a legal delay");
                }
            }
            Action::Arm(node) | Action::Activate(node) => {
                if self.form != Form::Herobrine
                    || !self.objects.iter().any(|o| {
                        o.id == *node
                            && o.confirmed
                            && network_recipe(o.recipe)
                            && o.blast.is_none()
                    })
                {
                    return self.reject("Needs an owned available redstone component");
                }
            }
            _ => return self.reject("Unknown Creative action"),
        }
        self.pending = Some(Pending {
            action: a.clone(),
            begin: t,
            end: t + ticks,
            bonus: false,
            charges: paid,
            previous_cd: previous,
            path: None,
            power_hp: i.me.max_hp,
        });
        self.selected = Some(a);
        self.reason = "Preparing Creative command";
        out.push(Cmd::Lock { ticks, pose: 0 });
        true
    }
    fn effect(
        &mut self,
        i: &Input,
        target: usize,
        kind: u8,
        power: i64,
        ticks: usize,
        period: usize,
        out: &mut Vec<Cmd>,
    ) {
        self.creative.effects.retain(|e| e.end >= i.tick);
        if matches!(kind, REGEN | POISON) {
            if let Some(e) = self
                .creative
                .effects
                .iter_mut()
                .find(|e| e.target == target && e.kind == kind)
            {
                if power * e.interval as i64 >= e.power * period as i64 {
                    e.power = power;
                    e.end = i.tick + ticks;
                    e.interval = period;
                } else {
                    return;
                }
            } else {
                self.creative.effects.push(Effect {
                    target,
                    kind,
                    power,
                    end: i.tick + ticks,
                    next: i.tick + period,
                    interval: period,
                });
            }
        }
        out.push(Cmd::Status {
            target,
            kind,
            power: if period > 0 {
                power * 60 / period as i64
            } else {
                power
            },
            ticks: if period > 0 { ticks + 1 } else { ticks },
        });
    }
    fn armour(&mut self, i: &Input, target: usize, r: u8, out: &mut Vec<Cmd>) {
        let Some(u) = ally(i, Some(target)) else {
            return;
        };
        for kind in [PROTECTION, THORNS] {
            out.push(Cmd::ClearStatus { target, kind });
        }
        self.creative.armour.retain(|a| a.target != target);
        self.creative.armour.push(Armour {
            target,
            recipe: r,
            end: i.tick + 480,
            enchant: None,
        });
        out.push(Cmd::Status {
            target: u.id,
            kind: IRON + r - 28,
            power: if r == 28 {
                20
            } else if r == 29 {
                30
            } else {
                20
            },
            ticks: 480,
        });
    }
    fn splash(&mut self, i: &Input, r: u8, p: Point, hp: i64, out: &mut Vec<Cmd>) {
        let targets: Vec<_> = i
            .units
            .iter()
            .filter(|u| {
                u.hp > 0
                    && u.targetable
                    && within(u.pos(), p, 18000)
                    && if r >= 23 && r <= 25 {
                        u.team == i.me.team && u.id != i.me.id && u.champion
                    } else {
                        u.team != i.me.team && u.visible && !u.tower
                    }
            })
            .cloned()
            .collect();
        for u in targets {
            match r {
                20 => self.effect(i, u.id, POISON, hp_hit(&u, 6, hp, 70), 240, 60, out),
                21 => {
                    if !u.cc_immune {
                        self.effect(i, u.id, SLOW, 25, 180, 0, out)
                    }
                }
                22 => self.effect(i, u.id, WEAKNESS, 15, 180, 0, out),
                23 => out.push(Cmd::Heal(u.id, 35 + hp * 3 / 100)),
                24 => self.effect(i, u.id, REGEN, 8 + hp * 70 / 10000, 240, 60, out),
                25 => self.effect(i, u.id, SPEED, 15, 180, 0, out),
                _ => {}
            }
        }
    }
    fn apply_creative(
        &mut self,
        i: &Input,
        r: u8,
        target: Option<usize>,
        q: Point,
        origin: Point,
        hp: i64,
        paid: Vec<usize>,
        old_cd: usize,
        out: &mut Vec<Cmd>,
    ) {
        if r >= 15 {
            if let Err(why) = self.creative_valid(i, r, target, q, origin) {
                self.reason = why;
                self.metrics.wasted += 1;
                return;
            }
        }
        match r {
            3 | 12 | 20..=25 | 43..=44 => {
                let duration =
                    (length(origin, q) / if r == 3 { 6000 } else { 7000 }).max(1) as usize;
                self.shots.push(Projectile {
                    recipe: r,
                    from: origin,
                    point: origin,
                    dest: q,
                    begin: i.tick,
                    end: i.tick + duration,
                    target,
                    hit: vec![],
                    paid,
                    old_cd,
                    bonus: false,
                    travelled: 0,
                    power_hp: hp,
                });
                out.push(Cmd::PathWarning {
                    path: vec![origin, q],
                    until: i.tick + duration,
                });
            }
            4 | 5 | 15..=19 | 26..=27 | 33..=39 => {
                self.object(i, r, q, paid, old_cd, false, hp, out);
            }
            28..=30 => {
                if let Some(u) = ally(i, target) {
                    self.armour(i, u.id, r, out)
                }
            }
            31..=32 => {
                if let Some(u) = ally(i, target) {
                    let own = self
                        .creative
                        .armour
                        .iter_mut()
                        .find(|a| a.target == u.id && a.end > i.tick);
                    let end = if let Some(a) = own {
                        a.enchant = Some(r);
                        a.end
                    } else {
                        u.gifted_armour_until
                    };
                    if end > i.tick {
                        for kind in [PROTECTION, THORNS] {
                            out.push(Cmd::ClearStatus { target: u.id, kind });
                        }
                        out.push(Cmd::Status {
                            target: u.id,
                            kind: if r == 31 { PROTECTION } else { THORNS },
                            power: if r == 31 { 10 } else { 15 },
                            ticks: end - i.tick,
                        });
                    }
                }
            }
            40 => {
                if let Some(u) = ally(i, target) {
                    self.creative.effects.retain(|e| e.target != u.id);
                    for kind in [
                        SPEED, HASTE, STRENGTH, RESISTANCE, POISON, WEAKNESS, REGEN, SLOW,
                    ] {
                        out.push(Cmd::ClearStatus { target: u.id, kind });
                    }
                }
            }
            41 => {
                if let Some(u) = ally(i, target) {
                    self.creative.totem_lock.insert(u.id, i.tick + 3600);
                    self.creative.totems.push(Totem {
                        target: u.id,
                        end: i.tick + 180,
                        shield: 60 + hp * 350 / 10000,
                    });
                    out.push(Cmd::Status {
                        target: u.id,
                        kind: TOTEM,
                        power: 1,
                        ticks: 180,
                    });
                }
            }
            42 => {
                if let Some(o) = self
                    .objects
                    .iter_mut()
                    .find(|o| Some(o.id as usize) == target)
                {
                    let delta = (q.0 - o.point.0, q.1 - o.point.1);
                    o.point = q;
                    let d = length(origin, q).max(1);
                    o.direction = ((q.0 - origin.0) * 10000 / d, (q.1 - origin.1) * 10000 / d);
                    for w in self.walls.iter_mut().filter(|w| w.object == Some(o.id)) {
                        for p in &mut w.path {
                            p.0 += delta.0;
                            p.1 += delta.1;
                        }
                    }
                    out.push(Cmd::RelocateObject(o.id, q));
                }
            }
            _ => {}
        }
        out.push(Cmd::Fx {
            kind: match r {
                20..=27 => 4,
                28..=32 | 40..=41 => 10,
                43..=44 => 12,
                _ => 0,
            },
            point: q,
        });
    }
    pub(super) fn creative_release(&mut self, i: &Input, p: &Pending, out: &mut Vec<Cmd>) -> bool {
        match &p.action {
            Action::Rotate { object, heading } => {
                if let Ok((point, direction)) = self.rotation(i, *object, *heading) {
                    if let Some(o) = self.objects.iter_mut().find(|o| o.id == *object) {
                        o.direction = direction;
                    }
                    for w in self.walls.iter_mut().filter(|w| w.object == Some(*object)) {
                        w.path = vec![
                            (
                                point.0 - direction.0 * 30000 / 10000,
                                point.1 - direction.1 * 30000 / 10000,
                            ),
                            (
                                point.0 + direction.0 * 30000 / 10000,
                                point.1 + direction.1 * 30000 / 10000,
                            ),
                        ];
                    }
                    self.metrics.casts[42] += 1;
                    out.push(Cmd::RelocateObject(*object, point));
                }
            }
            Action::Repair(id) => {
                if let Some(o) = self.objects.iter_mut().find(|o| {
                    o.id == *id && o.hp > 0 && o.confirmed && within(i.me.pos(), o.point, 40000)
                }) {
                    let n = (30 + p.power_hp * 6 / 100).min(o.max_hp - o.hp).max(0);
                    o.hp += n;
                    out.push(Cmd::HealObject(*id, n));
                }
            }
            Action::Recipe {
                recipe: r,
                target,
                point,
            } if *r >= 15 => {
                self.metrics.casts[*r as usize] += 1;
                self.apply_creative(
                    i,
                    *r,
                    *target,
                    *point,
                    i.me.pos(),
                    p.power_hp,
                    p.charges.clone(),
                    p.previous_cd.map_or(0, |v| v.1),
                    out,
                );
            }
            Action::Link { from, to } => {
                let nodes: Vec<_> = self
                    .objects
                    .iter()
                    .filter(|o| o.confirmed && o.hp > 0 && [from, to].contains(&&o.id))
                    .collect();
                if nodes.len() == 2
                    && within(nodes[0].point, nodes[1].point, 32000)
                    && !self.cycle(*from, *to)
                    && self.network_size(*from, *to) <= 4
                    && !self
                        .creative
                        .links
                        .get(from)
                        .is_some_and(|v| v.len() >= 2 || v.contains(to))
                {
                    self.creative.links.entry(*from).or_default().push(*to);
                }
            }
            Action::Configure { node, mode, delay } => {
                self.creative.settings.insert(*node, (*mode, *delay));
            }
            Action::Arm(node) => {
                if let Some(o) = self
                    .objects
                    .iter_mut()
                    .find(|o| o.id == *node && o.confirmed)
                {
                    self.creative.armed.insert(*node);
                    if matches!(o.recipe, 1 | 3) {
                        o.end = i.tick + 720;
                    }
                }
            }
            Action::Activate(node) => {
                self.creative.serial += 1;
                self.creative.pulses.push(Pulse {
                    event: self.creative.serial,
                    node: *node,
                    at: i.tick,
                    path: vec![],
                });
            }
            Action::Load {
                node,
                recipe,
                target,
                point,
            } => {
                if self
                    .objects
                    .iter()
                    .any(|o| o.id == *node && o.confirmed && o.recipe == 34)
                    && !self.creative.payloads.iter().any(|v| v.node == *node)
                {
                    self.creative.payloads.push(Payload {
                        node: *node,
                        recipe: *recipe,
                        target: *target,
                        point: *point,
                        hp: p.power_hp,
                    });
                }
            }
            _ => return false,
        }
        self.reason = "Creative command resolved";
        true
    }
    pub(super) fn creative_object(&mut self, i: &Input, o: &mut Object, out: &mut Vec<Cmd>) {
        match o.recipe {
            15..=19 => {
                let targets: Vec<_> = i
                    .units
                    .iter()
                    .filter(|u| {
                        u.team == i.me.team
                            && u.hp > 0
                            && u.champion
                            && within(u.pos(), o.point, 80000)
                            && (!(18..=19).contains(&o.recipe) || u.id != i.me.id)
                    })
                    .cloned()
                    .collect();
                self.creative.aura_clocks.retain(|(object, target), _| {
                    *object != o.id || targets.iter().any(|u| u.id == *target)
                });
                for u in targets {
                    match o.recipe {
                        15 => self.effect(i, u.id, SPEED, 12, 15, 0, out),
                        16 => self.effect(i, u.id, HASTE, 15, 15, 0, out),
                        17 => self.effect(i, u.id, STRENGTH, 12, 15, 0, out),
                        18 => self.effect(i, u.id, RESISTANCE, 12, 15, 0, out),
                        19 => self.aura_tick(
                            i,
                            o.id,
                            u.id,
                            REGEN,
                            8 + o.power_hp * 70 / 10000,
                            60,
                            out,
                        ),
                        _ => {}
                    }
                }
            }
            26 => {
                let targets: Vec<_> = i
                    .units
                    .iter()
                    .filter(|u| {
                        u.hp > 0
                            && u.team != i.me.team
                            && u.targetable
                            && !u.tower
                            && within(u.pos(), o.point, 22000)
                    })
                    .cloned()
                    .collect();
                self.creative.aura_clocks.retain(|(object, target), _| {
                    *object != o.id || targets.iter().any(|u| u.id == *target)
                });
                for u in targets {
                    self.aura_tick(
                        i,
                        o.id,
                        u.id,
                        POISON,
                        hp_hit(&u, 3, o.power_hp, 35),
                        30,
                        out,
                    );
                }
            }
            27 | 37 => {
                let radius = if o.recipe == 27 { 22000 } else { 20000 };
                let pct = if o.recipe == 27 { 30 } else { 25 };
                for u in i.units.iter().filter(|u| {
                    u.hp > 0
                        && u.team != i.me.team
                        && u.targetable
                        && !u.cc_immune
                        && within(u.pos(), o.point, radius)
                }) {
                    self.effect(i, u.id, SLOW, pct, 15, 0, out);
                }
            }
            38 => {
                let targets: Vec<_> = i
                    .units
                    .iter()
                    .filter(|u| {
                        u.hp > 0
                            && !u.immovable
                            && !u.cc_immune
                            && u.targetable
                            && within(u.pos(), o.point, 20000)
                            && !o.hit.contains(&u.id)
                    })
                    .cloned()
                    .collect();
                for u in targets {
                    let d = length(o.point, u.pos()).max(1);
                    let q = clip(
                        u.pos(),
                        (
                            u.x + (u.x - o.point.0) * 20000 / d,
                            u.y + (u.y - o.point.1) * 20000 / d,
                        ),
                        i.wall,
                    );
                    if u.team == i.me.team {
                        out.push(Cmd::Push(u.id, q, 15));
                    } else {
                        out.push(Cmd::Warning {
                            point: o.point,
                            radius: 20000,
                            until: i.tick + 15,
                            owner_hazard: false,
                        });
                        self.creative.bounces.push(Bounce {
                            object: o.id,
                            target: u.id,
                            at: i.tick + 15,
                            point: o.point,
                            destination: q,
                        });
                    }
                    o.hit.push(u.id);
                }
            }
            _ => {}
        }
    }
    fn aura_tick(
        &mut self,
        i: &Input,
        object: u32,
        target: usize,
        kind: u8,
        power: i64,
        period: usize,
        out: &mut Vec<Cmd>,
    ) {
        let score = power * 60 / period as i64;
        if self
            .creative
            .aura_winners
            .get(&(target, kind))
            .is_none_or(|(id, _)| *id != object)
            || self.creative.effects.iter().any(|e| {
                e.target == target
                    && e.kind == kind
                    && e.end >= i.tick
                    && e.power * 60 / e.interval.max(1) as i64 >= score
            })
        {
            return;
        }
        let last = self
            .creative
            .aura_clocks
            .entry((object, target))
            .or_insert(i.tick);
        out.push(Cmd::Status {
            target,
            kind,
            power: power * 60 / period as i64,
            ticks: 16,
        });
        if i.tick >= *last + period {
            *last = i.tick;
            self.serial += 1;
            out.push(Cmd::PotionTick {
                event: self.serial,
                target,
                power,
                kind,
            });
        }
    }
    pub(super) fn creative_shot(
        &mut self,
        i: &Input,
        s: &mut Projectile,
        out: &mut Vec<Cmd>,
    ) -> bool {
        if !matches!(s.recipe,20..=25|43..=44) {
            return false;
        }
        let r = s.recipe;
        let age = i.tick.saturating_sub(s.begin);
        let duration = (s.end - s.begin).max(1);
        let to = (
            s.from.0 + (s.dest.0 - s.from.0) * age.min(duration) as i64 / duration as i64,
            s.from.1 + (s.dest.1 - s.from.1) * age.min(duration) as i64 / duration as i64,
        );
        if !clear(s.point, to, i.wall) {
            s.end = i.tick;
            return true;
        }
        if r >= 43 {
            let hit = i
                .units
                .iter()
                .filter(|u| {
                    u.team != i.me.team
                        && u.hp > 0
                        && u.visible
                        && u.targetable
                        && !u.tower
                        && distance_segment(u.pos(), s.point, to) <= 8000
                })
                .min_by_key(|u| (square(s.point, u.pos()), u.id))
                .cloned();
            if let Some(u) = hit {
                self.damage(i, &u, hp_hit(&u, 30, s.power_hp, 400), 0, false, out);
                if r == 43 {
                    self.effect(i, u.id, POISON, hp_hit(&u, 6, s.power_hp, 70), 120, 60, out)
                } else if !u.cc_immune {
                    self.effect(i, u.id, SLOW, 25, 120, 0, out)
                }
                s.end = i.tick;
            }
        } else if i.tick >= s.end {
            self.splash(i, r, s.dest, s.power_hp, out);
        }
        s.point = to;
        true
    }
    pub(super) fn creative_step(&mut self, i: &Input, out: &mut Vec<Cmd>) {
        let live: BTreeSet<_> = self
            .objects
            .iter()
            .filter(|o| o.confirmed && o.hp > 0)
            .map(|o| o.id)
            .collect();
        self.creative
            .aura_clocks
            .retain(|(object, _), _| live.contains(object));
        let bounces = std::mem::take(&mut self.creative.bounces);
        for b in bounces {
            if !live.contains(&b.object) {
                continue;
            }
            if i.tick < b.at {
                self.creative.bounces.push(b);
                continue;
            }
            if let Some(u) = i.units.iter().find(|u| {
                u.id == b.target
                    && u.hp > 0
                    && u.targetable
                    && !u.cc_immune
                    && !u.immovable
                    && within(u.pos(), b.point, 20000)
            }) {
                out.push(Cmd::Push(u.id, clip(u.pos(), b.destination, i.wall), 15));
            }
        }
        self.creative.links.retain(|id, bs| {
            bs.retain(|b| live.contains(b));
            live.contains(id)
        });
        self.creative.payloads.retain(|p| live.contains(&p.node));
        self.creative.armed.retain(|id| live.contains(id));
        let plate: Vec<_> = self
            .objects
            .iter()
            .filter(|o| o.recipe == 35 && self.creative.armed.contains(&o.id))
            .filter(|o| {
                let ally = self.creative.settings.get(&o.id).is_some_and(|v| v.0 == 1);
                i.units.iter().any(|u| {
                    u.hp > 0
                        && u.targetable
                        && (u.team == i.me.team) == ally
                        && within(u.pos(), o.point, 12000)
                })
            })
            .map(|o| o.id)
            .collect();
        for node in plate {
            self.creative.armed.remove(&node);
            self.creative.serial += 1;
            self.creative.pulses.push(Pulse {
                event: self.creative.serial,
                node,
                at: i.tick,
                path: vec![],
            });
        }
        let pulses = std::mem::take(&mut self.creative.pulses);
        for mut pulse in pulses {
            if !live.contains(&pulse.node) || pulse.path.iter().any(|n| !live.contains(n)) {
                continue;
            }
            if pulse.at > i.tick {
                self.creative.pulses.push(pulse);
                continue;
            }
            if !self.creative.seen.insert((pulse.event, pulse.node)) {
                continue;
            }
            pulse.path.push(pulse.node);
            let Some(o) = self.objects.iter().find(|o| o.id == pulse.node).cloned() else {
                continue;
            };
            match o.recipe {
                3 => {
                    if let Some(real) = self.objects.iter_mut().find(|v| v.id == o.id) {
                        if real.blast.is_none() {
                            real.blast = Some(i.tick + 36);
                            out.push(Cmd::Warning {
                                point: o.point,
                                radius: 32000,
                                until: i.tick + 36,
                                owner_hazard: false,
                            });
                        }
                    }
                }
                1 | 33 => {
                    let end = (
                        o.point.0 + o.direction.0 * 24000 / 10000,
                        o.point.1 + o.direction.1 * 24000 / 10000,
                    );
                    out.push(Cmd::PathWarning {
                        path: vec![o.point, end],
                        until: i.tick + 12,
                    });
                    self.creative.settings.insert(o.id, (2, i.tick + 12));
                }
                34 => {
                    if let Some(n) = self.creative.payloads.iter().position(|p| p.node == o.id) {
                        let p = self.creative.payloads.remove(n);
                        self.apply_creative(
                            i,
                            p.recipe,
                            p.target,
                            p.point,
                            o.point,
                            p.hp,
                            vec![],
                            0,
                            out,
                        );
                    }
                }
                _ => {}
            }
            let delay = if o.recipe == 36 {
                self.creative.settings.get(&o.id).map_or(15, |v| v.1)
            } else {
                1
            };
            if let Some(next) = self.creative.links.get(&o.id).cloned() {
                for node in next {
                    self.creative.pulses.push(Pulse {
                        event: pulse.event,
                        node,
                        at: i.tick + delay,
                        path: pulse.path.clone(),
                    });
                }
            }
        }
        let pistons: Vec<_> = self
            .creative
            .settings
            .iter()
            .filter(|(_, v)| v.0 == 2 && i.tick >= v.1)
            .map(|(id, _)| *id)
            .collect();
        for id in pistons {
            self.creative.settings.remove(&id);
            if let Some(o) = self.objects.iter().find(|o| o.id == id).cloned() {
                for u in i.units.iter().filter(|u| {
                    u.hp > 0
                        && u.team != i.me.team
                        && !u.cc_immune
                        && !u.immovable
                        && distance_segment(
                            u.pos(),
                            o.point,
                            (
                                o.point.0 + o.direction.0 * 24000 / 10000,
                                o.point.1 + o.direction.1 * 24000 / 10000,
                            ),
                        ) < 10000
                }) {
                    let q = if o.recipe == 33 {
                        toward(u.pos(), o.point, 14000)
                    } else {
                        (
                            u.x + o.direction.0 * 18000 / 10000,
                            u.y + o.direction.1 * 18000 / 10000,
                        )
                    };
                    out.push(Cmd::Push(u.id, clip(u.pos(), q, i.wall), 12));
                }
            }
        }
        let mut effects = std::mem::take(&mut self.creative.effects);
        for mut e in effects.drain(..) {
            if i.tick > e.end {
                continue;
            }
            if self.creative.aura_restores.contains(&(e.target, e.kind))
                && self
                    .creative
                    .aura_winners
                    .get(&(e.target, e.kind))
                    .is_none_or(|(_, score)| *score <= e.power * 60 / e.interval.max(1) as i64)
            {
                out.push(Cmd::Status {
                    target: e.target,
                    kind: e.kind,
                    power: e.power * 60 / e.interval.max(1) as i64,
                    ticks: e.end - i.tick + 1,
                });
            }
            if i.tick >= e.next {
                if i.units.iter().any(|u| u.id == e.target && u.hp > 0)
                    && self
                        .creative
                        .aura_winners
                        .get(&(e.target, e.kind))
                        .is_none_or(|(_, score)| *score <= e.power * 60 / e.interval.max(1) as i64)
                {
                    self.serial += 1;
                    out.push(Cmd::PotionTick {
                        event: self.serial,
                        target: e.target,
                        power: e.power,
                        kind: e.kind,
                    });
                }
                e.next = i.tick + e.interval;
            }
            self.creative.effects.push(e);
        }
        self.creative.aura_restores.clear();
        self.creative.armour.retain(|a| a.end > i.tick);
        let mut totems = std::mem::take(&mut self.creative.totems);
        for t in totems.drain(..) {
            if let Some(u) = ally(i, Some(t.target)).filter(|u| u.hp <= 1 && i.tick < t.end) {
                out.push(Cmd::ClearStatus {
                    target: u.id,
                    kind: TOTEM,
                });
                out.push(Cmd::Shield(u.id, t.shield, 120));
                self.effect(i, u.id, REGEN, u.max_hp * 5 / 100, 241, 60, out);
            } else if i.tick < t.end {
                self.creative.totems.push(t);
            }
        }
        self.creative
            .seen
            .retain(|(id, _)| id.saturating_add(128) >= self.creative.serial);
    }
}
