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

    let mut text = vec![0;512];
    let mut imagepath = vec![0;512];

    if vaccel_bindings::image_classification(
        &mut sess,
        &mut body,
        &mut text,
        &mut imagepath,
    ).is_err() {
        return Err(error::ErrorBadRequest("Classification operations failed"));
    }

    if vaccel_bindings::close_session(&mut sess).is_err() {
        return Err(error::ErrorBadRequest("Could not close vaccel session"));
    }

    let mut tags = String::from_utf8(text).unwrap();
    tags.retain(|c| c != '\0');
    println!("test: {:?}", tags);
    Ok(HttpResponse::Ok().body(tags))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(classify_image)
    })
    .bind("0.0.0.0:3030")?
        .run()
        .await
}
