#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::time::Duration;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
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

peg::parser! {
  pub grammar c4lang() for str {
    pub rule identifier() -> String
      = n:$([^'0'..='9']['0'..='9'|'a'..='z' | 'A'..='Z']+) {? Ok(n.to_string()) };

    rule quoted_string() -> &'input str = "\"" n:$([^'"']+) "\"" { n }
    rule non_ws() -> &'input str = n:$([^' ']+)

    pub rule string() -> String =
        "\"" n:$([^'"']+) "\"" { n.to_string() }
        / n:$([^' ']+) { n.to_string() }

    pub rule whitespaces() -> &'input str =
        n: $(" "*) { n }

    pub rule name_assignment() -> (String, String)
        = i: identifier() ".name" whitespaces() "=" whitespaces() v: string() {
            (i, v)
        }
    pub rule point_to() -> Option<String> =
        ['-']+ i: identifier() ['-']+ ">" { Option::Some(i) }
        / ['-']+ ">" { Option::None };

    pub rule relationship() -> CArrow = a: identifier() whitespaces() p: point_to() whitespaces() b: identifier() {
            CArrow {
                name: p,
                from: CBox { name: a },
                to: CBox { name: b },
            }
        }
  }
}

#[derive(Debug, PartialEq)]
pub struct CBox {
    name: String,
}
#[derive(Debug, PartialEq)]
pub struct CArrow {
    name: Option<String>,
    from: CBox,
    to: CBox,
}

#[test]
fn test_parser() {
    assert!(c4lang::identifier("11aa2").is_err());
    assert_eq!(c4lang::string(r#""mamamia is a 1002.jjd££4""#), Ok("mamamia is a 1002.jjd££4".to_string()));
    assert_eq!(c4lang::string(r#"mamamia££4"#), Ok("mamamia££4".to_string()));
    let o = CArrow { name: None, from: CBox { name: "cow".to_string() }, to: CBox { name: "fresh".to_string() } };
    assert_eq!(c4lang::relationship(r#"cow -------> fresh"#), Ok(o));
}