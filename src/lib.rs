use ipnetwork::Ipv4Network;
use std::net::Ipv4Addr;

#[derive(Default)]
pub struct Ipset {
    entries: [(Entry, bool); 32],
}

impl Ipset {
    pub fn new(networks: &[Ipv4Network]) -> Self {
        let mut this = Self::default();
        for net in networks {
            this.insert(net);
        }
        this
    }

    pub fn insert(&mut self, net: &Ipv4Network) {
        // Special case, if they specify a CIDR with /0 that means
        // match everything.
        if net.prefix() == 0 {
            self.entries[0] = (Entry::Both, true);
            return;
        }

        let bits = significant_bits(net);
        self.entries[bits.len() - 1].1 = true;
        for (i, bit) in bits.into_iter().enumerate() {
            self.entries[i].0 = if bit {
                Entry::add_one(self.entries[i].0)
            } else {
                Entry::add_zero(self.entries[i].0)
            }
        }
    }

    pub fn contains(&self, ip: &Ipv4Addr) -> bool {
        let octets = ip.octets();
        let all_bits = octets.iter().map(|b| bits(*b)).flatten();
        for (i, bit) in all_bits.enumerate() {
            let (entry, done) = self.entries[i];
            if done {
                return true;
            }
            if !entry.matches(bit) {
                return false;
            }
        }
        true
    }
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum Entry {
    Zero,
    One,
    Both,
    Neither,
}

impl Default for Entry {
    fn default() -> Self {
        Self::Neither
    }
}

impl Entry {
    fn add_zero(self) -> Self {
        match self {
            Self::Neither => Self::Zero,
            Self::One => Self::Both,
            other => other,
        }
    }
    fn add_one(self) -> Self {
        match self {
            Self::Neither => Self::One,
            Self::Zero => Self::Both,
            other => other,
        }
    }
    fn matches(&self, b: bool) -> bool {
        matches!(
            (self, b),
            (Self::Both, _) | (Self::One, true) | (Self::Zero, false)
        )
    }
}

fn significant_bits(net: &Ipv4Network) -> Vec<bool> {
    let num_bits = net.prefix() as usize;
    net.network()
        .octets()
        .iter()
        .map(|b| bits(*b))
        .flatten()
        .take(num_bits)
        .collect()
}

fn bits(byte: u8) -> [bool; 8] {
    let mut array: [bool; 8] = Default::default();
    for i in 0u8..8u8 {
        array[i as usize] = is_bit_set(byte, i)
    }
    array
}

fn is_bit_set(byte: u8, i: u8) -> bool {
    let has_only_this_bit_set = 2_u8.pow(i as u32);
    (byte & has_only_this_bit_set) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_32_cidr() {
        let networks = vec!["10.10.0.32/32".parse().unwrap()];
        let set = Ipset::new(&networks);
        assert!(set.contains(&"10.10.0.32".parse().unwrap()));
        assert!(!set.contains(&"203.10.0.32".parse().unwrap()));
    }

    #[test]
    fn test_partial_cidr() {
        let networks = vec!["10.10.0.32/16".parse().unwrap()];
        let set = Ipset::new(&networks);
        assert!(set.contains(&"10.10.0.0".parse().unwrap()));
        assert!(!set.contains(&"11.0.0.0".parse().unwrap()));
    }

    #[test]
    fn test_multiple() {
        let networks = vec![
            "10.10.0.0/16".parse().unwrap(),
            "11.10.0.0/16".parse().unwrap(),
        ];
        let set = Ipset::new(&networks);
        assert!(!set.contains(&"9.10.0.0".parse().unwrap()));
        assert!(set.contains(&"10.10.0.0".parse().unwrap()));
        assert!(set.contains(&"11.10.0.0".parse().unwrap()));
        assert!(!set.contains(&"12.10.0.0".parse().unwrap()));
    }

    #[test]
    fn test_is_bit_set() {
        let tests = vec![
            (33, 0, true),
            (33, 1, false),
            (33, 2, false),
            (33, 3, false),
            (33, 4, false),
            (33, 5, true),
            (33, 6, false),
            (33, 7, false),
        ];
        for (byte, i, expected) in tests {
            assert_eq!(is_bit_set(byte, i), expected);
        }
    }

    #[test]
    fn test_significant_bits() {
        let net: Ipv4Network = "10.9.0.32/15".parse().unwrap();
        let bits = significant_bits(&net);
        let expected = vec![
            false, true, false, true, false, false, false, false, false, false, false, true, false,
            false, false,
        ];
        assert_eq!(bits.len(), expected.len());
        assert_eq!(bits, expected);
    }

    #[test]
    fn test_bits() {
        let tests = vec![
            (33, [true, false, false, false, false, true, false, false]),
            (10, [false, true, false, true, false, false, false, false]),
            (9, [true, false, false, true, false, false, false, false]),
        ];
        for (i, (byte, expected_bits)) in tests.into_iter().enumerate() {
            let actual_bits = bits(byte);
            assert_eq!(expected_bits, actual_bits, "test {}", i);
        }
    }
}