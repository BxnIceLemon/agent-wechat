use std::future::Future;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use crate::{ia::types::Session, sessions::manager};

pub struct ActiveSession(pub Session);

fn requested_session_id(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-session-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_owned()
}

impl<S> FromRequestParts<S> for ActiveSession
where
    S: Send + Sync,
{
    type Rejection = Response;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let session_id = requested_session_id(&parts.headers);

        async move {
            manager::get_session(&session_id).map(Self).ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Session not found" })),
                )
                    .into_response()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::requested_session_id;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn session_header_selects_account_and_defaults_safely() {
        let mut headers = HeaderMap::new();
        assert_eq!(requested_session_id(&headers), "default");
        headers.insert("x-session-id", HeaderValue::from_static("account-2"));
        assert_eq!(requested_session_id(&headers), "account-2");
    }
}
