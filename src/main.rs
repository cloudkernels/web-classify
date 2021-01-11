use actix_web::{post, App, web, HttpResponse, HttpServer, Error, error};
use futures::StreamExt;
use log::{debug, error};

extern crate pretty_env_logger;

// maximum payload is 256M
const MAX_SIZE: usize = 268_435_456;

#[post("/classify")]
async fn classify_image(
    mut payload: web::Payload
) -> Result<HttpResponse, Error> {
    debug!("Handling classification request");

    let mut body = web::BytesMut::new();

    let hostname =
        match hostname::get() {
            Ok(name) => name,
            Err(_) => {
                error!("Could not retrieve server's hostname");
                return Err(error::ErrorBadRequest("Server error: Could not get hostname"));
            }
        };

    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;

        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    debug!("Received image");

    let mut sess: vaccel_bindings::vaccel_session = Default::default();
    if vaccel_bindings::new_session(&mut sess, 0).is_err() {
        return Err(error::ErrorBadRequest("Could not create vaccel session"));
    }
    debug!("Created vaccel session with id: {:?}", sess.session_id);

    let mut text = vec![0;512];
    let mut imagepath = vec![0;512];

    let tags = match vaccel_bindings::image_classification(
        &mut sess,
        &mut body,
        &mut text,
        &mut imagepath,
    ) {
        Ok(()) => {
            let mut t =
                match String::from_utf8(text) {
                    Ok(t) => t,
                    Err(_e) => return Err(error::ErrorBadRequest("Could not get vaccel string")),
                };

            t.retain(|c| c != '\0');
            t
        }
        Err(_e) => {
            return Err(error::ErrorBadRequest("Classification operations failed"));
        }
    };

    debug!("Classification completed. Tags: {:?}", tags);

    if vaccel_bindings::close_session(&mut sess).is_err() {
        error!("Could not close vaccel session: {:?}", sess.session_id);
        return Err(error::ErrorBadRequest("Could not close vaccel session"));
    }
    debug!("Closed vaccel session {:?}", sess.session_id);

    Ok(HttpResponse::Ok()
       .body(format!("[{:?}]: {:?}\n", hostname, tags)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init_timed();

    HttpServer::new(|| {
        App::new()
            .service(classify_image)
    })
    .workers(1)
    .bind("0.0.0.0:3030")?
        .run()
        .await
}
