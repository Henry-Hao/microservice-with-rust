#![allow(unused_must_use)]
use clap::{
    crate_name, crate_authors, crate_version, crate_description,
    App, AppSettings, Arg, SubCommand
};
use postgres:: {
    Error as DbError, NoTls, Client, Config
};

#[derive(Debug)]
enum Error {
    CliError(std::num::ParseIntError),
    DbError(DbError)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CliError(err) => {
                write!(f, "CliError:{}", err)
            },
            Error::DbError(err) => {
                write!(f, "DbError:{}", err)
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



const CMD_CREATE: &str = "create";
const CMD_ADD: &str = "add";
const CMD_LIST: &str = "list";


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
        .get_matches();
    let host = matches.value_of("host").unwrap_or("localhost");
    let port = matches.value_of("port").unwrap_or("5432");
    let database = matches.value_of("database").unwrap_or("postgres");
    let mut conn = Config::new()
        .host(host)
        .port(port.parse::<u16>()?)
        .user(database)
        .connect(NoTls)?;
    match matches.subcommand() {
        (CMD_CREATE, _) => {
            create_table(&mut conn)?;
        },
        (CMD_LIST, _) => {
            let users = list_users(&mut conn)?;
            for (name, email) in users {
                println!("Name: {:20} Email: {:20}", name, email);
            }
        },
        (CMD_ADD, Some(matches)) => {
            let name = matches.value_of("name").unwrap();
            let email = matches.value_of("email").unwrap();
            create_user(&mut conn, name, email)?;
        },
        _ => {
            matches.usage();
        }
    }
    Ok(())
}

fn create_table(conn: &mut Client) -> Result<(), Error> {
    conn.execute("CREATE TABLE users (
                        id SERIAL PRIMARY KEY,
                        name VARCHAR NOT NULL,
                        email VARCHAR NOT NULL)", &[])
        .map(drop)
        .map_err(From::from)
}


fn create_user(conn: &mut Client, name: &str, email: &str) -> Result<(), Error> {
    conn.execute("INSERT INTO users (name, email) VALUES ($1, $2)", &[&name, &email])
        .map(drop)
        .map_err(From::from)
}

fn list_users(conn: &mut Client) -> Result<Vec<(String, String)>, Error> {
    let res = conn.query("SELECT name, email FROM users", &[])?.into_iter()
        .map(|row|(row.get(0), row.get(1)))
        .collect();
    Ok(res)
}
