//! ```cargo
//! [dependencies]
//! tauri-runtime = "0.14"
//! tauri-runtime-wry = "0.14"
//! url = "2.4.1"
//! ```
use std::borrow::Cow;
use std::error::Error;
use std::fmt::Display;

use htmx::{htmx, Html, HtmxSrc};
use tauri_runtime::http::ResponseBuilder;
use tauri_runtime::webview::{WebviewAttributes, WindowBuilder};
use tauri_runtime::window::PendingWindow;
use tauri_runtime::Runtime;
use tauri_runtime_wry::Wry;
use url::Url;

fn index() -> Html {
    htmx! {
        <head><HtmxSrc/></head>
        <h1>"Tauri Demo"</h1>
        <form hx::get="/greet" hx::swap="outerHTML">
            <input name="name" placeholder="Name"/>
            <button> "Greet me" </button>
        </form>
    }
}

fn greet(name: impl Display) -> Html {
    htmx! {
        "Hello"
        {format!("{name}!")}
        <a href="/"> ":D" </a>
    }
}

fn get_param<'a>(key: &str, url: &'a Url) -> Result<Cow<'a, str>, String> {
    url.query_pairs()
        .find_map(|(k, v)| (k == key).then_some(v))
        .ok_or_else(|| format!("expected `{key}` param"))
}

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = Wry::<()>::new()?;

    let mut window = PendingWindow::new(
        WindowBuilder::new(),
        WebviewAttributes::new(Default::default()),
        "main",
    )?;
    window.url = "app://main".into();
    window.navigation_handler = Some(Box::new(|url| url.scheme() == "app"));
    window.register_uri_scheme_protocol("app", |req| {
        let url: Url = req.uri().parse()?;
        Ok(ResponseBuilder::new()
            .status(200)
            .body(
                match url.path() {
                    "/" => index(),
                    "/greet" => greet(get_param("name", &url)?),
                    path => return Err(format!("Unknown path `{path}`").into()),
                }
                .to_string()
                .into_bytes(),
            )
            .unwrap())
    });

    runtime.create_window(window)?;

    runtime.run(|_| {});
    Ok(())
}
