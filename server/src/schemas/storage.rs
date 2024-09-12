use juniper::{graphql_object, GraphQLInputObject};
use crate::schemas::{root::Context,user::User};
use rusqlite::Error;


#[derive(Default, Debug)]
pub struct Storage {
    pub id: i32,
    pub user_id: i32,
    pub storage_name: String,
    pub storage_path: String,
    pub storage_type: String,
    pub storage_url: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub added_time: String,
    pub storage_usage: String,
}

#[juniper::graphql_object(Context = Context)]
impl Storage {
    fn id(&self) -> &i32 {
        &self.id
    }
    fn user_id(&self) -> &i32 {
        &self.user_id
    }

    fn storage_name(&self) -> &str{
        &self.storage_name
    }
    fn storage_path(&self) -> &str{
        &self.storage_path
    }
    fn storage_type(&self) -> &str{
        &self.storage_type
    }
    fn storage_url(&self) -> &str{
        &self.storage_url
    }
    fn access_key(&self) -> &str{
        &self.access_key
    }
    fn secret_key(&self) -> &str{
        &self.secret_key
    }
    fn bucket_name(&self) -> &str{
        &self.bucket_name
    }
    fn added_time(&self) -> &str{
        &self.added_time
    }
    fn storage_usage(&self) -> &str{
        &self.storage_usage
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

pub fn row2storage(row:&rusqlite::Row<'_>) -> Result<Storage,Error>{
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
}

#[derive(GraphQLInputObject)]
#[graphql(description = "Storage Input")]
pub struct StorageInput {
    pub user_id: i32,
    pub storage_name: String,
    pub storage_path: String,
    pub storage_type: String,
    pub storage_url: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub storage_usage: String,
}