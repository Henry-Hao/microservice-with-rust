#[macro_use]
extern crate diesel;
use clap::{
    crate_name, crate_authors, crate_version, crate_description,
    App, Arg, SubCommand
};


use r2d2::Pool;
use diesel::{  r2d2::ConnectionManager, prelude::* };
use failure::Error;

pub mod schema;
pub mod models;


const CMD_ADD: &'static str = "add";
const CMD_LIST: &'static str = "list";

fn main() -> Result<(), Error>{
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("database")
             .short("d")
             .long("db")
             .value_name("FILE")
             .help("Set a file name of a database")
             .takes_value(true))
        .subcommand(SubCommand::with_name(CMD_ADD)
                    .about("Add user to the table")
                    .arg(Arg::with_name("NAME")
                         .help("Set the name of a user")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("EMAIL")
                         .help("Set an email of the user")
                         .required(true)
                         .index(2)))
        .subcommand(SubCommand::with_name(CMD_LIST)
                    .about("List users of the table"))
        .get_matches();
    let path = matches.value_of("database")
        .unwrap_or("test.db");
    let manager = ConnectionManager::<SqliteConnection>::new(path);
    let pool = Pool::new(manager)?;

    match matches.subcommand() {
        (CMD_ADD, Some(matches)) => {
            let conn = pool.get()?;
            let name = matches.value_of("NAME").unwrap();
            let email = matches.value_of("EMAIL").unwrap();
            let uuid = uuid::Uuid::new_v4().to_string();
            let new_user = models::NewUsers {
                name,
                email,
                id: &uuid
            };
            diesel::insert_into(schema::users::table)
                .values(&new_user)
                .execute(&conn)?;

        },
        (CMD_LIST, _) => {
            let conn = pool.get()?;
            use self::schema::users::dsl::users;
            let items = users.load::<models::User>(&conn)?;
            for user in items {
                println!("{:?}", user);
            }
        },
        _ => {
            matches.usage();
        }
    };



    Ok(())
}
