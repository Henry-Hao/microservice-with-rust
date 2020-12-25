#![allow(unused_must_use)]
use clap::{
    crate_name, crate_authors, crate_version, crate_description,
    App, AppSettings, Arg, SubCommand
};
use redis::{ Commands, RedisError };
use r2d2_redis::RedisConnectionManager;
use std::collections::HashMap;

#[derive(Debug)]
enum Error {
    CliError(std::num::ParseIntError),
    DbError(RedisError),
    CsvError(csv::Error),
    PoolError(r2d2::Error),
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
impl From<RedisError> for Error {
    fn from(err: RedisError) -> Self {
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

const CMD_REMOVE: &str = "remove";
const CMD_ADD: &str = "add";
const CMD_LIST: &str = "list";
const SESSIONS: &str = "sessions";


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
        .subcommand(SubCommand::with_name(CMD_ADD).about("create a session record")
                    .arg(Arg::with_name("token")
                         .help("Token")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("uid")
                         .help("uid of the user")
                         .required(true)
                         .index(2)))
        .subcommand(SubCommand::with_name(CMD_REMOVE).about("remove a session record")
                    .arg(Arg::with_name("token")
                         .help("Token")
                         .required(true)
                         .index(1)))
        .subcommand(SubCommand::with_name(CMD_LIST).about("print list of sessions"))
        .get_matches();
    let host = matches.value_of("host").or(Some("127.0.0.1")).unwrap();
    let port = matches.value_of("port").unwrap_or("6379").parse::<u16>()?;
    let manager = RedisConnectionManager::new(format!("redis://{}:{}/", host, port).as_str())?;
    let pool = r2d2::Pool::builder().build(manager)?;
    match matches.subcommand() {
        (CMD_REMOVE, Some(matches)) => {
            let token = matches.value_of("token").unwrap();
            remove_session(pool, token)?;
        },
        (CMD_LIST, _) => {
            let sessions = list_session(pool)?;
            for (token, uid) in sessions {
                println!("Token: {:20} uid: {:20}", token, uid);
            }
        },
        (CMD_ADD, Some(matches)) => {
            let token = matches.value_of("token").unwrap();
            let uid = matches.value_of("uid").unwrap();
            add_session(pool, token, uid)?
        },
        _ => {
            matches.usage();
        }
    }
    Ok(())
}

fn add_session(pool: r2d2::Pool<RedisConnectionManager>, token:&str, uid: &str) -> Result<(), Error> {
    let conn = pool.get()?;
    conn.hset(SESSIONS, token, uid).map_err(From::from)
}

fn remove_session(pool: r2d2::Pool<RedisConnectionManager>, token: &str) -> Result<(), Error> {
    let conn = pool.get()?;
    conn.hdel(SESSIONS, token).map_err(From::from)
}

fn list_session(pool: r2d2::Pool<RedisConnectionManager>) -> Result<HashMap<String, String>, Error> {
    let conn = pool.get()?;
    conn.hgetall(SESSIONS).map_err(From::from)
}
