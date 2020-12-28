#![allow(unused_must_use)]
use clap::{
    crate_name, crate_authors, crate_version, crate_description,
    App, AppSettings, Arg, SubCommand
};
use chrono::Utc;
use failure::{Error, format_err};
use rusoto_core::Region;
use rusoto_dynamodb::{
    AttributeValue, DynamoDb, DynamoDbClient, QueryInput, UpdateItemInput
};
use std::collections::HashMap;

#[derive(Debug)]
struct Location {
    user_id: String,
    timestamp: String,
    longitude: String,
    latitude: String
}

impl Location {
    fn from_map(map: HashMap<String, AttributeValue>) -> Result<Location, Error> {
        let user_id = map
            .get("Uid")
            .ok_or_else(|| format_err!("No Uid in record"))
            .and_then(attr_to_string)?;
        let timestamp = map
            .get("TimeStamp")
            .ok_or_else(|| format_err!("No TimeStamp in record"))
            .and_then(attr_to_string)?;
        let latitude = map
            .get("Latitude")
            .ok_or_else(|| format_err!("No Latitude in record"))
            .and_then(attr_to_string)?;
        let longitude = map
            .get("Longitude")
            .ok_or_else(|| format_err!("No Longitude in record"))
            .and_then(attr_to_string)?;
        let location = Location {
            user_id,
            timestamp,
            latitude,
            longitude
        };
        Ok(location)
    }
}

fn add_location(conn: &DynamoDbClient, location: Location) -> Result<(), Error> {
    let mut key: HashMap<String, AttributeValue> = HashMap::new();
    key.insert("Uid".into(), s_attr(location.user_id));
    key.insert("TimeStamp".into(), s_attr(location.timestamp));
    let expression = format!("SET Latitude = :y, Longitude = :x");
    let mut values = HashMap::new();
    values.insert(":y".into(), s_attr(location.latitude));
    values.insert(":x".into(), s_attr(location.longitude));
    let update = UpdateItemInput {
        table_name: "Locations".into(),
        key,
        update_expression: Some(expression),
        expression_attribute_values: Some(values),
        ..Default::default()
    };
    conn.update_item(update)
        .sync()
        .map(drop)
        .map_err(Error::from)

}

fn list_locations(conn: &DynamoDbClient, user_id: String) -> Result<Vec<Location>, Error> {
    let expression = format!("Uid = :uid");
    let mut values = HashMap::new();
    values.insert(":uid".into(), s_attr(user_id));
    let query = QueryInput {
        table_name: "Locations".into(),
        key_condition_expression: Some(expression),
        expression_attribute_values: Some(values),
        ..Default::default()
    };
    let items = conn.query(query)
        .sync()?
        .items
        .ok_or_else(|| format_err!("No items"))?;

    let mut locations = Vec::new();
    for item in items {
        let location = Location::from_map(item)?;
        locations.push(location);
    }

    Ok(locations)
}

fn s_attr(s: String) -> AttributeValue {
    AttributeValue {
        s: Some(s),
        ..Default::default()
    }
}

fn attr_to_string(attr: &AttributeValue) -> Result<String, Error> {
    if let Some(value) = &attr.s {
        Ok(value.to_owned())
    } else {
        Err(format_err!("no string value"))
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
        .arg(Arg::with_name("region")
             .short("r")
             .long("region")
             .value_name("REGION")
             .help("Set the region of the AWS account")
             .takes_value(true))
        .arg(Arg::with_name("endpoint")
             .short("e")
             .long("endpoint")
             .value_name("URL")
             .help("Set the endpoint of the AWS account")
             .takes_value(true))
        .subcommand(SubCommand::with_name(CMD_ADD).about("create a location record")
                    .arg(Arg::with_name("USER_ID")
                         .help("Set the userid")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("LATITUDE")
                         .help("Set the latitude")
                         .required(true)
                         .index(2))
                    .arg(Arg::with_name("LONGITUDE")
                         .help("Set the longitude")
                         .required(true)
                         .index(3)))
        .subcommand(SubCommand::with_name(CMD_LIST).about("print list of locations")
                    .arg(Arg::with_name("USER_ID")
                         .help("user id to filter records")
                         .required(true)
                         .index(1)))
        .get_matches();
    let region = matches.value_of("endpoint").map(|endpoint| {
        Region::Custom {
            name: "custom".into(),
            endpoint: endpoint.into()
        }
    })
    .ok_or_else(||format_err!("Region not set"))
    .or_else(|_| {
        matches.value_of("region")
            .unwrap_or("us-east-1")
            .parse()
    })?;
    let client = DynamoDbClient::new(region);
    match matches.subcommand() {
        (CMD_LIST, Some(matches)) => {
            let user_id = matches.value_of("USER_ID").unwrap().to_string();
            for location in list_locations(&client, user_id)? {
                println!("{:?}", location);
            }
        },
        (CMD_ADD, Some(matches)) => {
            let user_id = matches.value_of("USER_ID").unwrap().to_string();
            let timestamp = Utc::now().to_string();
            let latitude = matches.value_of("LATITUDE").unwrap().to_string();
            let longitude = matches.value_of("LONGITUDE").unwrap().to_string();
            let location = Location {
                user_id,
                timestamp,
                latitude,
                longitude
            };
            add_location(&client, location)?

        },
        _ => {
            matches.usage();
        }
    }
    Ok(())
}

