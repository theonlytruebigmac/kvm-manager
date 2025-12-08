fn main() {
    // Link against libvirt using pkg-config
    pkg_config::Config::new()
        .probe("libvirt")
        .expect("Failed to find libvirt library. Please install libvirt-dev.");

    tauri_build::build()
}
