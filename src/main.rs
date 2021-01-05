use warp::{http, Filter};

async fn classify_image(
    bytes: bytes::Bytes
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("bytes: {:?}", bytes);

    let mut sess: vaccel_bindings::vaccel_session = Default::default();
    if vaccel_bindings::new_session(&mut sess, 0).is_err() {
        return Ok(warp::reply::with_status(
                "Could not create vaccel session",
                http::StatusCode::INTERNAL_SERVER_ERROR
        ));
    }

    let mut text = String::with_capacity(512);
    let mut imagepath = String::with_capacity(512);

    if vaccel_bindings::image_classification(
        &mut sess,
        bytes.borrow_mut(),
        text.as_mut_vec(),
        imagepath.as_mut_vec(),
    ).is_err() {
        return Ok(warp::reply::with_status(
                "classification op failed",
                http::StatusCode::INTERNAL_SERVER_ERROR
        ));
    }

    Ok(warp::reply::with_status(
            text.as_str(),
            http::StatusCode::CREATED
    ))
}

fn post_bytes() -> impl Filter<Extract = (bytes::Bytes,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(32 * 1024 * 1024)
        .and(warp::body::bytes())
}

#[tokio::main]
async fn main() {
    // GET /classify
    let classify = warp::post()
        .and(warp::path("classify"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(32 * 1024 * 1024))
        .and(post_bytes())
        .and_then(classify_image);

    warp::serve(classify)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
