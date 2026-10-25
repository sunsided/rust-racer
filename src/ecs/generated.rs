//! The code sillyecs generates from `ecs.yaml` (see `build.rs`), in its own module so the lint
//! exemptions below don't extend to the hand-written components, states and systems.

#![allow(
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    mismatched_lifetime_syntaxes,
    elided_lifetimes_in_paths,
    unused_qualifications,
    dead_code
)]
#![allow(
    clippy::missing_safety_doc,
    clippy::new_without_default,
    clippy::nonminimal_bool,
    clippy::excessive_precision,
    clippy::len_without_is_empty,
    clippy::extra_unused_type_parameters,
    rustdoc::broken_intra_doc_links,
    rustdoc::redundant_explicit_links
)]

// The generated code refers to the component data, states and system data by name.
use super::*;

// The generated world stores entity locations in a map type it expects to find by this name.
type EntityLocationMap<K, V> = std::collections::HashMap<K, V>;

include!(concat!(env!("OUT_DIR"), "/components_gen.rs"));
include!(concat!(env!("OUT_DIR"), "/archetypes_gen.rs"));
include!(concat!(env!("OUT_DIR"), "/systems_gen.rs"));
include!(concat!(env!("OUT_DIR"), "/world_gen.rs"));
