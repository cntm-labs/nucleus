use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use uuid::Uuid;

pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let rid = RequestId(format!(
        "req_{}",
        hex::encode(&Uuid::new_v4().as_bytes()[..8])
    ));
    let header_value = HeaderValue::from_str(&rid.0).unwrap();
    req.extensions_mut().insert(rid);

    let mut response = next.run(req).await;
    response.headers_mut().insert("x-request-id", header_value);
    response
}

#[derive(Clone, Debug)]
pub struct RequestId(pub String);
