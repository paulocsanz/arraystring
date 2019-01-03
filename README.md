# ArrayString

Generic-array based string

Since rust doesn't have constant generics yet `typenum` is used to allow for generic arrays (through `generic-array` crate)

Can't outgrow capacity (defined at compile time), always occupies `capacity` `+ 1` bytes of memory

*Doesn't allocate memory on the heap*

## Why

Data is generally bounded, you don't want a phone number with 30 characters, nor a username with 100. You probably don't even support it in your database.

Why pay the cost of heap allocations of strings with unlimited capacity if you have limited boundaries?

Array based strings always occupy the full space in memory, so they may use more memory (in the stack) than dynamic strings.

Stack based strings are generally faster to create, clone and append to than heap based strings (custom allocators and thread-locals may help with heap based ones).

But that becomes less true as you increase the array size, 255 bytes is the maximum we accept (bigger will just wrap) and it's probably already slower than heap based strings of that size (like in `std::string::String`)

There are other stack based strings out there, they generally can have "unlimited" capacity (heap allocate), but the stack based size is defined by the library implementor, we go through a different route by implementing a string based in a generic array.

**TODO: bench against other implementations**

## Features

 **default:** `std`

 - `std` enabled by default, enables `std` compatibility (remove it to be `#[no_std]` compatible)
 - `serde-traits` enables serde traits integration (`Serialize`/`Deserialize`)

     Opperates like `String`, but truncates it if it's bigger than capacity

 - `diesel-traits` enables diesel traits integration (`Insertable`/`Queryable`)

     Opperates like `String`, but truncates it if it's bigger than capacity

 - `logs` enables internal logging

     You will probably only need this if you are debugging this library

 ## Examples

```rust
use arraystring::{Error, ArrayString, typenum::U5, typenum::U20};

type Username = ArrayString<U20>;
type Role = ArrayString<U5>;

#[derive(Debug)]
pub struct User {
    pub username: Username,
    pub role: Role,
}

fn main() -> Result<(), Error> {
    let user = User {
        username: Username::try_from_str("user")?,
        role: Role::try_from_str("admin")?
    };
    println!("{:?}", user);

    Ok(())
}
```

## Licenses

[MIT](master/license/MIT) and [Apache-2.0](master/license/APACHE)
