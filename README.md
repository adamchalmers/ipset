# Ipset
Quickly check if a given IP is in a set of networks.

## Performance
On my Macbook, `Ipset::contains` takes 15ns, `Ipset::insert` takes 80ns, and the
`Ipset` type's size is 40 bytes. All operations take constant i.e. O(1) time and
space.

Implementation is inspired by hash array mapped tries, but because IPs only have
two possible symbols and always have a fixed length, we can use a very compact
and specialized representation. Bitwise operations make this very fast.

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
