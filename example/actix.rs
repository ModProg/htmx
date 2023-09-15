//! ```cargo
//! [dependencies]
//! actix-web = "4.4.0"
//! ```
use std::collections::HashMap;

use actix_web::web::Form;
use actix_web::{get, post, App, HttpServer, Responder};
use htmx::{htmx, HtmxSrc};

#[get("/")]
async fn index() -> impl Responder {
    htmx! {
        <head>
            <HtmxSrc/>
            <script>
                fn hello_function() {
                    alert("RUSTY JS!!!!")
                }
            </script>
        </head>
        <h1>"Actix Demo"</h1>
        <form hx::post="/greet" hx::swap="outerHTML">
            <input name="name" placeholder="Name"/>
            <button> "Greet me" </button>
        </form>
        <button onclick="hello_function()"> "Alert me" </button>
    }
}

#[post("/greet")]
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
