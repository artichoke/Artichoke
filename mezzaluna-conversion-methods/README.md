# mezzaluna-conversion-methods

[![GitHub Actions](https://github.com/artichoke/artichoke/workflows/CI/badge.svg)](https://github.com/artichoke/artichoke/actions)
[![Discord](https://img.shields.io/discord/607683947496734760)](https://discord.gg/QCe2tp2)
[![Twitter](https://img.shields.io/twitter/follow/artichokeruby?label=Follow&style=social)](https://twitter.com/artichokeruby)
<br>
[![Crate](https://img.shields.io/crates/v/mezzaluna-conversion-methods.svg)](https://crates.io/crates/mezzaluna-conversion-methods)
[![API](https://docs.rs/mezzaluna-conversion-methods/badge.svg)](https://docs.rs/mezzaluna-conversion-methods)
[![API trunk](https://img.shields.io/badge/docs-trunk-blue.svg)](https://artichoke.github.io/artichoke/mezzaluna_load_path/)

Ruby implicit conversion vocabulary types.

This crate provides a lookup table for Ruby object conversion methods and their
metadata. It maps method names to their C string equivalents and categorizes
them as either implicit conversions or coercions. This is used when booting an
Artichoke interpreter and for implementing native Ruby object conversion
routines.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
mezzaluna-conversion-methods = "1.0.0"
```

And initialize the implicit conversions like this:

```rust
use intaglio::bytes::SymbolTable;
use mezzaluna_conversion_methods::{ConvMethods, InitError};

fn initialize_conversion_methods() -> Result<(), InitError> {
    let mut symbols = SymbolTable::new();
    let methods = ConvMethods::new();
    let table = methods.get_or_init(&mut symbols)?;
    assert_eq!(table.len(), 12);

    let method = methods.find_method(&mut symbols, "to_int")?;
    assert!(method.is_some());
    Ok(())
}
```

## License

`mezzaluna-conversion-methods` is licensed with the [MIT License](LICENSE) (c)
Ryan Lopopolo.
