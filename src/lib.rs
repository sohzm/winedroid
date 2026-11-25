#[cfg(all(target_family = "wasm", feature = "jit"))]
compile_error!(
    "the `jit` feature is unavailable on WebAssembly; build with `--no-default-features`"
);

mod abi;
#[cfg(not(target_os = "emscripten"))]
mod activity_runtime;
#[cfg(target_os = "emscripten")]
#[path = "activity_runtime_emscripten.rs"]
mod activity_runtime;
mod android_framework;
mod apk;
mod apk_signature;
mod art_probe;
mod automation;
mod binary_xml;
mod capability_fs;
#[cfg(not(target_os = "emscripten"))]
mod capability_sqlite;
mod cli;
mod config;
mod dex;
#[cfg(all(feature = "jit", not(target_family = "wasm")))]
mod dex3oat;
mod dex_interpreter;
mod dex_run;
mod elf;
mod error;
mod external_storage;
mod frame_clock;
mod fxhash;
mod guest_filesystem;
mod host_bridge;
mod host_window;
mod input;
mod inspect;
mod install;
#[cfg(all(feature = "jit", not(target_family = "wasm")))]
mod jit;
mod launch;
mod layout_render;
mod manifest;
mod method_metrics;
mod missing;
mod native_deps;
mod native_runtime;
mod package_registry;
mod permissions;
mod prefix;
mod render_metrics;
mod resource_table;
mod runtime_check;
mod skia_renderer;
mod trace;
mod tzdata;

pub use error::{Result, WinedroidError};

pub fn run_cli() -> Result<()> {
    cli::run()
}

pub fn report_missing_primitives() {
    missing::report();
}

#[cfg(target_os = "emscripten")]
pub fn launch_web_apk(path: &std::path::Path, width: u32, height: u32) -> Result<()> {
    let display_size = config::DisplaySize::new(width, height)?;
    launch::launch_apk_with_display_size(path, Some(display_size))
}
