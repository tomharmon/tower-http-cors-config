# About

The [`tower-http`](https://docs.rs/tower-http/latest/tower_http/) crate is a
wonderful crate, however, it's [`cors`](https://docs.rs/tower-http/latest/tower_http/cors/index.html)
module can be repetitive to configure dynamically as none of the types
implement `Deserialize`. The point of this crate is to provide these 
deserializable config types for others so that users of the crate across
the ecosystem don't need to repeatedly create their own versions of these
types.
