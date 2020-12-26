#![allow(unused_must_use)]
use clap::{
    crate_name, crate_authors, crate_version, crate_description,
    App, AppSettings, Arg, SubCommand
};
use chrono::offset::Utc;
use r2d2_mongodb::{ ConnectionOptions, MongodbConnectionManager };
use mongodb::db::ThreadedDatabase;
use bson::{ doc, bson };
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
struct Activity {
    user_id: String,
    activity: String,
    datetime: String
}

#[derive(Debug)]
enum Error {
    CliError(std::num::ParseIntError),
    DbError(mongodb::error::Error),
    PoolError(r2d2::Error),
    DecoderError(mongodb::DecoderError)
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
            Error::DecoderError(err) => {
                write!(f, "DecoderError:{}", err)
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
impl From<mongodb::error::Error> for Error {
    fn from(err:mongodb::error::Error) -> Self {
        Error::DbError(err)
    }
}
impl From<r2d2::Error> for Error {
    fn from(err:r2d2::Error) -> Self {
        Error::PoolError(err)
    }
}
impl From<mongodb::DecoderError> for Error {
    fn from(err:mongodb::DecoderError) -> Self {
        Error::DecoderError(err)
    }
}

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
        .subcommand(SubCommand::with_name(CMD_ADD).about("create a session record")
                    .arg(Arg::with_name("USER_ID")
                         .help("Set the userid")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("ACTIVITY")
                         .help("Set the activity")
                         .required(true)
                         .index(2)))
        .subcommand(SubCommand::with_name(CMD_LIST).about("print list of activities"))
        .get_matches();
    let host = matches.value_of("host").or(Some("127.0.0.1")).unwrap();
    let port = matches.value_of("port").unwrap_or("27017").parse::<u16>()?;
    let database = matches.value_of("database").unwrap_or("admin");
    let manager = MongodbConnectionManager::new(
        ConnectionOptions::builder()
        .with_host(host, port)
        .with_db(database)
        .build()
        );
    let pool = r2d2::Pool::builder().max_size(4).build(manager)?;
    match matches.subcommand() {
        (CMD_LIST, _) => {
            let activities = list_activity(pool)?;
            for activity in activities {
                println!("User:{:20}, Activity: {:20}, Datetime: {:20}",
                         activity.user_id, activity.activity, activity.datetime);
            }

        },
        (CMD_ADD, Some(matches)) => {
            let user_id = matches.value_of("USER_ID").unwrap().to_string();
            let activity = matches.value_of("ACTIVITY").unwrap().to_string();
            let activity = Activity {
                user_id,
                activity,
                datetime: Utc::now().to_string()
            };
            add_activity(pool, activity);
        },
        _ => {
            matches.usage();
        }
    }
    Ok(())
}

fn add_activity(pool: r2d2::Pool<MongodbConnectionManager>, activity: Activity) -> Result<(), Error> {
    let doc = doc! {
        "user_id": activity.user_id,
        "activity": activity.activity,
        "datetime": activity.datetime
    };
    let coll = pool.get()?.collection("activities");
    coll.insert_one(doc, None).map(drop).map_err(From::from)
}

fn list_activity(pool: r2d2::Pool<MongodbConnectionManager>) -> Result<Vec<Activity>, Error> {
    let coll = pool.get()?.collection("activities");
    coll.find(None,None)?
        .try_fold(Vec::new(), |mut vec, doc| {
            let doc = doc?;
            let activity: Activity = bson::from_bson(bson::Bson::Document(doc))?;
            vec.push(activity);
            Ok(vec)
        })
}

