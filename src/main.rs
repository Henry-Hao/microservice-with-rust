use std::path::Path;
use std::convert::Infallible;
use hyper::{Method, Server, Request, StatusCode, Response, Body};
use hyper::service::{ service_fn, make_service_fn};
use lazy_static::lazy_static;
use regex::Regex;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::sync::{ Mutex, Arc};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures::stream::{StreamExt, TryStreamExt};
use std::error::Error;
use hyper_staticfile::FileBytesStream;


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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>>{
    let files = Path::new("./files");
    std::fs::create_dir(files)?;
    let files = Arc::new(Mutex::new(files));
    let addr = ([127, 0, 0, 1], 8080).into();
    let make_svc = make_service_fn(|_conn| {
        let files = Arc::clone(&files);
        async { Ok::<_, Infallible>(service_fn(move |req| {
            let files= Arc::clone(&files);
            microservice_handler(req, files)
        })) }
    });
    let server = Server::bind(&addr).serve(make_svc);
    server.await?;

    Ok(())
}

async fn microservice_handler(req: Request<Body>, files: Arc<Mutex<&Path>>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    let method = req.method();
    let path = req.uri().path();
    match (method, path) {
        (&Method::GET, "/") => {
            Ok(Response::new(INDEX.into()))
        },
        (&Method::POST, "/upload") => {
            let name: String = thread_rng().sample_iter(&Alphanumeric).take(20).map(char::from).collect();
            let files = *(files.lock().unwrap());
            let mut filepath  = files.to_path_buf();
            filepath.push(&name);
            let create_file = File::create(filepath).await?;
            req.into_body().map_err(other).fold(create_file, |mut file, chunk| async move {
                file.write_all(&chunk.unwrap()).await;
                file
            }).await;
            Ok(Response::new(name.into()))
        },
        (&Method::GET, path) if path.starts_with("/download") => {
            if let Some(cap) = DOWNLOAD_FILE.captures(path) {
                let filename = cap.name("filename").unwrap().as_str();
                let files = *files.lock().unwrap();
                let mut filepath = files.to_path_buf();
                filepath.push(&filename);
                let open_file = File::open(filepath).await?;
                let stream = FileBytesStream::new(open_file);
                Ok(Response::new(Body::wrap_stream(stream)))
            } else {
                response_with_code(StatusCode::METHOD_NOT_ALLOWED)
            }
        },
        _ => response_with_code(StatusCode::METHOD_NOT_ALLOWED)
    }
}

fn response_with_code(code: StatusCode) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    Ok(Response::builder().status(code).body(Body::empty()).unwrap())
}

fn other<E>(err: E) -> std::io::Error
where E: Into<Box<dyn std::error::Error + Send + Sync>>
{
    std::io::Error::new(std::io::ErrorKind::Other, err)
}
