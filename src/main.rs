#![forbid(unsafe_code)]

use crate::web::{UsersDatabase, WebState};
use axum::Router;
use axum::routing::get;
use clap::{crate_name, crate_version};
use std::fs;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tracing::{Level, info};

mod web;

#[tokio::main]
async fn main() {
    let ansi_enabled = fix_ansi_term();

    tracing_subscriber::fmt()
        .with_ansi(ansi_enabled)
        .with_max_level(Level::DEBUG)
        .init();

    let matches = clap::Command::new("authelia_webfinger")
        .version(crate_version!())
        .author("Chris Lieb")
        .arg(
            clap::Arg::new("ip")
                .short('i')
                .long("ip")
                .default_value("0.0.0.0"),
        )
        .arg(
            clap::Arg::new("port")
                .short('p')
                .long("port")
                .default_value("8081"),
        )
        .arg(
            clap::Arg::new("conf")
                .short('c')
                .long("conf")
                .default_value("users_database.yml")
                .help("Path to the Authelia users_database.yml file"),
        )
        .arg(
            clap::Arg::new("auth_url")
                .short('u')
                .long("auth-url")
                .required(true)
                .help("The callback URL for performing auth"),
        )
        .get_matches();

    let ip = matches.get_one::<String>("ip").unwrap().clone();
    let port = matches
        .get_one::<String>("port")
        .unwrap()
        .clone()
        .parse::<u16>()
        .unwrap();
    let config_path = matches.get_one::<String>("conf").unwrap().clone();
    let auth_url = matches.get_one::<String>("auth_url").unwrap().clone();

    info!(
        "Starting {} v{}: http://{}:{} for {}",
        crate_name!(),
        crate_version!(),
        ip,
        port,
        config_path
    );

    let raw_config = fs::read_to_string(config_path).expect("Unable to find users database file");
    let config = serde_saphyr::from_str::<UsersDatabase>(raw_config.as_str())
        .expect("Unable to parse YAML in users database file");

    let app = Router::new()
        .route("/webfinger", get(web::webfinger))
        .with_state(WebState::new(config.clone(), auth_url.clone()))
        .layer(
            ServiceBuilder::new().layer(CompressionLayer::new()).layer(
                TraceLayer::new_for_http()
                    .on_request(())
                    .on_eos(())
                    .on_body_chunk(()),
            ),
        );
    let listener = tokio::net::TcpListener::bind(format!("{ip}:{port}"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(target_os = "windows")]
fn fix_ansi_term() -> bool {
    nu_ansi_term::enable_ansi_support().is_ok_and(|()| true)
}

#[cfg(not(target_os = "windows"))]
fn fix_ansi_term() -> bool {
    true
}
