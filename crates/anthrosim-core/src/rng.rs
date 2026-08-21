use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Creates deterministic named random streams derived from one master seed.
///
/// The derivation algorithm is deliberately owned by AnthroSim rather than by
/// a hash map or platform hasher whose output may change between versions.
#[derive(Debug, Clone, Copy)]
pub struct RngFactory {
    master_seed: u64,
}

impl RngFactory {
    #[must_use]
    pub const fn new(master_seed: u64) -> Self {
        Self { master_seed }
    }

    #[must_use]
    pub fn stream(self, name: &str) -> ChaCha8Rng {
        ChaCha8Rng::from_seed(derive_seed(self.master_seed, name.as_bytes()))
    }
}

fn derive_seed(master_seed: u64, label: &[u8]) -> [u8; 32] {
    // FNV-1a folds the stable byte label into the master seed. SplitMix64 then
    // expands the mixed value into the 256-bit ChaCha seed. This is not a
    // password/KDF construction; it is a stable stream-separation mechanism.
    let mut state = master_seed ^ 0xcbf2_9ce4_8422_2325;
    for &byte in label {
        state ^= u64::from(byte);
        state = state.wrapping_mul(0x0000_0100_0000_01b3);
    }

    let mut seed = [0_u8; 32];
    for chunk in seed.chunks_exact_mut(8) {
        state = splitmix64(state);
        chunk.copy_from_slice(&state.to_le_bytes());
    }
    seed
}

const fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = value;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn same_seed_and_stream_are_reproducible() {
        let factory = RngFactory::new(42);
        let mut a = factory.stream("demography");
        let mut b = factory.stream("demography");

        for _ in 0..64 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn named_streams_are_separated() {
        let factory = RngFactory::new(42);
        let mut world = factory.stream("world");
        let mut migration = factory.stream("migration");

        assert_ne!(world.next_u64(), migration.next_u64());
    }
}
