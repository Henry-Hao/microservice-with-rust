#[macro_use]
extern crate diesel;


use diesel::{
    insert_into, pg::PgConnection, Connection, ExpressionMethods,
    OptionalExtension, QueryDsl, RunQueryDsl,
};
use failure::{ format_err, Error };

pub mod models;
pub mod schema;

use self::models::{ User, Channel, Membership, Message, Id };
use self::schema::{ users, channels, memberships, messages };
use std::env;

pub struct Api {
    conn: PgConnection
}

impl Api {
    pub fn connect() -> Result<Self, Error> {
        let database_uri = env::var("DATABASE_URL")
            .unwrap_or("postgres://postgres@localhost:5432".to_string());
        let conn = PgConnection::establish(&database_uri)?;
        Ok(Self {conn})
    }

    pub fn register_user(&self, email: &str) -> Result<User, Error> {
        insert_into(users::table)
            .values(users::email.eq(email))
            .returning((users::id, users::email))
            .get_result(&self.conn)
            .map_err(Error::from)
    }

    pub fn create_channel(&self, user_id: Id, title: &str, is_public: bool) -> Result<Channel, Error> {
        insert_into(channels::table)
            .values((
                    channels::user_id.eq(user_id),
                    channels::title.eq(title),
                    channels::is_public.eq(is_public),
                    channels::created_at.eq(diesel::dsl::now),
                    channels::updated_at.eq(diesel::dsl::now)
                    ))
            .returning((
                    channels::id,
                    channels::user_id,
                    channels::title,
                    channels::is_public,
                    channels::created_at,
                    channels::updated_at
                    ))
            .get_result(&self.conn)
            .map_err(From::from)
    }

    pub fn publish_channel(&self, channel_id: Id) -> Result<(), Error> {
        let channel = channels::table
            .filter(channels::id.eq(channel_id))
            .select((
                    channels::id,
                    channels::user_id,
                    channels::title,
                    channels::is_public,
                    channels::created_at,
                    channels::updated_at
                    ))
            .first::<Channel>(&self.conn)
            .optional()
            .map_err(Error::from)?;
        if let Some(channel) = channel {
            diesel::update(&channel)
                .set(channels::is_public.eq(true))
                .execute(&self.conn)?;
            Ok(())
        } else {
            Err(format_err!("channel not found"))
        }
    }

    pub fn add_member(&self, user_id: Id, channel_id: Id) -> Result<Membership, Error> {
        insert_into(memberships::table)
            .values((
                    memberships::channel_id.eq(channel_id),
                    memberships::user_id.eq(user_id)
                    ))
            .returning((
                    memberships::id,
                    memberships::user_id,
                    memberships::channel_id
                    ))
            .get_result(&self.conn)
            .map_err(Error::from)
    }

    pub fn add_message(&self, channel_id: Id, user_id: Id, text: &str) -> Result<Message, Error> {
        insert_into(messages::table)
            .values((
                    messages::timestamp.eq(diesel::dsl::now),
                    messages::channel_id.eq(channel_id),
                    messages::user_id.eq(user_id),
                    messages::text.eq(text)
                    ))
            .returning((
                    messages::id,
                    messages::timestamp,
                    messages::user_id,
                    messages::channel_id,
                    messages::text
                    ))
            .get_result(&self.conn)
            .map_err(Error::from)
    }

    pub fn delete_message(&self, message_id: Id) -> Result<(), Error> {
        diesel::delete(messages::table)
            .filter(messages::id.eq(message_id))
            .execute(&self.conn)?;
        Ok(())
    }
}


fn main() {
    print!("hello world");
}

#[cfg(test)]
mod test {
    use super::Api;

    #[test]
    fn create_users() {
        let api = Api::connect().unwrap();
        let user_1 = api.register_user("user1@example.com").unwrap();
        let user_2 = api.register_user("user2@example.com").unwrap();
        let channel = api.create_channel(user_1.id, "My Channel", false).unwrap();
        api.publish_channel(channel.id).unwrap();
        api.add_member(user_2.id, channel.id).unwrap();
        let message = api.add_message(channel.id, user_2.id, "Hello world from user2").unwrap();
        api.add_message(channel.id, user_1.id, "Reply from user1").unwrap();
        api.delete_message(message.id).unwrap();

    }
}
