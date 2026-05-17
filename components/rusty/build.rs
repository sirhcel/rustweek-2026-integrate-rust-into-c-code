use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("env var CARGO_MANIFEST_DIR missing");
    let esp_idf_dir = env::var("IDF_PATH").expect("env var IDF_PATH missing");
    let project_dir = env::var("PROJECT_DIR").expect("env var PROJECT_DIR missing");

    let config_dir = project_dir + "/build/config";
    let esp_event_include_dir = esp_idf_dir.clone() + "/components/esp_event/include";
    let esp_wifi_include_dir = esp_idf_dir.clone() + "/components/esp_wifi/include";
    let esp_hw_support_include_dir = esp_idf_dir.clone() + "/components/esp_hw_support/include";

    // Generate Rust bindings for some parts of the ESP-IDF imported by this crate.
    bindgen::builder()
        .use_core()
        .header("bindgen/esp_idf.h")
        // TODO: Generating the bindings for everything included from `esp_idf.h` currently fails at
        // offset checks for `wifi_sta_config_t::sae_h2e_identifier`. Let's sort this out later and
        // generate bindings just for the wantent items.
        .allowlist_item("wifi_auth_mode_t")
        .clang_args([
            "-target", "xtensa-esp32s3-none-elf",
            "-I", &config_dir,
            "-I", &esp_event_include_dir,
            "-I", &esp_hw_support_include_dir,
            "-I", &esp_wifi_include_dir,
        ])
        .generate()
        .expect("generating FFI bindings for ESP-IDF failed")
        .write_to_file("src/sys/esp_idf.rs")
        .expect("writing FFI bindings for ESP-IDF failed");

    // Generate C bindings for FFI items exported from this crate.
    cbindgen::generate(crate_dir).map_or_else(
        // Don't let errors from cbindgen fail the entire build. In case of parsing errors, I would
        // have them sorted out by the Rust compiler for even better pointed out by rust-analyzer.
        // See https://github.com/mozilla/cbindgen/issues/472#issuecomment-831439826.
        |error| match error {
            cbindgen::Error::ParseSyntaxError { .. } => {}
            e => panic!("{e:?}"),
        },
        // It looks like there is currently no way of determining the current target directory from
        // the build script. See https://github.com/rust-lang/cargo/issues/5457. Let's make an
        // educated guess then.
        |bindings| {
            bindings.write_to_file("target/include/rusty.h");
        },
    );
}
