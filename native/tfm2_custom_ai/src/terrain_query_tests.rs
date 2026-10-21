#[cfg(test)]
mod area_tests {
    use super::*;
    use std::time::Instant;
    fn old(x: i64, y: i64, r: i64) -> Vec<(i64, i64)> {
        let mut out = vec![];
        for cy in 0..N {
            for cx in 0..N {
                let (bx, by) = (cx * CELL + CELL / 2, cy * CELL + CELL / 2);
                let d = (bx - x) as i128 * (bx - x) as i128 + (by - y) as i128 * (by - y) as i128;
                if d <= (r as i128) * (r as i128) && bush_at(bx, by) && !wall_at(bx, by) {
                    out.push((d, (bx, by)));
                }
            }
        }
        out.sort();
        out.into_iter().map(|r| r.1).collect()
    }
    fn fixture(f: impl FnOnce()) {
        let _lock = TEST_LOCK.lock().unwrap();
        let walls = GRID.read().unwrap().clone();
        let bushes = BUSH.read().unwrap().clone();
        set((0..N * N).map(|i| i % 13 == 0).collect());
        set_bushes((0..N * N).map(|i| i % 4 == 0).collect());
        f();
        *GRID.write().unwrap() = walls;
        *BUSH.write().unwrap() = bushes;
        VERSION.fetch_add(1, std::sync::atomic::Ordering::Release);
    }
    #[test]
    fn batching_preserves_bush_locations_order_edges_and_missing_cells() {
        fixture(|| {
            for x in [-1000, 0, 32000, 480000, 959999, 960000] {
                for y in [0, 123000, 959999] {
                    for r in [0, 30000, 150000, 1000000] {
                        assert_eq!(old(x, y, r), bushes_near(x, y, r));
                    }
                }
            }
        });
    }
    #[test]
    #[ignore = "Explicit five-run performance comparison"]
    fn terrain_benchmark() {
        fixture(|| {
            let mut times = [vec![], vec![]];
            for i in 0..1000 {
                for k in [i % 2, 1 - i % 2] {
                    let at = Instant::now();
                    let result = if k == 0 {
                        old(480000, 480000, 1000000)
                    } else {
                        bushes_near(480000, 480000, 1000000)
                    };
                    std::hint::black_box(result);
                    times[k].push(at.elapsed().as_nanos() as u64);
                }
            }
            for a in &mut times {
                a.sort_unstable();
            }
            let row = |k: usize| serde_json::json!({"medianNs":times[k][500],"p95Ns":times[k][950],"p99Ns":times[k][990]});
            println!(
                "BUSH_BENCH {}",
                serde_json::json!({"samples":1000,"baseline":row(0),"candidate":row(1)})
            );
        });
    }
}
