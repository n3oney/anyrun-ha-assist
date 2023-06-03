use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::history)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct History {
    pub id: i32,
    pub query: String,
}

#[derive(Insertable)]
#[diesel(table_name=crate::schema::history)]
pub struct NewHistory<'a> {
    pub query: &'a str,
}
