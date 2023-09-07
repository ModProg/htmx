use std::collections::HashMap;

use actix_web::web::{self, Form};
use actix_web::{get, post, App, HttpServer, Responder};
use htmx::{htmx, HtmxSrc};

#[get("/")]
async fn index() -> impl Responder {
    htmx! {
        <head>
            <HtmxSrc/>
        </head>
        <h1>"Actix Demo"</h1>
        <form data_hx_post="/greet" data_hx_swap="outerHTML">
            <input name="name" placeholder="Name"/>
            <button> "Greet me" </button>
        </form>
    }
}

// #[routes]
#[post("/greet")]
// #[get("/greet/{name}")]
async fn greet(Form(form): Form<HashMap<String, String>>) -> impl Responder {
    htmx! {
        "Hello"
        {form.get("name").map(|name| format!("{name}!"))}
        <a href="/"> ":D" </a>
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("http://localhost:8080");
    HttpServer::new(|| App::new().service(index).service(greet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
