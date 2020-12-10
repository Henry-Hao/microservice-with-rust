use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use futures::{Future, future};
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode };
use hyper::service::service_fn;
use slab::Slab;
use std::fmt;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static!{
    static ref INDEX_PATH: Regex = Regex::new("^/(index\\.html?)?$").unwrap();
    static ref USER_PATH: Regex = Regex::new("^/user/((?P<user_id>\\d+?)/?)?$").unwrap();
    static ref USERS_PATH: Regex = Regex::new("^/users/?$").unwrap();
}


type UserId = u64;
struct UserData;
type UserDB = Arc<Mutex<Slab<UserData>>>;

const INDEX: &'static str= r#"
<!doctype html>
<html>
    <head>
        <title> Rust Microservice </title>
    </head>
    <body>
        <h3> Rust Microservice </h3>
    </body>
</html>
"#;


impl fmt::Display for UserData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("{}")
    }
}

fn main() {
    let addr:SocketAddr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let user_db = Arc::new(Mutex::new(Slab::new()));
    let server = builder.serve(move || {
        let user_db = user_db.clone();
        service_fn(move |req| microservice_handler(req, &user_db))
    });
    let server = server.map_err(drop);
    hyper::rt::run(server);
}


fn response_with_code(code: StatusCode) -> Response<Body> {
    Response::builder()
        .status(code)
        .body(Body::empty())
        .unwrap()
}

fn microservice_handler(req: Request<Body>, user_db: &UserDB) -> impl Future<Item=Response<Body>, Error=Error> {
    let method = req.method();
    let path = req.uri().path();
    let mut users = user_db.lock().unwrap();
    let response = {
        if INDEX_PATH.is_match(path) {
            if method == &Method::GET {
                Response::new(INDEX.into())
            } else {
                response_with_code(StatusCode::BAD_REQUEST)
            }
        } else if USERS_PATH.is_match(path) {
            if method == &Method::GET {
                let list = users.iter()
                    .map(|(id, _)| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                Response::new(list.into())
            } else {
                response_with_code(StatusCode::BAD_REQUEST)
            }
        } else if let Some(cap) = USER_PATH.captures(path) {
            let user_id = cap.name("user_id").and_then(|x| {
                x.as_str()
                    .parse::<UserId>()
                    .ok()
                    .map(|x| x as usize)
            });
            match (method, user_id) {
                (&Method::GET, Some(user_id)) => {
                    // get user with user_id
                    if let Some(data) = users.get(user_id) {
                        Response::new(data.to_string().into())
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                },
                (&Method::POST, None) => {
                    // create user
                    let id = users.insert(UserData);
                    Response::new(id.to_string().into())
                },
                (&Method::POST, Some(_)) => {
                    // bad request
                    response_with_code(StatusCode::BAD_REQUEST)
                },
                (&Method::PUT, Some(user_id)) => {
                    // update user with user_id
                    if let Some(user) = users.get_mut(user_id) {
                        *user = UserData;
                        response_with_code(StatusCode::OK)
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                },
                (&Method::DELETE, Some(user_id)) => {
                    // delete user with user_id
                    if users.contains(user_id) {
                        users.remove(user_id);
                        response_with_code(StatusCode::OK)
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                },
                _ => {
                    response_with_code(StatusCode::METHOD_NOT_ALLOWED)
                }
            }
        } else {
                response_with_code(StatusCode::BAD_REQUEST)
        }
    };
    future::ok(response)
}
