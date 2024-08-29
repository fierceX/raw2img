use juniper::{graphql_object, GraphQLInputObject};
use crate::schemas::{root::Context,user::User};


#[derive(Default, Debug)]
pub struct Image {
    pub id: i32,
    pub user_id: i32,
    pub original_url: String,
    pub cached_url: String,
    pub file_name: String,
    pub cache_file_name: String,
    pub scan_time: String,
    pub file_size: i32,
    pub mime_type: String,
    pub exif:String,
}

#[juniper::graphql_object(Context = Context)]
impl Image {
    fn id(&self) -> &i32 {
        &self.id
    }
    fn user_id(&self) -> &i32 {
        &self.user_id
    }

    fn original_url(&self) -> &str {
        &self.original_url
    }
    fn cached_url(&self) -> &str {
        &self.cached_url
    }
    fn file_name(&self) -> &str {
        &self.file_name
    }
    fn cache_file_name(&self) -> &str {
        &self.cache_file_name
    }
    fn scan_time(&self) -> &str {
        &self.scan_time
    }
    fn file_size(&self) -> &i32 {
        &self.file_size
    }
    fn mime_type(&self) -> &str {
        &self.mime_type
    }
    fn exif(&self) -> &str {
        &self.exif
    }

    fn user(&self, context: &Context) -> Option<User> {
        let conn = context.db_pool.get().unwrap();

        let res = conn.query_row("select * from users where id = :id;", &[(":id",&self.user_id)], |row|{
            Ok(User{
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                email: row.get(2).unwrap(),
                wb: row.get(4).unwrap(),
                half_size: row.get(5).unwrap(),
                quality: row.get(6).unwrap(),
                lut_id: row.get(7).unwrap_or(-1),
            })
        });
        if let Err(_err) = res{
            None
        }
        else{
            Some(res.unwrap())
        }
    }
}