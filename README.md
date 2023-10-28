# ruby-marshal-rs
A Rust implementation of Ruby's Marshal module. Currently supports only a part of the format.

## Supported Values
 * Nil
 * False
 * True
 * Fixnum
 * Symbol
 * Symbol Link
 * Object Link
 * Array
 * Object
 * String

## Alternative Implementation (thurgood)
Why not use/improve [`thurgood`](https://docs.rs/thurgood/latest/thurgood/)? 
There are a few reasons in the form of different design choices.


Firstly, `thurgood` makes the assumption that a Ruby string can always be represented as UTF-8, which is not always the case.
However, assuming this is true makes Ruby strings much nicer to interact with, which is fine if you control the marshal data.
I could not make that assumption for my use case as I did not control the marshal data, so my library does not make that assumption.


Secondly, `thurgood` makes the assumption that serialized objects are not cyclic, and as a result, cannot handle cycles correctly/safely.
In addition, it uses some questionable `unsafe` blocks to implement object links (this library currently uses 100% safe code, but that is not guaranteed to always be the case).
While my use case did not directly require cyclic objects, I still wanted the ability to handle cyclic objects safely as I did not control the marshal data.


Thirdly, it focuses closely on creating a `serde_json`-like object model, providing close compatibility between Ruby objects and JSON.
It even provided methods for converting between the two.
In contrast, this library uses an arena to allocate Ruby values, mostly to handle cycles cleanly.
However, interacting with an arena is closer to how one would interact with a DOM;
it is much more difficult to interact with the objects in this library.


If your use case is not impeded by any of the above design choices, you may be better served by `thurgood`; its implementation is far more complete.
For my use case, I was forced to implement this library.

## References
 * https://ruby-doc.org/core-2.4.8/
 * http://jakegoulding.com/blog/2013/01/16/another-dip-into-rubys-marshal-format/

## License
Licensed under either of
 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
