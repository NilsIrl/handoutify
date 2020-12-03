use actix_multipart::Multipart;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use askama_actix::Template;
use futures::StreamExt;
use lopdf::Document;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

async fn index() -> IndexTemplate {
    IndexTemplate {}
}

async fn convert(mut payload: Multipart) -> HttpResponse {
    let mut field = payload.next().await.unwrap().unwrap();
    let mut pdf = Vec::new();
    while let Some(chunk) = field.next().await {
        pdf.extend(chunk.unwrap());
    }
    let mut doc = Document::load_mem(&pdf).unwrap();
    handoutify::handoutify(&mut doc);
    doc.save_to(&mut pdf).unwrap();
    HttpResponse::Ok().content_type("application/pdf").body(pdf)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new().wrap(Logger::default()).service(
            web::resource("/")
                .route(web::get().to(index))
                .route(web::post().to(convert)),
        )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
