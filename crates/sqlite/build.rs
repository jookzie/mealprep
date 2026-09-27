fn main() {
    // `sqlx::migrate!` embeds the migrations at compile time, so a new file must rebuild.
    println!("cargo:rerun-if-changed=migrations");
}
