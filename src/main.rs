use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use hyper::{ Body, Response, Request, Server, Error, StatusCode, Method };
use futures::{ future, Future, Stream };
use hyper::service::service_fn;
use std::ops::Range;
use rand::Rng;
use rand::distributions::{Uniform, Normal, Bernoulli};
use failure::Fail;
use base64::STANDARD;
use base64_serde::base64_serde_type;

mod color;
use color::Color;

static INDEX:&'static str = "Microservice Rust";
base64_serde_type!(Base64Standard, STANDARD);

#[derive(Serialize)]
#[serde(rename_all="lowercase")]
enum RngResponse {
    Value(f64),
    #[serde(with="Base64Standard")]
    Bytes(Vec<u8>),
    Color(Color)

}


#[derive(Deserialize, Serialize)]
#[serde(tag = "distribution", rename_all = "lowercase", content = "parameters")]
enum RngRequest {
    Uniform {
        #[serde(flatten)]
        range: Range<i32>
    },
    Normal {
        mean: f64,
        std_dev: f64
    },
    Bernoulli {
        p: f64
    },
    Shuffle {
        #[serde(with="Base64Standard")]
        data: Vec<u8>
    },
    Color {
        from: Color,
        to: Color
    }
}

fn main() {
    let addr:SocketAddr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(|| {
        service_fn(|req| microservice_handler(req))
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);
}

fn microservice_handler(req:Request<Body>) -> Box<dyn Future<Item=Response<Body>, Error=Error> + Send>{
    match(req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            Box::new(future::ok(Response::new(INDEX.into())))
        },
        (&Method::POST, "/random") => {
            let body = req.into_body().concat2()
                .map(|chunk|{
                    let res = serde_json::from_slice::<RngRequest>(chunk.as_ref())
                        .map(body_handler)
                        .and_then(|resp| serde_json::to_string(&resp));
                    match res {
                        Ok(body) => {
                            Response::new(body.into())
                        }, 
                        Err(err) => {
                            Response::builder()
                                .status(StatusCode::UNPROCESSABLE_ENTITY)
                                .body(err.to_string().into())
                                .unwrap()
                        }
                    }
                });
            Box::new(body)
        },
        _ => {
            Box::new(future::ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()).unwrap()))
        }
    }
}


fn body_handler(request: RngRequest) -> RngResponse {
    let mut rng = rand::thread_rng();
    match request {
        RngRequest::Uniform { range } => {
            RngResponse::Value(rng.sample(Uniform::from(range)) as f64)
        },
        RngRequest::Normal { mean, std_dev } => {
            RngResponse::Value(rng.sample(Normal::new(mean, std_dev)) as f64)
        },
        RngRequest::Bernoulli { p } => {
            RngResponse::Value(rng.sample(Bernoulli::new(p)) as i8 as f64)
        },
        RngRequest::Shuffle { mut data } => {
            rng.shuffle(&mut data);
            RngResponse::Bytes(data)
        },
        RngRequest::Color { from, to } => {
            let red = rng.sample(Uniform::new_inclusive(from.red, to.red));
            let green = rng.sample(Uniform::new_inclusive(from.green, to.green));
            let blue = rng.sample(Uniform::new_inclusive(from.blue, to.blue));
            RngResponse::Color(Color{red, green, blue})
        }
    }
}

