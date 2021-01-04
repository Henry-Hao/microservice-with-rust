use futures::TryFutureExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{server::Server, Body, Method, Request, Response, StatusCode};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::error::Error;


use tokio::sync::{mpsc, oneshot};

mod worker;
use worker::*;

const INDEX: &'static str = r#"
<!doctype html>
<html>
    <head>
        <title> Rust Microservice </title>
    </head>
    <body>
        <h3> Image service </h3>
    </body>
</html>
"#;

lazy_static! {
    static ref DOWNLOAD_FILE: Regex = Regex::new("^/download/(?P<filename>\\w{20})?$").unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 8080).into();
    let sender = start_worker();
    let make_service = make_service_fn(move |_| {
        let sender = sender.clone();
        async {
            Ok::<_, std::io::Error>(service_fn(move |req| {
                microservice_handler(sender.clone(), req)
            }))
        }
    });
    let server = Server::bind(&addr).serve(make_service);
    server.await?;

    Ok(())
}

async fn microservice_handler(
    sender: mpsc::Sender<WorkerRequest>,
    req: Request<Body>,
) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(INDEX.into())),
        (&Method::POST, "/resize") => {
            let (width, height) = {
                let uri = req.uri().query().unwrap_or("");
                let query = queryst::parse(uri).unwrap_or(Value::Null);
                let width = to_number(&query["width"], 180);
                let height = to_number(&query["height"], 180);
                (width, height)
            };
            let buffer: Vec<u8> = hyper::body::to_bytes(req.into_body())
                .map_err(other)
                .and_then(|chunk| futures::future::ok(chunk.to_vec()))
                .await?;
            let (resp_sender, resp_receiver) = oneshot::channel::<WorkerResponse>();
            let request = WorkerRequest {
                buffer,
                width,
                height,
                sender: resp_sender,
            };
            println!("75");
            let body = sender
                .send(request)
                .map_err(other)
                .and_then(move |_| async {
                    println!("79");
                    let text = resp_receiver.map_err(other).await?;
                    Ok::<Response<Body>, _>(Response::new(text.unwrap().into()))
                })
                .await?;
            Ok(body)
        }
        _ => response_with_code(StatusCode::NOT_FOUND),
    }
}

fn response_with_code(code: StatusCode) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    Ok(Response::builder()
        .status(code)
        .body(Body::empty())
        .unwrap())
}

fn other<E>(err: E) -> std::io::Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    std::io::Error::new(std::io::ErrorKind::Other, err)
}

fn to_number(value: &Value, default: u16) -> u16 {
    value
        .as_str()
        .and_then(|x| x.parse::<u16>().ok())
        .unwrap_or(default)
}
