use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=migrations");

    let entries = fs::read_dir("migrations").expect("migrations directory should exist");
    for entry in entries {
        let path = entry.expect("migration entry should exist").path();
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
