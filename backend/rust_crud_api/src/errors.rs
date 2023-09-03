use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

pub enum CustomError {
    _BadRequest,
    TaskNotFound,
    InternalServerError
}

impl IntoResponse for CustomError {
    fn into_response(self)-> axum::response::Response {
        let (status, error_message) = match self {
            Self::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
            ),
            Self::_BadRequest => (StatusCode::BAD_REQUEST, "Bad Request"),
            Self::TaskNotFound => (StatusCode::NOT_FOUND, "Information Not Found")
        };
        (status, Json(json!({"Error": error_message}))).into_response()
    }
}