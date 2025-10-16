# State Validation
[![Crates.io](https://img.shields.io/crates/v/state-validation)](https://crates.io/crates/state-validation)

`state-validation` lets you validate an input for a given state. Then, run an action using the validated output.

For an in-depth guide, see [docs.rs](https://docs.rs/state-validation/latest/state_validation/).

## Soundness Rules
`Validator::try_new` takes ownership of the `state` to disallow consecutive
`Validator::execute` calls because an action is assumed to mutate the `state`.
Since an action is assumed to mutate the `state`, any validators using the same `state`
cannot be created.

It is up to you to make sure the filters properly validate what they promise.

## Limitations
Currently, the amount of filters that can be chained is eight.
The reason for this is because of variadics not being supported as of Rust 2024.
Having no more than eight implementations is arbitrary because having more than eight filters is unlikely.
There is no reason not to implement more in the future, if more than eight filters are required.
