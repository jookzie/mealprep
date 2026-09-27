// The Kotlin side is invoked from Rust only, so the plugin exposes no commands to the webview.
fn main() {
    tauri_plugin::Builder::new(&[])
        .android_path("android")
        .build();
}
