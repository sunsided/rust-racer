use sillyecs::EcsCode;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=ecs.yaml");
    let file = File::open("ecs.yaml").expect("open ecs.yaml");
    EcsCode::generate(BufReader::new(file))
        .expect("generate ECS")
        .write_files()
        .expect("write ECS files");
    patch_system_bounds();
}

/// sillyecs 0.0.6 forgets the `+` between the `CreateSystem<..>` bounds of
/// `MainWorld::new_with_events` once a world has more than one system (the template's
/// loop counter is reset every iteration), which leaves the generated world unparsable.
fn patch_system_bounds() {
    let path = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("world_gen.rs");
    let code = fs::read_to_string(&path).expect("read world_gen.rs");
    let patched = code.replace(">CreateSystem<", "> + CreateSystem<");
    if patched != code {
        fs::write(&path, patched).expect("write world_gen.rs");
    }
}
