use actix_web::{
    dev::HttpResponseBuilder, error, http::header, http::StatusCode, post, web, App, HttpResponse,
    HttpServer,
};
use derive_more::{Display, Error};
use futures::StreamExt;
use log::debug;
use vaccel_bindings::vaccel_session;

extern crate pretty_env_logger;

// maximum payload is 256M
const MAX_SIZE: usize = 268_435_456;

#[derive(Debug, Display, Error)]
enum Error {
    #[display(fmt = "vAccel error({}): {}", err, msg)]
    VAccel { err: u32, msg: String },

    #[display(fmt = "An internal error occured: {}", msg)]
    Internal { msg: String },

    #[display(fmt = "Request error: {}", msg)]
    Request { msg: String },
}

impl error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            Error::VAccel { .. } => StatusCode::BAD_REQUEST,
            Error::Internal { .. } => StatusCode::BAD_REQUEST,
            Error::Request { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

#[post("/classify")]
async fn classify_image(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    debug!("Handling classification request");

    let mut body = web::BytesMut::new();

    let hostname = hostname::get().map_err(|_e| Error::Internal {
        msg: "Server error: Could not get hostname".to_string(),
    })?;

    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(|_e| Error::Request {
            msg: "Could not read payload".to_string(),
        })?;

        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(Error::Request {
                msg: "overflow".to_string(),
            });
        }
        body.extend_from_slice(&chunk);
    }
    debug!("Received image");

    let mut sess = vaccel_session::new(0).map_err(|e| Error::VAccel {
        err: e,
        msg: "Could not create vAccel session".to_string(),
    })?;

    debug!("Created vaccel session with id: {:?}", sess.session_id);

    let (tags, _imgpath) = sess
        .image_classification(&body)
        .map_err(|e| Error::VAccel {
            err: e,
            msg: "Could not classify image".to_string(),
        })?;

    let tags = match String::from_utf8(tags) {
        Ok(mut t) => t.retain(|c| c != '\0'),
        Err(_e) => {
            return Err(Error::Internal {
                msg: "Could not get tags".to_string(),
            })
        }
    };

    debug!("Classification completed. Tags: {:?}", tags);

    sess.close().map_err(|e| Error::VAccel {
        err: e,
        msg: "Could not close vAccel session".to_string(),
    })?;

    debug!("Closed vaccel session {:?}", sess.session_id);

    Ok(HttpResponse::Ok().body(format!("[{:?}]: {:?}\n", hostname, tags)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init_timed();

    HttpServer::new(|| App::new().service(classify_image))
        .workers(1)
        .bind("0.0.0.0:3030")?
        .run()
        .await
}
