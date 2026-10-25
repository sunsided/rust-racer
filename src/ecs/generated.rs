//! The code sillyecs generates from `ecs.yaml` (see `build.rs`), in its own module so the lint
//! exemptions below don't extend to the hand-written components, states and systems.

// The generated code makes unchecked slice accesses without `unsafe {}` blocks, has unused
// `NAME` constants, and links docs to items that don't exist (sunsided/sillyecs#65).
#![allow(unsafe_op_in_unsafe_fn, dead_code)]
#![allow(rustdoc::broken_intra_doc_links, rustdoc::redundant_explicit_links)]

// The generated code refers to the component data, states and system data by name.
use super::*;

// The generated world stores entity locations in a map type it expects to find by this name.
type EntityLocationMap<K, V> = std::collections::HashMap<K, V>;

include!(concat!(env!("OUT_DIR"), "/components_gen.rs"));
include!(concat!(env!("OUT_DIR"), "/archetypes_gen.rs"));
include!(concat!(env!("OUT_DIR"), "/systems_gen.rs"));
include!(concat!(env!("OUT_DIR"), "/world_gen.rs"));
