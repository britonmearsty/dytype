pub trait Rng {
    fn next_u32(&mut self) -> u32;

    fn next_below(&mut self, n: u32) -> u32 {
        debug_assert!(n > 0);
        self.next_u32() % n
    }

    fn chance(&mut self, percent: u32) -> bool {
        self.next_below(100) < percent
    }
}

pub struct XorShift(u64);

impl XorShift {
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    pub fn from_time() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Self::new(seed)
    }
}

impl Rng for XorShift {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x & 0xFFFF_FFFF) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_generator_is_deterministic() {
        let mut a = XorShift::new(42);
        let mut b = XorShift::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = XorShift::new(1);
        let mut b = XorShift::new(2);
        let mut same = true;
        for _ in 0..20 {
            same &= a.next_u32() == b.next_u32();
        }
        assert!(!same);
    }

    #[test]
    fn next_below_stays_in_range() {
        let mut rng = XorShift::new(7);
        for _ in 0..500 {
            let value = rng.next_below(17);
            assert!(value < 17);
        }
    }

    #[test]
    fn chance_zero_percent_never_fires() {
        let mut rng = XorShift::new(3);
        let mut hits = 0;
        for _ in 0..1000 {
            if rng.chance(0) {
                hits += 1;
            }
        }
        assert_eq!(hits, 0);
    }

    #[test]
    fn chance_hundred_percent_always_fires() {
        let mut rng = XorShift::new(3);
        let mut hits = 0;
        for _ in 0..100 {
            if rng.chance(100) {
                hits += 1;
            }
        }
        assert_eq!(hits, 100);
    }
}