# Ipset
Quickly check if a given IP is in a set of networks.

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
