# rad crate for r/w

This crate provides neat ways to eat bytes out of your favorite [readers](https://doc.rust-lang.org/std/io/trait.Read.html) and push bytes into cute [writers](https://doc.rust-lang.org/std/io/trait.Write.html).

This crate has three modules, one for each kind of [endianness](https://en.wikipedia.org/wiki/Endianness): [`be`](https://docs.rs/raad/latest/raad/be/index.html) (big endian), [`le`](https://docs.rs/raad/latest/raad/le/index.html) (little endian), and [`ne`](https://docs.rs/raad/latest/raad/ne/index.html) (native endian— whatever your system is on)

# Examples

Read unsigned 16 bit big-endian integers from a [`Reader`](https://doc.rust-lang.org/std/io/trait.Read.html):

```rust
use raad::be::*; // < note how we specify we want big endian when we import the trait
let mut rdr = &mut &[02, 05, 03, 00][..];
assert_eq!([0x0205, 0x0300], rdr.r::<[u16; 2]>().unwrap());
```

Write unsigned 16 bit little-endian integers to a [`Writer`](https://doc.rust-lang.org/std/io/trait.Write.html):

```rust
use raad::le::*; // and here we specify little endian
let mut wtr = vec![];
wtr.w([0x0205u16, 0x0300]).unwrap();
assert_eq!(wtr, vec![05, 02, 00, 03]);
```

# Why

These helpers can greatly increase the ease of reading numbers and other things from a file/…

See, to read 3 u64s from a reader, you would have to go through all this trouble:

```rust
use std::io::Read;
fn read3(t: &mut impl Read) -> std::io::Result<[u64; 3]> {
    let mut out = [0; 3];
    let mut tmp = [0; 8];
    for elem in &mut out {
        t.read_exact(&mut tmp)?;
        *elem = u64::from_ne_bytes(tmp);
    }
    Ok(out)
}
```

wheras, with this crate, its as simple as

```rust,ignore
use raad::ne::*;
t.read::<[u64; 3]>();
```
