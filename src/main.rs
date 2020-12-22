use std::path::{Path,PathBuf};
use std::convert::Infallible;
use hyper::{Method, Server, Request, StatusCode, Response, Body};
use hyper::service::{ service_fn, make_service_fn};
use lazy_static::lazy_static;
use regex::Regex;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::sync::{ Mutex, Arc};
use tokio::fs::File;
use futures::stream::Stream;
use tokio::io::AsyncWriteExt;
use futures::future::{Future, FutureExt, TryFuture};
use futures::stream::{StreamExt, TryStreamExt};
use std::error::Error;


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
    std::fs::create_dir(files);
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
            let write = req.into_body().map_err(other).fold(create_file, |mut file, chunk| async move {
                file.write_all(&chunk.unwrap()).await;
                file
            }).await;
            Ok(Response::new(name.into()))
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

// // async fn microservice_handler(req: Request<Body>, files: &Path) -> Box<dyn Future<Output = Result<Response<Body>, Infallible>>> {
// async fn microservice_handler(req: Request<Body>, files: &Path) -> Result<Response<Body>, Infallible> {
//     let method = req.method();
//     let path = req.uri().path();
//     match (method, path) {
//         (&Method::GET, "/") => {
//             // Box::new(future::ok(Response::new(INDEX.into())))
//             Ok(Response::new(INDEX.into()))
//         },
//         (&Method::POST, "/upload") => {
//             let name: String = thread_rng().sample_iter(&Alphanumeric).take(20).map(char::from).collect();
//             let mut filepath = files.to_path_buf();
//             filepath.push(&name);
//             // let create_file = fs::File::create(filepath);
//             let create_file = fs::File::create(filepath).await?;
//             let write = create_file.and_then(|file| async {
//                 req.into_body()
//                     // .map_err(other)
//                     .map(|chunk| {
//                         file.write_all(&chunk.unwrap())
//                     })
//             });
//             let body = write.map(|_| {
//                 Response::new(name.into())
//             });
//             // Box::new(future::ok(body))
//             body
//         },
//         (&Method::GET, path) if path.starts_with("/download") => {
//             if let Some(cap) = DOWNLOAD_FILE.captures(path) {
//                 let name = cap.name("filename").unwrap().as_str();
//                 let mut filepath = files.to_path_buf();
//                 filepath.push(&name);
//                 let open_file = fs::File::open(filepath);
//                 let body = open_file.map(|file| {
//                     let stream = FileBytesStream::new(file);
//                     Response::new(Body::wrap_stream(stream))
//                 });
//                 // Box::new(future::ok(body))
//                 body
//
//             } else {
//                 response_with_code(StatusCode::NOT_FOUND)
//             }
//         },
//         _ => {
//             response_with_code(StatusCode::METHOD_NOT_ALLOWED)
//         }
//     }
//
// }
//
//
// fn other<E>(err: E) -> io::Error
// where E: Into<Box<dyn std::error::Error + Send + Sync>>
// {
//     io::Error::new(io::ErrorKind::Other, err)
// }
//
//
// // fn response_with_code(code: StatusCode) -> Box<dyn Future<Output = Result<Response<Body>, Infallible>>> {
// fn response_with_code(code: StatusCode) -> Result<Response<Body>, Infallible> {
//     // Box::new(future::ok(Response::builder().status(code).body(Body::empty()).unwrap()))
//     Ok(Response::builder().status(code).body(Body::empty()).unwrap())
// }
