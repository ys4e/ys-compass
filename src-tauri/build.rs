fn main() {
    // Check for migrations.
    println!("cargo:rerun-if-changed=migrations");

    tauri_build::build()
}
