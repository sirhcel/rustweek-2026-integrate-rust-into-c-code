use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

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
