# Apple HFS+ NFD-like Unicode Normalization Library for Rust

![CI (master)](<https://github.com/tats-u/rust-hfs-nfd/workflows/CI%20(master)/badge.svg>)
![CI (Release)](<https://github.com/tats-u/rust-hfs-nfd/workflows/CI%20(Release)/badge.svg>)
[![hfs_nfd at crates.io](https://img.shields.io/crates/v/hfs_nfd.svg)](https://crates.io/crates/hfs_nfd)
[![hfs_nfd at docs.rs](https://docs.rs/hfs_nfd/badge.svg)](https://docs.rs/hfs_nfd)

HFS+, the file system formerly used in Apple macOS, uses a unique Unicode normalization similar to NFD.

- https://developer.apple.com/library/archive/technotes/tn/tn1150table.html
- https://developer.apple.com/library/archive/technotes/tn/tn1150.html

This library composes or decomposes Unicode code points according to the normalization. e.g.

- Université[`U+00E9`] de Paris (Common) ⇔ Université[`U+0065 U+0301`] de Paris (HFS+)
- アップ[`U+30D7`]ル (Common) ⇔ アップ[`U+30D5 U+309A`]ル (HFS+)

# How to use

Add this library `hfs_nfd` to your `Cargo.toml`.

```toml
[dependencies]
another_library1 = "<version>"
another_library2 = "<version>"
# *snip*
hfs_nfd = "1.0.0" # <= Here
# *snip*
```

Then, use these functions:

```rust
use hfs_nfd::{compose_from_hfs_nfd,decompose_into_hfs_nfd}

assert_eq!(decompose_into_hfs_nfd("Universit\u{00E9}"), "Universite\u{0301}".to_string());
assert_eq!(compose_from_hfs_nfd("アッフ\u{309A}ル"), "アッ\{30D7}ル".to_string());
```
