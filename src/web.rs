use actix_web::{error, web as a_web, HttpRequest};
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
struct WebfingerParams {
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

pub async fn webfinger(
    req: HttpRequest,
    data: a_web::Data<WebState>,
) -> Result<a_web::Json<WebfingerResponse>, error::Error> {
    let params = a_web::Query::<WebfingerParams>::from_query(req.query_string())?;
    let request_email_addr = params
        .resource
        .strip_prefix("acct:")
        .unwrap_or(params.resource.as_str())
        .to_string();
    let valid_user = data.config.contains_key(&request_email_addr);

    if valid_user {
        Ok(a_web::Json(WebfingerResponse {
            subject: params.resource.clone(),
            links: vec![WebfingerResponseLinks {
                rel: params.rel.clone(),
                href: data.auth_url.clone(),
            }],
        }))
    } else {
        Err(error::ErrorNotFound(format!(
            "No user with email address {request_email_addr} exists"
        )))
    }
}
