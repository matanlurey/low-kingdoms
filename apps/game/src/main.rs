//! Executable entry point for Low Kingdoms on desktop and WebAssembly.

fn main() -> std::process::ExitCode {
    low_kingdoms_client::app::run()
}
