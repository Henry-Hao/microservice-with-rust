use microservice_with_rust::schema::comments;
use diesel::SqliteConnection;
use diesel::{ RunQueryDsl, ExpressionMethods, QueryDsl };

#[derive(Serialize, Debug, Clone, Insertable, Queryable)]
#[table_name="comments"]
pub struct Comment {
    pub id: Option<i32>,
    pub uid: String,
    pub text: String
}


#[derive(FromForm)]
pub struct NewComment {
    pub uid: String,
    pub text: String
}


impl Comment {
    pub fn all(conn: &SqliteConnection) -> Vec<Comment> {
        comments::table
            .order(comments::id.desc())
            .load::<Comment>(conn)
            .unwrap()
    }


    pub fn insert(comment: NewComment, conn: &SqliteConnection) -> bool {
        let t = Comment {
            id: None,
            uid: comment.uid,
            text: comment.text
        };
        diesel::insert_into(comments::table).values(&t).execute(conn).is_ok()
    }
}
