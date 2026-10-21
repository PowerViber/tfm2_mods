//! Delivery cuts are applied once; stored shield payloads remain unscaled.
use crate::balance_data::*;
pub fn shield(amount: usize) -> usize {
    amount.saturating_mul(CODER_SHIELD_PERCENT) / 100
}
pub fn knife(attack: usize, farming: bool) -> usize {
    let raw = if farming {
        30 + attack * 60 / 100
    } else {
        40 + attack * 75 / 100
    };
    raw.saturating_mul(DIO_KNIFE_PERCENT) / 100
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn knife_cut_rounds_after_the_existing_formula() {
        for attack in 0..500 {
            assert_eq!(knife(attack, false), (40 + attack * 75 / 100) * 75 / 100);
            assert_eq!(knife(attack, true), (30 + attack * 60 / 100) * 75 / 100);
        }
    }
    #[test]
    fn replicated_shields_scale_the_unscaled_payload_once() {
        assert_eq!(shield(200), 150);
        assert_eq!(shield(200 * 70 / 100), 105);
        assert_ne!(shield(200 * 70 / 100), shield(shield(200) * 70 / 100));
    }
}
