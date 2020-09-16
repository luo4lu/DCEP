use actix_web::{App, HttpServer};
mod config;
mod config_command;
mod currency_transaction;
use actix_cors::Cors; //跨域crate
use clap::ArgMatches;
use log::Level;
pub mod response;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(Level::Warn).unwrap();

    let mut _path: String = String::new();
    let matches: ArgMatches = config_command::get_command();
    if let Some(t) = matches.value_of("dcdt") {
        _path = t.to_string();
    }else{
        _path = String::from("127.0.0.1:9002");
    }
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::new().supports_credentials().finish())
            .data(config::get_db())
            .data(config::ConfigPath::default())
            .service(currency_transaction::digistal_transaction)
    })
    .bind(_path)?
    .run()
    .await
}