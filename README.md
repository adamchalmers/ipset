# Ipset
Quickly check if a given IP is in a set of networks.

## Performance
On my Macbook, `Ipset::contains` takes 15ns, `Ipset::insert` takes 20-80ns, and
the `Ipset` type's size is 40 bytes. All operations take constant i.e. O(1) time
and space.

Implementation is inspired by a trie (prefix trees), but with two IP-specific
optimizations:

1. IPs have a fixed length, we can use a fixed-length representation instead of
a tree or vec. This means no heap allocation is necessary, making this
implementation _fast_.
2. IPs only have two possible "letters", zero and one, so the data structure can
be represented as a bitvec. Both inserts and queries can use bitwise operations,
making this _even faster_.

## Examples
```rust
use ipset::Ipset;
fn main() {
    let networks = vec![
        "10.10.0.0/16".parse().unwrap(),
        "11.10.0.0/16".parse().unwrap(),
    ];
    let set = Ipset::new(&networks);
    assert!(set.contains(&"10.10.0.0".parse().unwrap()));
    assert!(set.contains(&"11.10.0.0".parse().unwrap()));
    assert!(!set.contains(&"9.10.0.0".parse().unwrap()));
    assert!(!set.contains(&"12.10.0.0".parse().unwrap()));
}
```
