use crate::shared::{ErrMsgJsonGenerator, ErrResponse};
use axum::{
    body::Bytes,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use hyper::Body;
use tracing::{debug, info};

pub async fn print_request_response(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<impl IntoResponse, ErrResponse> {
    let (parts, body) = req.into_parts();
    info!("Request: {} '{}'", parts.method, parts.uri);
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    info!("Response: {}", parts.status);
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, ErrResponse>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                ErrMsgJsonGenerator::new(format!("failed to read {} body: {}", direction, err))
                    .generate(),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        debug!("{} body = {:?}", direction, body);
    }

    Ok(bytes)
}
