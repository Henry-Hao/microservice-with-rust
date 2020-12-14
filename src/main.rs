use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use hyper::{Body, Response, Server };
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use log::{debug, info, trace};
use clap::*;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
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
        .get_matches();
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    info!("Rand microservice - v0.1.0");
    trace!("Starting... ",);
    let addr: SocketAddr = matches.value_of("address")
        .map(|x| x.to_owned())
        .or(env::var("ADDRESS").ok())
        .unwrap_or("127.0.0.1:8080".into())
        .parse()
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

