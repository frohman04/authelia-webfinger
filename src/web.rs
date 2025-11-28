use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct UsersDatabaseUser {
    disabled: bool,
    displayname: String,
    password: String,
    email: String,
    groups: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UsersDatabase {
    users: HashMap<String, UsersDatabaseUser>,
}

#[derive(Clone)]
pub struct WebState {
    // email -> username
    config: HashMap<String, String>,
    auth_url: String,
}

impl WebState {
    pub fn new(db: UsersDatabase, auth_url: String) -> Self {
        Self {
            config: db
                .users
                .into_iter()
                .map(|(key, value)| (value.email, key))
                .collect(),
            auth_url,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WebfingerParams {
    rel: String,
    resource: String,
}

#[derive(Debug, Serialize)]
struct WebfingerResponseLinks {
    rel: String,
    href: String,
}

#[derive(Debug, Serialize)]
pub struct WebfingerResponse {
    subject: String,
    links: Vec<WebfingerResponseLinks>,
}

pub enum WebfingerError {
    UserNotFound(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    message: String,
}

impl IntoResponse for WebfingerError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            WebfingerError::UserNotFound(email) => (
                StatusCode::NOT_FOUND,
                format!("No user with email address {email} exists"),
            ),
        };
        (status, Json(ErrorResponse { message })).into_response()
    }
}

pub async fn webfinger(
    Query(params): Query<WebfingerParams>,
    State(data): State<WebState>,
) -> Result<Json<WebfingerResponse>, WebfingerError> {
    let request_email_addr = params
        .resource
        .strip_prefix("acct:")
        .unwrap_or(params.resource.as_str())
        .to_string();
    let valid_user = data.config.contains_key(&request_email_addr);

    if valid_user {
        Ok(Json(WebfingerResponse {
            subject: params.resource.clone(),
            links: vec![WebfingerResponseLinks {
                rel: params.rel.clone(),
                href: data.auth_url.clone(),
            }],
        }))
    } else {
        Err(WebfingerError::UserNotFound(request_email_addr))
    }
}
