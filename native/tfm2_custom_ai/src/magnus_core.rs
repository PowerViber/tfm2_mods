//! Shared deterministic army implementation; the editor compiles this same source.
include!("magnus_army.rs");
#[cfg(test)]
#[path = "magnus_tests.rs"]
mod tests;
