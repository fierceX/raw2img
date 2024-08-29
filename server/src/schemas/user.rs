use juniper::{graphql_object, GraphQLInputObject};
use crate::schemas::{root::Context,image::Image,storage::Storage};


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
            Ok(Storage {
                id: row.get(0).unwrap(),
                user_id: row.get(1).unwrap(),
                storage_name: row.get(2).unwrap(),
                storage_path: row.get(3).unwrap_or("".to_string()),
                storage_type: row.get(4).unwrap(),
                storage_url: row.get(5).unwrap(),
                access_key: row.get(6).unwrap_or("".to_string()),
                secret_key: row.get(7).unwrap_or("".to_string()),
                bucket_name: row.get(8).unwrap_or("".to_string()),
                added_time: row.get(9).unwrap(),
                storage_usage: row.get(10).unwrap(),
            })
        }).unwrap().into_iter().filter_map(Result::ok).collect()
    }
    fn images(&self, context: &Context) -> Vec<Image> {
        let conn = context.db_pool.get().unwrap();

        let mut res = conn.prepare("select * from images_view where user_id = :user_id;").unwrap();
        res.query_map(&[(":user_id",&self.id)],|row| {
            Ok(Image {
                id: row.get(0).unwrap(),
                user_id: row.get(1).unwrap(),
                file_name: row.get(2).unwrap(),
                cache_file_name: row.get(3).unwrap_or("".to_string()),
                scan_time: row.get(4).unwrap(),
                file_size: row.get(5).unwrap(),
                mime_type: row.get(6).unwrap(),
                exif: row.get(7).unwrap_or("".to_string()),
                original_url: row.get(8).unwrap(),
                cached_url: row.get(9).unwrap_or("".to_string()),
            })
        }).unwrap().into_iter().filter_map(Result::ok).collect()

    }

}