use serde_derive::Serialize;
use super::schema::users;

#[derive(Serialize, Debug, Queryable)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUsers<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub email: &'a str,
}
