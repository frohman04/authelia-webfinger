#![forbid(unsafe_code)]

extern crate actix_web;
extern crate clap;
extern crate tracing;
extern crate tracing_actix_web;
extern crate tracing_log;
extern crate tracing_subscriber;

use actix_web::error;
use actix_web::middleware::{Compress, Logger};
use actix_web::web::Data;
use actix_web::{web as a_web, web, App, HttpRequest, HttpServer};
use clap::{crate_name, crate_version};
use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use tracing_actix_web::TracingLogger;
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ansi_enabled = fix_ansi_term();
    LogTracer::init().expect("routing log to tracing failed");

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_ansi(ansi_enabled)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

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
                .default_value("users_database.yaml")
                .help("Path to the Authelia users_database.yaml file"),
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

    info!(
        "Starting {} v{}: http://{}:{} for {}",
        crate_name!(),
        crate_version!(),
        ip,
        port,
        config_path
    );

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(Compress::default())
            .route("webfinger", a_web::get().to(webfinger))
    })
    .bind(format!("{}:{}", ip, port))?
    .run()
    .await
}

#[derive(Debug, Deserialize)]
struct WebfingerParams {
    rel: String,
    resource: String,
}

#[derive(Debug, Serialize)]
struct WebfingerOutputLinks {
    rel: String,
    href: String,
}

#[derive(Debug, Serialize)]
struct WebfingerOutput {
    subject: String,
    links: Vec<WebfingerOutputLinks>,
}

async fn webfinger(req: HttpRequest) -> Result<web::Json<WebfingerOutput>, error::Error> {
    todo!()
}

#[cfg(target_os = "windows")]
fn fix_ansi_term() -> bool {
    nu_ansi_term::enable_ansi_support().is_ok_and(|()| true)
}

#[cfg(not(target_os = "windows"))]
fn fix_ansi_term() -> bool {
    true
}
