use std::path::Path;
use std::io;
use tokio::fs;
use tokio::prelude::Future;
use hyper::{ Request, Method, Body, Server, Response, StatusCode };
use hyper::service::service_fn;
use futures::future;
use hyper::rt::{ Stream};
use rand::{ thread_rng, Rng, distributions::Alphanumeric };
use lazy_static::lazy_static;
use regex::Regex;
use hyper_staticfile::FileChunkStream;

const INDEX: &'static str= r#"
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

lazy_static!{
    static ref DOWNLOAD_FILE: Regex = Regex::new("^/download/(?P<filename>\\w{20})?$").unwrap();
}


fn main() {
    let files = Path::new("./files");
    std::fs::create_dir(files).ok();
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(move || {
        service_fn(move |req| microservice_handler(req, &files))
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);

}


fn microservice_handler(req: Request<Body>, files: &Path) -> Box<dyn Future<Item=Response<Body>, Error=std::io::Error> + Send> {
    let method = req.method();
    let path = req.uri().path();
    match (method, path) {
        (&Method::GET, "/") => {
            Box::new(future::ok(Response::new(INDEX.into())))
        },
        (&Method::POST, "/upload") => {
            let name: String = thread_rng().sample_iter(&Alphanumeric).take(20).collect();
            let mut filepath = files.to_path_buf();
            filepath.push(&name);
            let create_file = fs::File::create(filepath);
            let write = create_file.and_then(|file| {
                req.into_body()
                    .map_err(other)
                    .fold(file, |file, chunk| {
                        tokio::io::write_all(file, chunk)
                            .map(|(file,_)| file)
                    })
            });
            let body = write.map(|_| {
                Response::new(name.into())
            });
            Box::new(body)
        },
        (&Method::GET, path) if path.starts_with("/download") => {
            if let Some(cap) = DOWNLOAD_FILE.captures(path) {
                let name = cap.name("filename").unwrap().as_str();
                let mut filepath = files.to_path_buf();
                filepath.push(&name);
                let open_file = fs::File::open(filepath);
                let body = open_file.map(|file| {
                    let stream =FileChunkStream::new(file);
                    Response::new(Body::wrap_stream(stream))
                });
                Box::new(body)

            } else {
                response_with_code(StatusCode::NOT_FOUND)
            }
        },
        _ => {
            response_with_code(StatusCode::METHOD_NOT_ALLOWED)
        }
    }

}


fn other<E>(err: E) -> io::Error 
where E: Into<Box<dyn std::error::Error + Send + Sync>>
{
    io::Error::new(io::ErrorKind::Other, err)
}


fn response_with_code(code: StatusCode) -> Box<dyn Future<Item=Response<Body>, Error=std::io::Error> + Send> {
    Box::new(future::ok(Response::builder().status(code).body(Body::empty()).unwrap()))
}
