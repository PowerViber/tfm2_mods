//! Versioned, bounded transport for the same planner used by both native teams.
use crate::game_sense_core::{Development, Ratings};
use crate::strategic_core::*;
#[derive(Default)]
pub struct Lab {
    development: Option<Development>,
}
impl Lab {
    pub fn run(&mut self, a: &[i64]) -> String {
        if a.len() < 20
            || a[0] != 1
            || !(0..=1).contains(&a[1])
            || !(0..=32).contains(&a[4])
            || !(0..=32).contains(&a[5])
            || !(0..=64).contains(&a[6])
            || a.len() != 20 + a[4] as usize * 20 + a[5] as usize * 10 + a[6] as usize * 4
        {
            return "{\"error\":\"Invalid strategic frame\"}".into();
        }
        if a[12..16].iter().any(|v| !(0..=100).contains(v))
            || a[16..20].iter().any(|v| !(0..100).contains(v))
        {
            return "{\"error\":\"Invalid development profile\"}".into();
        }
        if !(0..=3).contains(&a[7])
            || !(0..=3).contains(&a[8])
            || a[9] < 0
            || a[10] < 0
            || !(0..=15).contains(&a[11])
        {
            return "{\"error\":\"Invalid development event\"}".into();
        }
        let mut at = 20;
        let mut actors = vec![];
        for _ in 0..a[4] {
            let v = &a[at..at + 20];
            at += 20;
            if v[0..3].iter().any(|n| *n < 0)
                || v[3..5].iter().any(|n| n.unsigned_abs() > 2000000)
                || v[5..10].iter().any(|n| !(0..=100000000).contains(n))
                || !(0..=4).contains(&v[10])
                || v[14..20].iter().any(|n| !(0..=100).contains(n))
            {
                return "{\"error\":\"Invalid actor\"}".into();
            }
            actors.push(Actor {
                id: v[0] as usize,
                athlete: v[1] as usize,
                team: v[2] as usize,
                point: (v[3], v[4]),
                hp: v[5],
                max_hp: v[6],
                speed: v[7],
                power: v[8],
                range: v[9],
                role: v[10] as u8,
                controlled: v[11] != 0,
                ultimate: v[12] != 0,
                visible: v[13] != 0,
                ratings: Ratings(v[14..18].try_into().unwrap()),
                judgement: v[18],
                ego: v[19],
            });
        }
        let mut targets = vec![];
        for _ in 0..a[5] {
            let v = &a[at..at + 10];
            at += 10;
            if v[..3].iter().any(|n| *n < 0)
                || v[3..5].iter().any(|n| n.unsigned_abs() > 2000000)
                || v[5..8].iter().any(|n| !(0..=100000000).contains(n))
            {
                return "{\"error\":\"Invalid target\"}".into();
            }
            targets.push(Target {
                id: v[0] as usize,
                kind: v[1] as u8,
                team: v[2] as usize,
                point: (v[3], v[4]),
                hp: v[5],
                max_hp: v[6],
                loss: v[7],
                visible: v[8] != 0,
                value: v[9].clamp(0, 1000),
            });
        }
        let rects: Vec<_> = a[at..].chunks_exact(4).collect();
        let blocked = |p: Point, q: Point| {
            (0..=32).any(|n| {
                let (x, y) = (p.0 + (q.0 - p.0) * n / 32, p.1 + (q.1 - p.1) * n / 32);
                rects
                    .iter()
                    .any(|r| x >= r[0] && x <= r[2] && y >= r[1] && y <= r[3])
            })
        };
        let routes = crate::strategic_routes::build(&actors, &targets, &|p| {
            rects
                .iter()
                .any(|r| p.0 >= r[0] && p.0 <= r[2] && p.1 >= r[1] && p.1 <= r[3])
        });
        let view = View {
            routes: Some(&routes),
            team: a[1] as usize,
            actors: &actors,
            targets: &targets,
            home: if a[1] == 0 {
                (96000, 864000)
            } else {
                (864000, 96000)
            },
            recent_advantage: a[2].clamp(-10, 10),
            unseen_risk: a[3].clamp(0, 100),
            blocked: &blocked,
        };
        let d = self.development.get_or_insert_with(|| {
            let mut d = Development::new(Ratings(a[12..16].try_into().unwrap()));
            d.progress = std::array::from_fn(|k| a[16 + k] as u16);
            d
        });
        let learned = match a[7] {
            1 => {
                if (0..4).contains(&a[8]) {
                    d.focus = a[8] as usize;
                }
                false
            }
            2 => d.training(a[9].max(0) as u64),
            3 => d.learn(
                a[9].max(0) as u64,
                &format!("lab-set-{}", a[10]),
                a[11] as u8,
            ),
            _ => false,
        };
        let plans=evaluate(&view).iter().map(|p|format!("{{\"caller\":{},\"goal\":{:?},\"target\":{},\"point\":[{},{}],\"score\":{},\"reason\":{:?},\"jobs\":[{}]}}",p.caller,format!("{:?}",p.goal),p.target,p.point.0,p.point.1,p.score,p.reason,p.jobs.iter().map(|j|format!("{{\"actor\":{},\"job\":{:?},\"target\":{},\"goal\":{:?},\"point\":[{},{}]}}",j.actor,format!("{:?}",j.job),j.target,format!("{:?}",j.goal),j.point.0,j.point.1)).collect::<Vec<_>>().join(","))).collect::<Vec<_>>().join(",");
        format!("{{\"version\":1,\"plans\":[{plans}],\"development\":{{\"ratings\":{:?},\"progress\":{:?},\"focus\":{},\"changed\":{learned},\"saved\":{:?}}}}}",d.ratings.0,d.progress,d.focus,d.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_frames_do_not_award_or_initialize_development() {
        let mut l = Lab::default();
        let mut a = vec![
            1, 0, 0, 0, 1, 0, 0, 2, 0, 1, 1, 15, 50, 50, 50, 50, 0, 0, 0, 0,
        ];
        a.extend([0; 20]);
        a[23] = i64::MIN;
        assert!(l.run(&a).contains("error"));
        assert!(l.development.is_none());
    }
    #[test]
    fn independent_sessions_preserve_deduplication_after_save() {
        let a = [
            1, 0, 0, 0, 0, 0, 0, 2, 0, 20261021, 1, 15, 50, 50, 50, 50, 0, 0, 0, 0,
        ];
        let mut first = Lab::default();
        let mut other = Lab::default();
        assert_eq!(first.run(&a), other.run(&a));
        assert!(first.run(&a).contains("\"changed\":false"));
        let encoded = first.development.as_ref().unwrap().encode();
        let saved = Development::decode(&encoded).unwrap();
        assert_eq!(saved, first.development.unwrap());
    }
    #[test]
    fn extreme_frames_fail_without_panicking() {
        for size in [0, 19, 20, 60, 8193] {
            let mut l = Lab::default();
            let a = vec![i64::MIN; size];
            assert!(l.run(&a).contains("error"));
        }
    }
}
