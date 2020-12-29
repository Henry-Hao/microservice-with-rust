use crate::schema::{channels, memberships, messages, users};
use chrono::NaiveDateTime;
// use chrono::naive::NaiveDateTime;
use serde::{Deserialize, Serialize};

pub type Id = i32;

#[derive(Serialize, Deserialize, Debug, Queryable, Identifiable)]
#[table_name = "users"]
pub struct User {
    pub id: Id,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Queryable, Identifiable, Associations)]
#[table_name = "channels"]
#[belongs_to(User)]
pub struct Channel {
    pub id: Id,
    pub user_id: Id,
    pub title: String,
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Queryable, Identifiable, Associations)]
#[table_name = "memberships"]
#[belongs_to(User)]
#[belongs_to(Channel)]
pub struct Membership {
    pub id: Id,
    pub channel_id: Id,
    pub user_id: Id,
}

#[derive(Serialize, Deserialize, Debug, Queryable, Identifiable, Associations)]
#[table_name = "messages"]
#[belongs_to(User)]
#[belongs_to(Channel)]
pub struct Message {
    pub id: Id,
    pub timestamp: NaiveDateTime,
    pub channel_id: Id,
    pub user_id: Id,
    pub text: String,
}
