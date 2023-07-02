#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::fs;
use std::time::Duration;

mod parse;
mod sugiyama;
use parse::parse_to_graph;
use sugiyama::sugiyama_method;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let contents = fs::read_to_string("src/example.rustpeg").expect("File not found.");

    let mut graph = parse_to_graph(contents.as_str()).unwrap();
    println!("{:?}", graph);

    let layered = sugiyama_method(&mut graph);

    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(code_c4::TemplateApp::new(cc))),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(code_c4::TemplateApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
