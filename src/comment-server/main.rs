#![feature(decl_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate rocket;

mod model;
use model::{ Comment, NewComment };

use rocket::fairing::AdHoc;
use rocket::request::Form;
use rocket_contrib::json::Json;
use diesel::SqliteConnection;

#[database("sqlite_database")]
pub struct Db(SqliteConnection);
// the path should be relative to the directory containing 'cargo.toml'
embed_migrations!("migrations/comment-server");

#[get("/list")]
fn list(conn: Db) -> Json<Vec<Comment>> {
    Json(Comment::all(&conn))
}
#[post("/new_comment", data="<comment_form>")]
fn add_new(comment_form: Form<NewComment>, conn: Db) {
    let comment = comment_form.into_inner();
    Comment::insert(comment, &conn);
}



fn main() {
    rocket::ignite()
        .attach(Db::fairing())
        .attach(AdHoc::on_request("Logging", |req, _data| {
            debug!("Incoming request:{}", req);
        }))
        .attach(AdHoc::on_attach("Database Migration", |rocket| {
            let conn = Db::get_one(&rocket).expect("no database connections");
            match embedded_migrations::run(&*conn) {
                Ok(_) => Ok(rocket),
                Err(err) => {
                    error!("Failed to run database migrations :{:?}", err);
                    Err(rocket)
                }
            }
        }))
        .mount("/", routes![list, add_new])
        .launch();
}


