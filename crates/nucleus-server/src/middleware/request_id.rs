use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use uuid::Uuid;

pub async fn request_id_middleware(
    mut req: Request,
    next: Next,
) -> Response {
    let request_id =
        format!("req_{}", hex::encode(&Uuid::new_v4().as_bytes()[..8]));
    req.extensions_mut().insert(RequestId(request_id.clone()));

    let mut response = next.run(req).await;
    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&request_id).unwrap(),
    );
    response
}

#[derive(Clone, Debug)]
pub struct RequestId(pub String);
