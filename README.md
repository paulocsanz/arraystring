# ArrayString

String based on generic array

Since rust doesn't have constant generics yet `typenum` is used to allow for generic arrays (U1 to U255)

*If you need bigger than `U255` open an issue explaining your use-case and we may implement*

Can't outgrow initial capacity (defined at compile time), always occupies `capacity` `+ 1` bytes of memory

*Doesn't allocate memory on the heap and never panics in release (all panic branches are stripped at compile time - except `Index`/`IndexMut` traits, since they are supposed to)*

* [Documentation](https://docs.rs/arraystring/0.2.4/arraystring)

## Why

Data is generally bounded, you don't want a phone number with 30 characters, nor a username with 100. You probably don't even support it in your database.

Why pay the cost of heap allocations of strings with unlimited capacity if you have limited boundaries?

Stack based strings are generally faster to create, clone and append to than heap based strings (custom allocators and thread-locals may help with heap based ones).

But that becomes less true as you increase the array size, `CacheString` occuppies a full cache line, 255 bytes is the maximum we accept - `MaxString` (bigger will just wrap) and it's probably already slower than heap based strings of that size (like in `std::string::String`)

There are other stack based strings out there, they generally can have "unlimited" capacity (heap allocate), but the stack based size is defined by the library implementor, we go through a different route by implementing a string based in a generic array.

Array based strings always occupies the full space in memory, so they may use more memory (in the stack) than dynamic strings.

## Features

 **default:** `std`

 - `std` enabled by default, enables `std` compatibility - `impl Error` trait for errors (remove it to be `#[no_std]` compatible)
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

 ## Benchmarks

*This benchmarks ran while I streamed video and used my computer (with* **non-disclosed specs**) *as usual, so don't take the actual times too serious, just focus on the comparison*

```my_custom_benchmark
string                     clone                 25.850 ns
string                     from                  25.815 ns
---------------------------------------------------------
small-string  (21 bytes)   clone                  4.556 ns
small-string  (21 bytes)   try_from_str          15.749 ns
small-string  (21 bytes)   from_str_truncate     10.991 ns
small-string  (21 bytes)   from_str_unchecked    11.195 ns
---------------------------------------------------------
cache-string  (63 bytes)   clone                 10.345 ns
cache-string  (63 bytes)   try_from_str          24.959 ns
cache-string  (63 bytes)   from_str_truncate     17.485 ns
cache-string  (63 bytes)   from_str_unchecked    16.684 ns
---------------------------------------------------------
max-string   (255 bytes)   clone                145.750 ns
max-string   (255 bytes)   try_from_str         157.890 ns
max-string   (255 bytes)   from_str_truncate    193.870 ns
max-string   (255 bytes)   from_str_unchecked   163.740 ns
```

## Licenses

[MIT](master/license/MIT) and [Apache-2.0](master/license/APACHE)
