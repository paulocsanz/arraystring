# Limited String

A stack based string that has a maximum (customizable) size.

## Why

Data is generally bounded, you don't want a phone number with 30 characters, nor a username with 100. You probably don't even support it in your database.

Why use a heap allocated string with unlimited size if you have limited bounds?

There are other stack based strings out there, they generally do grow, but the stack based size is defined by the library implementor, we go through a different route. 

Limited strings are created through a macro that creates the appropriate structure and implements the appropriate traits to opperate like a string

```rust
// Uses 21 bytes of space
impl_string(struct Username(20));
```

**TODO: bench against other implementations**

## Examples

```rust
extern crate limited_string;
#[macro_use]
use limited_string::{Error, prelude::*}];

impl_string!(pub struct Username(20));
impl_string!(pub struct Role(5));

#[derive(Debug)]
pub struct User {
    pub username: Username,
    pub role: Role,
}

fn main() -> Result<(), Error> {
    let user = User {
        username: LimitedString::from_str("user")?,
        role: LimitedString::from_str("admin")?
    };
    println!("{:?}", user);
}
```

## Roadmap

- `no_std`
- Serde integration
- Diesel integration

## Licenses

[MIT](blob/master/license/MIT) and [Apache-2.0](blob/master/license/APACHE)
