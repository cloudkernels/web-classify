use actix_web::{post, App, web, HttpResponse, HttpServer, Error, error};
use futures::StreamExt;

// maximum payload is 256M
const MAX_SIZE: usize = 268_435_456;

#[post("/classify")]
async fn classify_image(
    mut payload: web::Payload
) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();

    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;

        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    let mut sess: vaccel_bindings::vaccel_session = Default::default();
    if vaccel_bindings::new_session(&mut sess, 0).is_err() {
        return Err(error::ErrorBadRequest("Could not create vaccel session"));
    }

    let text = String::with_capacity(512);
    let imagepath = String::with_capacity(512);

    if vaccel_bindings::image_classification(
        &mut sess,
        &mut body,
        &mut text.into_bytes(),
        &mut imagepath.into_bytes(),
    ).is_err() {
        return Err(error::ErrorBadRequest("Classification operations failed"));
    }

    Ok(HttpResponse::Ok().body("Yoopee"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(classify_image)
    })
    .bind("127.0.0.1:3030")?
        .run()
        .await
}
