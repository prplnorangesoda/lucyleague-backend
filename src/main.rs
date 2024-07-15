use std::io;
use std::sync::Mutex;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};

struct StateCounter {
    counter: Mutex<i32>,
}

async fn hello(data: web::Data<StateCounter>) -> String {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    format!("hi! you are #{counter}")
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let counter = web::Data::new(StateCounter {
        counter: Mutex::new(0),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .route("/", web::get().to(hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
