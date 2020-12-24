#![allow(unused_must_use)]
use clap::{
    crate_name, crate_authors, crate_version, crate_description,
    App, AppSettings, Arg, SubCommand
};
use postgres:: {
    Error as DbError, NoTls, Client, Config, config::SslMode
};
use rayon::prelude::*;
use serde_derive::Deserialize;
use r2d2_postgres::PostgresConnectionManager;
use r2d2::Pool;

#[derive(Debug)]
enum Error {
    CliError(std::num::ParseIntError),
    DbError(DbError),
    CsvError(csv::Error),
    PoolError(r2d2::Error)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CliError(err) => {
                write!(f, "CliError:{}", err)
            },
            Error::DbError(err) => {
                write!(f, "DbError:{}", err)
            },
            Error::CsvError(err) => {
                write!(f, "CsvError:{}", err)
            },
            Error::PoolError(err) => {
                write!(f, "PoolError:{}", err)
            }
        }
    }
}

impl std::error::Error for Error { }
impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::CliError(err)
    }
}
impl From<DbError> for Error {
    fn from(err: DbError) -> Self {
        Error::DbError(err)
    }
}
impl From<csv::Error> for Error {
    fn from(err:csv::Error) -> Self {
        Error::CsvError(err)
    }
}
impl From<r2d2::Error> for Error {
    fn from(err:r2d2::Error) -> Self {
        Error::PoolError(err)
    }
}

#[derive(Deserialize)]
struct User {
    name: String,
    email: String
}


const CMD_CREATE: &str = "create";
const CMD_ADD: &str = "add";
const CMD_LIST: &str = "list";
const CMD_IMPORT: &str= "import";


fn main() -> Result<(), Error> {
    let matches = App::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequired)
        .arg(Arg::with_name("host")
             .short("h")
             .long("host")
             .value_name("HOST")
             .help("Set the host of db connection")
             .takes_value(true))
        .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .value_name("PORT")
             .help("Set the port of db connection")
             .takes_value(true))
        .arg(Arg::with_name("database")
             .short("d")
             .long("db")
             .value_name("DATABASE")
             .help("Set the databse of db connection")
             .takes_value(true))
        .subcommand(SubCommand::with_name(CMD_CREATE).about("create users table"))
        .subcommand(SubCommand::with_name(CMD_ADD).about("create a user record")
                    .arg(Arg::with_name("name")
                         .help("name of the user")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("email")
                         .help("email of the user")
                         .required(true)
                         .index(2)))
        .subcommand(SubCommand::with_name(CMD_LIST).about("print list of users"))
        .subcommand(SubCommand::with_name(CMD_IMPORT).about("import users from csv"))
        .get_matches();
    let host = matches.value_of("host").unwrap_or("localhost");
    let port = matches.value_of("port").unwrap_or("5432").parse::<u16>()?;
    let database = matches.value_of("database").unwrap_or("postgres");
    let mut config = Config::new();
    config.host(host)
        .port(port)
        .user(database);
    let manager = PostgresConnectionManager::new(config.to_owned(), NoTls);
    let pool = r2d2::Pool::new(manager).unwrap();
    match matches.subcommand() {
        (CMD_CREATE, _) => {
            create_table(pool)?;
        },
        (CMD_LIST, _) => {
            let users = list_users(pool)?;
            for user in users {
                println!("Name: {:20} Email: {:20}", user.name, user.email);
            }
        },
        (CMD_ADD, Some(matches)) => {
            let name = matches.value_of("name").unwrap();
            let email = matches.value_of("email").unwrap();
            create_user(pool, &User{ name: name.to_string(), email: email.to_string() })?;
        },
        (CMD_IMPORT, _) => {
            let mut rdr = csv::Reader::from_reader(std::io::stdin());
            let mut users = Vec::new();
            for user in rdr.deserialize() {
                users.push(user?)
            }
            users.par_iter().map(|user| -> Result<(), failure::Error>{
                let pool = pool.clone();
                create_user(pool, &user)?;
                Ok(())
            }).for_each(drop);
        },
        _ => {
            matches.usage();
        }
    }
    Ok(())
}

fn create_table(pool: r2d2::Pool<PostgresConnectionManager<NoTls>>) -> Result<(), Error> {
    let mut conn = pool.get()?;
    conn.execute("CREATE TABLE users (
                        id SERIAL PRIMARY KEY,
                        name VARCHAR NOT NULL,
                        email VARCHAR NOT NULL)", &[])
        .map(drop)
        .map_err(From::from)
}


fn create_user(pool: r2d2::Pool<PostgresConnectionManager<NoTls>>, user: &User) -> Result<(), Error> {
    let mut conn = pool.get()?;
    conn.execute("INSERT INTO users (name, email) VALUES ($1, $2)", &[&user.name, &user.email])
        .map(drop)
        .map_err(From::from)
}

fn list_users(pool: r2d2::Pool<PostgresConnectionManager<NoTls>>) -> Result<Vec<User>, Error> {
    let mut conn = pool.get()?;
    let res = conn.query("SELECT name, email FROM users", &[])?.into_iter()
        .map(|row| User {name: row.get(0), email: row.get(1)})
        .collect();
    Ok(res)
}
