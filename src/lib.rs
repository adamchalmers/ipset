//! Quickly check if a given IP is in a set of networks.
//! # Examples
//! ```
//! use ipset::Ipset;
//!
//! let networks = vec![
//!     "10.10.0.0/16".parse().unwrap(),
//!     "11.10.0.0/16".parse().unwrap(),
//! ];
//! let set = Ipset::new(&networks);
//! assert!(set.contains(&"10.10.0.0".parse().unwrap()));
//! assert!(set.contains(&"11.10.0.0".parse().unwrap()));
//! assert!(!set.contains(&"9.10.0.0".parse().unwrap()));
//! assert!(!set.contains(&"12.10.0.0".parse().unwrap()));
//! ```

use bitvec::prelude::*;
pub use ipnetwork;
use ipnetwork::Ipv4Network;
use std::net::Ipv4Addr;

/// Stores a set of networks and can quickly query if a given IP is in the set.
#[derive(Default)]
pub struct Ipset {
    // For each of the 32 bits in an IP, does the set contain a network where
    // that bit is set?
    entries: [Entry; 32],
    // For each of the 32 bits in an IP, does the set contain a network which
    // ends at that bit?
    terminal: BitArr!(for 32),
}

impl Ipset {
    /// Find the union of the given networks.
    pub fn new(networks: &[Ipv4Network]) -> Self {
        let mut this = Self::default();
        for net in networks {
            this.insert(net);
        }
        this
    }

    /// Insert a network into the set.
    pub fn insert(&mut self, net: &Ipv4Network) {
        // Special case, if they specify a CIDR with /0 that means
        // match everything.
        if net.prefix() == 0 {
            self.entries[0] = Entry::Both;
            self.terminal.set(0, true);
            return;
        }

        let num_bits = net.prefix() as usize;
        self.terminal.set(num_bits - 1, true);

        for (o, octet) in net.network().octets().iter().enumerate() {
            for (b, bit) in bits(*octet).iter().enumerate() {
                let i = (o * 8) + b;
                if i == num_bits {
                    return;
                }
                if *bit {
                    Entry::add_one(&mut self.entries[i])
                } else {
                    Entry::add_zero(&mut self.entries[i])
                }
            }
        }
    }

    /// Is the given IP in the set of IP networks?
    pub fn contains(&self, ip: &Ipv4Addr) -> bool {
        let octets = ip.octets();
        let all_bits = octets.iter().map(|b| bits(*b)).flatten();
        for (i, bit) in all_bits.enumerate() {
            if self.terminal[i] {
                return true;
            }
            if !self.entries[i].matches(bit) {
                return false;
            }
        }
        true
    }
}

/// Does the Ipset contain a network with the given bit value at a particular
/// index?
#[derive(Eq, PartialEq, Clone, Copy)]
enum Entry {
    /// Yes, at least one network has a zero at this bit.
    Zero,
    /// Yes, at least one network has a one at this bit.
    One,
    /// Yes, there are networks with zero and others with one at this bit.
    Both,
    /// No.
    Neither,
}

impl Default for Entry {
    fn default() -> Self {
        Self::Neither
    }
}

impl Entry {
    /// A network with a zero at this bit was added to the Ipset.
    #[inline]
    fn add_zero(&mut self) {
        *self = match &self {
            Self::Neither => Self::Zero,
            Self::One => Self::Both,
            other => **other,
        }
    }
    /// A network with a one at this bit was added to the Ipset.
    #[inline]
    fn add_one(&mut self) {
        *self = match &self {
            Self::Neither => Self::One,
            Self::Zero => Self::Both,
            other => **other,
        }
    }
    fn matches(&self, b: bool) -> bool {
        matches!(
            (self, b),
            (Self::Both, _) | (Self::One, true) | (Self::Zero, false)
        )
    }
}

/// Convert the byte to an array of bits.
#[inline]
fn bits(byte: u8) -> BitArr!(for 8) {
    let mut array: BitArray = Default::default();
    for i in 0..8 {
        array.set(i as usize, is_bit_set(byte, i));
    }
    array
}

/// Does the byte have a 1 at the given index?
const fn is_bit_set(byte: u8, i: u8) -> bool {
    let has_only_this_bit_set = 1 << i;
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
}
