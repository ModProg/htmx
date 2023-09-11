//! ```cargo
//! [dependencies]
//! axum = "0.6.20"
//! ```
use std::collections::HashMap;
use std::error::Error;

use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Form, Router};
use htmx::{htmx, HtmxSrc};

async fn index() -> impl IntoResponse {
    htmx! {
        <head>
            <HtmxSrc/>
        </head>
        <h1>"Axum Demo"</h1>
        <form hx::post="/greet" hx::swap="outerHTML">
            <input name="name" placeholder="Name"/>
            <button> "Greet me" </button>
        </form>
    }
}

async fn greet(Form(form): Form<HashMap<String, String>>) -> impl IntoResponse {
    htmx! {
        "Hello"
        {form.get("name").map(|name| format!("{name}!"))}
        <a href="/"> ":D" </a>
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("http://localhost:8080");
    axum::Server::bind(&"127.0.0.1:8080".parse()?)
        .serve(
            Router::new()
                .route("/", get(index))
                .route("/greet", post(greet))
                .into_make_service(),
        )
        .await
        .map_err(Into::into)
}
