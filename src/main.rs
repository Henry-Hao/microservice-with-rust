use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use hyper::{Body, Response, Server };
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use log::{debug, info, trace, warn};
use clap::{ crate_name, crate_authors, crate_version, crate_description, Arg, App, AppSettings, SubCommand };
use serde_derive::Deserialize;
use std::fs::File;
use std::io::{Read, self};

#[derive(Deserialize)]
struct Config {
    address: SocketAddr
}

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("run")
                    .about("Run the server")
                    .arg(Arg::with_name("address")
                         .short("a")
                         .long("address")
                         .value_name("ADDRESS")
                         .help("Set an address")
                         .takes_value(true))
                    .arg(Arg::with_name("config")
                         .short("c")
                         .long("config")
                         .value_name("FILE")
                         .help("Specify a config file")
                         .takes_value(true))
                    .subcommand(SubCommand::with_name("key")
                                .about("Generate a secret key for cookies")))
        .get_matches();
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    info!("Rand microservice - v0.1.0");
    trace!("Starting... ");
    if let Some(sub_m) = matches.subcommand_matches("run") {
        let config = File::open("microservice.toml")
            .and_then(|mut file| {
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
                Ok(buffer)
            }).and_then(|buffer|{
                toml::from_str::<Config>(&buffer)
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
            }).map_err(|err|{
                warn!("Can't read config file :{}", err);
            })
        .ok();
        if let Some(_) = sub_m.subcommand_matches("key") {
            let random_byte = rand::random::<u8>();
            println!("Your secret key is:{}", random_byte);
        } else {
            let addr: SocketAddr = config.map(|c| {
                debug!("Trying config file");
                c.address
            })
            .or_else(|| {
                debug!("Trying env variable");
                env::var("ADDRESS").ok().map(|addr| addr.parse().unwrap())
            })
            .or_else(|| {
                debug!("Trying command line");
                sub_m.value_of("address").map(|addr| addr.to_owned().parse().unwrap())
            })
            .or_else(|| {
                debug!("Using default");
                Some(([127, 0, 0, 1], 9999).into())
            })
            .expect("Cannot parse the address");
            debug!("Trying to bind server with address {}:", addr);
            let builder = Server::bind(&addr);
            trace!("Creating service handler...");
            let server = builder.serve(|| {
                service_fn_ok(|req| {
                    trace!("Incoming request is: {:?}", req);
                    let random_byte = rand::random::<u8>();
                    debug!("Generated value is: {}", random_byte);
                    Response::new(Body::from(random_byte.to_string()))
                })
            });
            info!("Used address: {}", server.local_addr());
            let server = server.map_err(drop);
            debug!("Run");
            hyper::rt::run(server);
        }
    }
}

