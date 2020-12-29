#![allow(deprecated)]
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use failure::Error;
use rouille::{ Request, Response };
use diesel::{ dsl::*, QueryDsl, RunQueryDsl, ExpressionMethods };
use serde::Serialize;
use log::debug;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rouille;
mod schema;
mod model;

#[derive(Serialize)]
struct UserId {
    id: String
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let manager = ConnectionManager::<SqliteConnection>::new("test.db");
    let pool = Pool::builder().build(manager)?;
    rouille::start_server("127.0.0.1:8001", move |request| {
        match handler(&request, &pool) {
            Ok(response) => response,
            Err(err) => Response::text(err.to_string()).with_status_code(500),
        }
    })
}

fn handler(
    request: &Request,
    pool: &Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Response, Error> {
    debug!("Request: {:?}", request);
    let resp = router!(request,
        (GET) (/) => {
            Response::text("Users Microservice")
        },
        (POST) (/signup) => {
            let data = post_input!(request, {
                email: String,
                password: String
            })?;
            debug!("Sign up for {}",data.email);
            let user_email:String = data.email.trim().to_lowercase();
            let user_password:String = data.password.trim().to_lowercase();
            {
                use self::schema::users::dsl::*;
                let conn = pool.get()?;
                let user_exists: bool = select(exists(users.filter(email.eq(user_email.clone())))).get_result(&conn)?;
        debug!("Request: {:?}", request);
                if !user_exists {
                    let uuid = format!("{}", uuid::Uuid::new_v4());
                    let new_user = model::NewUser {
                        id: &uuid,
                        email: &user_email,
                        password: &user_password
                    };
                    diesel::insert_into(schema::users::table)
                        .values(&new_user).execute(&conn)?;
                    Response::json(&())
                } else {
                    Response::text(format!("user {} exists", data.email)).with_status_code(400)
                }
            }
        },
        (POST) (/signin) => {
            let data = post_input!(request, {
                email: String,
                password: String
            })?;
            debug!("Sign in for {}",data.email);
            let user_email:String = data.email.trim().to_lowercase();
            let user_password:String = data.password.trim().to_lowercase();
            {
                use self::schema::users::dsl::*;
                let conn = pool.get()?;
                let user = users.filter(email.eq(user_email))
                    .first::<model::User>(&conn)?;
                let valid = user.password == user_password;
                if valid {
                    let user_id = UserId {
                        id: user.id
                    };
                    Response::json(&user_id).with_status_code(200)
                } else {
                    Response::text("Access denied").with_status_code(403)
                }
            }
        },
        _ => Response::empty_404()
    );
    Ok(resp)
}
