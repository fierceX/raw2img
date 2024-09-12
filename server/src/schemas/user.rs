use juniper::{graphql_object, GraphQLInputObject};
use crate::schemas::{root::Context,image::Image,image::row2img,storage::Storage,storage::row2storage};
use rusqlite::Error;

#[derive(Default, Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub lut_id: i32,
    pub wb: bool,
    pub half_size: bool,
    pub quality: i32,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "User Input")]
pub struct UserInput {
    pub name: String,
    pub email: String,
    pub password: String,
    pub lut_id: i32,
    pub wb: bool,
    pub half_size: bool,
    pub quality: i32,
}

#[graphql_object(Context = Context)]
impl User {
    fn id(&self) -> &i32 {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn email(&self) -> &str {
        &self.email
    }
    fn lut_id(&self) -> &i32{
        &self.lut_id
    }
    fn wb(&self) -> &bool{
        &self.wb
    }
    fn half_size(&self) -> &bool{
        &self.half_size
    }
    fn quality(&self) -> &i32{
        &self.quality
    }

    fn storages(&self, context: &Context) -> Vec<Storage> {
        let conn = context.db_pool.get().unwrap();

        let mut res = conn.prepare("SELECT * FROM storages WHERE user_id = :user_id").unwrap();
        res.query_map(&[(":user_id", &self.id)],|row| {
            row2storage(row)
        }).unwrap().into_iter().filter_map(Result::ok).collect()
    }
    fn images(&self, context: &Context) -> Vec<Image> {
        let conn = context.db_pool.get().unwrap();

        let mut res = conn.prepare("select * from images_view where user_id = :user_id;").unwrap();
        res.query_map(&[(":user_id",&self.id)],|row| {
            row2img(row)
        }).unwrap().into_iter().filter_map(Result::ok).collect()
    }
}

pub fn row2user(row:&rusqlite::Row<'_>) -> Result<User, Error>{
    Ok(User{
        id: row.get(0).unwrap(),
        name: row.get(1).unwrap(),
        email: row.get(2).unwrap(),
        wb: row.get(4).unwrap(),
        half_size: row.get(5).unwrap(),
        quality: row.get(6).unwrap(),
        lut_id: row.get(7).unwrap_or(-1),
    })
}