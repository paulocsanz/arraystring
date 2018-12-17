# ArrayString

A stack based strings with a maximum (customizable) size.

**Never panics (all panic branches are impossible and therefore removed at compile time)**

**Never allocates memory on the heap**

## Why

Data is generally bounded, you don't want a phone number with 30 characters, nor a username with 100. You probably don't even support it in your database.

Why pay the cost of heap allocations of strings with unlimited capacity if you have limited boundaries?

Array based strings always occupy the full space in memory, so they may use more size than dynamic strings.

Array based strings are generally faster to create, clone and append than heap based strings (custom allocators and thread-locals may help with heap based ones).

There are other stack based strings out there, they generally can grow (heap allocate), but the stack based size is defined by the library implementor, we go through a different route (fully stack based with customizable maximum size - per type)

ArrayStrings types are created through a macro with customizable maximum size (implementing the appropriate traits)

```rust
// Always occupies 21 bytes of memory (in the stack)
//
// String's current (2018) implementation always uses 24 bytes + up to 20 bytes (actual username)
//   - Assuming 64 bit usize
//
// Remember that UTF-8 characters can use up to 4 bytes
impl_string(struct Username(20));
```

**TODO: bench against other implementations**

## Features

**default:** `std`

- `std` enabled by default, enables `std` compatibility (remove it to be `#[no_std]` compatible)
- `serde-traits` enables serde traits integration (`Serialize`/`Deserialize`)
- `diesel-traits` enables diesel traits integration (opperates like `String`)
- `logs` enables internal logging (you probably don't need it)

## Examples

```rust
extern crate arraystring;
#[macro_use]
use arraystring::{Error, prelude::*};

impl_string!(pub struct Username(20));
impl_string!(pub struct Role(5));

#[derive(Debug)]
pub struct User {
    pub username: Username,
    pub role: Role,
}

fn main() -> Result<(), Error> {
    let user = User {
        username: Username::from_str("user")?,
        role: Role::from_str("admin")?
    };
    println!("{:?}", user);
}
```

## Roadmap

- Never panics (panic branches are removed at compile time)

## Licenses

[MIT](master/license/MIT) and [Apache-2.0](master/license/APACHE)
