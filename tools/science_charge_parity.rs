//! Standalone integer replay consumed by verify_science_charges.cjs.
#[allow(dead_code)]
#[path = "../native/tfm2_custom_ai/src/unified_theory_math.rs"]
mod math;
fn main() {
    let mut bank = math::ChargeBank::default();
    let mut held = None;
    for tick in 0..2400 {
        bank.advance(tick);
        if tick % 17 == 0 {
            if let Some(slot) = bank.reserve(tick) {
                if tick % 51 == 0 {
                    bank.cancel(slot);
                } else {
                    assert!(bank.commit(slot, tick));
                }
            }
        }
        if tick % 47 == 0 {
            held = bank.reserve(tick);
        }
        if tick % 47 == 1 {
            if let Some(slot) = held.take() {
                bank.cancel(slot);
            }
        }
        let c = bank.cells(tick);
        println!("{tick}|{}|{},{},{}", bank.available(), c[0], c[1], c[2]);
    }
}
