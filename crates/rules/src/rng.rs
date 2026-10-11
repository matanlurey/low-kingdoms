//! Fixed `SplitMix64` stream and unbiased descending Fisher-Yates shuffle.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private authoritative types stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) struct Rng {
    pub(crate) state: u64,
}

impl Rng {
    pub(crate) const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    const fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    // Rejection sampling removes modulo bias. Bounds come from Vec lengths on supported targets.
    pub(crate) fn below(&mut self, bound: usize) -> usize {
        let bound64 = u64::try_from(bound).unwrap_or(u64::MAX);
        let threshold = bound64.wrapping_neg() % bound64;
        loop {
            let value = self.next();
            if value >= threshold {
                return usize::try_from(value % bound64).unwrap_or(0);
            }
        }
    }

    pub(crate) fn shuffle<T>(&mut self, cards: &mut [T]) {
        for end in (1..cards.len()).rev() {
            let selected = self.below(end + 1);
            cards.swap(end, selected);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    #[test]
    fn splitmix64_reference_stream_and_shuffle() {
        let mut rng = Rng::new(0);
        assert_eq!(rng.next(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(rng.next(), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(rng.next(), 0x06c4_5d18_8009_454f);
        let mut cards = [0, 1, 2, 3, 4, 5];
        Rng::new(0).shuffle(&mut cards);
        assert_eq!(cards, [4, 2, 5, 3, 0, 1]);
    }
}
