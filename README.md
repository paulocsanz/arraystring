*For experimentation with nightly only `const generics` features try [`staticvec`](https://github.com/slightlyoutofphase/staticvec/)*

# ArrayString

Fixed capacity stack based generic string

Can't outgrow initial capacity (defined at compile time), always occupies `capacity` `+ 1` bytes of memory

*Maximum Capacity is 255*

*Doesn't allocate memory on the heap and should never panic in release (except in `Index`/`IndexMut` traits, since they are supposed to)*

*The no panic garantee can be ensured at compilation time with the `no-panic` feature, just be aware that a compiler update might break this garantee, therefore making the crate uncompilable, open an issue if you notice.*

* [Documentation](https://docs.rs/arraystring/latest/arraystring)

## Why

Data is generally bounded, you don't want a phone number with 30 characters, nor a username with 100. You probably don't even support it in your database.

Why pay the cost of heap allocations of strings with unlimited capacity if you have limited boundaries?

Stack based strings are generally faster to create, clone and append to than heap based strings (custom allocators and thread-locals may help with heap based ones).

But that becomes less true as you increase the array size, `CacheString` occuppies a full cache line, 255 bytes is the maximum we accept - `MaxString` and it's probably already slower than heap based strings of that size (like in `std::string::String`)

There are other stack based strings out there, they generally can have "unlimited" capacity using small string optimizations, but the stack based size is defined by the library implementor. We go through a different route by implementing a string based in a generic array.

Be aware that array based strings always occupy the full space in memory, so they may use more memory (although in the stack) than dynamic strings.

## Features

 **default:** `std`

 - `std` enabled by default, enables `std` compatibility, implementing std trait (disable it to be `#[no_std]` compatible)
 - `serde-traits` enables serde traits integration (`Serialize`/`Deserialize`)

     Opperates like `String`, but truncates it if it's bigger than capacity

 - `diesel-traits` enables diesel traits integration

     Opperates like `String`, but truncates it if it's bigger than capacity

 - `no-panic` checks at compile time that the panic function is not linked by the library

     Only works when all optimizations are enabled, and may break in future compiler updates. Please open an issue if you notice.

 - `logs` enables internal logging

     You will probably only need this if you are debugging this library

 ## Examples

```rust
use arraystring::{Error, ArrayString};

type Username = ArrayString<20>;
type Role = ArrayString<5>;

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
