use actix_multipart::Multipart;
use actix_web::{http, middleware::Logger, web, App, Either, HttpResponse, HttpServer};
use askama_actix::Template;
use futures::StreamExt;
use lopdf::Document;
use std::env;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    message: Option<&'a str>,
}

async fn index() -> IndexTemplate<'static> {
    IndexTemplate { message: None }
}

type ConvertionResult<'a> = Either<HttpResponse, IndexTemplate<'a>>;

async fn convert(mut payload: Multipart) -> ConvertionResult<'static> {
    let mut prune_objects = false;
    let mut renumber_objects = false;
    let mut download = false;
    let mut pdf = Vec::new();

    while let Some(Ok(mut field)) = payload.next().await {
        match field.content_disposition().unwrap().get_name().unwrap() {
            "download" => {
                download = true;
            }
            "prune-objects" => {
                prune_objects = true;
            }
            "renumber-objects" => {
                renumber_objects = true;
            }
            "file" => {
                while let Some(chunk) = field.next().await {
                    pdf.extend(chunk.unwrap());
                }
            }
            _ => unreachable!(),
        }
    }
    let mut doc = match Document::load_mem(&pdf) {
        Ok(doc) => doc,
        Err(_) => {
            return Either::B(IndexTemplate {
                message: Some("Invalid PDF document."),
            });
        }
    };
    pdf.clear();
    match handoutify::handoutify(&mut doc) {
        Ok(_) => {
            if prune_objects {
                doc.prune_objects();
            }
            if renumber_objects {
                doc.renumber_objects();
            }

            doc.save_to(&mut pdf).unwrap();
            Either::A(
                HttpResponse::Ok()
                    .set_header(
                        http::header::CONTENT_DISPOSITION,
                        if download {
                            "attachment; filename=\"document.pdf\""
                        } else {
                            "inline"
                        },
                    )
                    .content_type("application/pdf")
                    .body(pdf),
            )
        }
        Err(_) => Either::B(IndexTemplate {
            message: Some("The PDF file cannot be modified. Does it contain pauses?"),
        }),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::new(
                "%{X-Forwarded-For}i \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T",
            ))
            .service(
                web::resource("/")
                    .route(web::get().to(index))
                    .route(web::post().to(convert)),
            )
    })
    .bind((
        "0.0.0.0",
        env::var("PORT").map(|s| s.parse().unwrap()).unwrap_or(8080),
    ))?
    .run()
    .await
}
