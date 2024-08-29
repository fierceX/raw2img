use juniper::{EmptySubscription, RootNode};
use juniper::graphql_object;
use juniper::{
    graphql_value, FieldError, FieldResult,
};
use chrono::prelude::*;
use rusqlite::params;
use crate::db::Pool;

use super::image::Image;
use super::storage::{Storage, StorageInput};
use super::user::{User, UserInput};
use super::lut::Lut;
pub struct Context {
    pub db_pool: Pool,
}

impl juniper::Context for Context {}

pub struct QueryRoot;

#[graphql_object(Context = Context)]
impl QueryRoot {
    #[graphql(description = "List of all users")]
    fn users(context: &Context) -> FieldResult<Vec<User>> {
        let conn = context.db_pool.get().unwrap();

        // let res = conn.execute(sql, params).unwrap();
        let mut res = conn.prepare("select * from users;").unwrap();
        let users:Vec<User> = res.query_map([],|row| {
            Ok(User {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                email: row.get(2).unwrap(),
                wb: row.get(4).unwrap(),
                half_size: row.get(5).unwrap(),
                quality: row.get(6).unwrap(),
                lut_id: row.get(7).unwrap_or(-1),
            })
        }).unwrap().into_iter().filter_map(Result::ok).collect();

        Ok(users)

    }

    #[graphql(description = "Get Single user reference by user ID")]
    fn user(context: &Context, id: String) -> FieldResult<User> {
        let conn = context.db_pool.get().unwrap();

        let res = conn.query_row("select * from users where id = :id;", &[(":id",&id)], |row|{
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
            Err(FieldError::new(
                        "User Not Found",
                        graphql_value!({ "not_found": "user not found" }),
                    ))
        }
        else{
            Ok(res.unwrap())
        }
    }

    #[graphql(description = "List of all storages")]
    fn storages(context: &Context) -> FieldResult<Vec<Storage>> {
        let conn = context.db_pool.get().unwrap();

        let mut res = conn.prepare("select * from storages;").unwrap();
        let products:Vec<Storage> = res.query_map([],|row| {
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
        }).unwrap().into_iter().filter_map(Result::ok).collect();

        Ok(products)

    }

    #[graphql(description = "Get Single storage reference by storage ID")]
    fn storage(context: &Context, id: String) -> FieldResult<Storage> {
        let conn = context.db_pool.get().unwrap();

        let res = conn.query_row("select * from storages where id = :id;", &[(":id",&id)], |row|{
            Ok(Storage {
                id: row.get(0).unwrap(),
                user_id: row.get(1).unwrap(),
                storage_name: row.get(2).unwrap(),
                storage_path: row.get(3).unwrap(),
                storage_type: row.get(4).unwrap(),
                storage_url: row.get(5).unwrap(),
                access_key: row.get(6).unwrap(),
                secret_key: row.get(7).unwrap(),
                bucket_name: row.get(8).unwrap(),
                added_time: row.get(9).unwrap(),
                storage_usage: row.get(10).unwrap(),
            })
        });
        if let Err(_err) = res{
            Err(FieldError::new(
                        "Product Not Found",
                        graphql_value!({ "not_found": "product not found" }),
                    ))
        }
        else{
            // Some(res.unwrap())
            Ok(res.unwrap())
        }
    }

    #[graphql(description = "Get Single image reference by user ID")]
    fn image(context: &Context, id: String) -> FieldResult<Image> {
        let conn = context.db_pool.get().unwrap();

        let res = conn.query_row("select * from images_view where id = :id;", &[(":id",&id)], |row|{
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
        });
        if let Err(_err) = res{
            Err(FieldError::new(
                        "Product Not Found",
                        graphql_value!({ "not_found": "product not found" }),
                    ))
        }
        else{
            // Some(res.unwrap())
            Ok(res.unwrap())
        }
    }
    
    #[graphql(description = "List of all Luts")]
    fn luts(context: &Context) -> FieldResult<Vec<Lut>> {
        let conn = context.db_pool.get().unwrap();

        let mut res = conn.prepare("select luts.id,lut_name,storages.storage_path as path,comment from luts left join storages on luts.storage_id = storages.id;").unwrap();
        let users:Vec<Lut> = res.query_map([],|row| {
            Ok(Lut {
                id: row.get(0).unwrap(),
                lut_name: row.get(1).unwrap(),
                path: row.get(2).unwrap(),
                comment: row.get(2).unwrap(),
            })
        }).unwrap().into_iter().filter_map(Result::ok).collect();

        Ok(users)

    }

    #[graphql(description = "Get Single lut reference by lut ID")]
    fn lut(context: &Context, id: String) -> FieldResult<Lut> {
        let conn = context.db_pool.get().unwrap();

        let res = conn.query_row("select id,lut_name,storages.storage_path as path,comment from luts left join storages on luts.storage_id = storages.id where luts.id = :id;", &[(":id",&id)], |row|{
            Ok(Lut {
                id: row.get(0).unwrap(),
                lut_name: row.get(1).unwrap(),
                path: row.get(2).unwrap(),
                comment: row.get(2).unwrap(),
            })
        });
        if let Err(_err) = res{
            Err(FieldError::new(
                        "Product Not Found",
                        graphql_value!({ "not_found": "product not found" }),
                    ))
        }
        else{
            // Some(res.unwrap())
            Ok(res.unwrap())
        }
    }
  
}

pub struct MutationRoot;

#[graphql_object(Context = Context)]
impl MutationRoot {
    // fn create_user(context: &Context, user: UserInput) -> FieldResult<User> {
    //     let conn = context.db_pool.get().unwrap();
    //     // let new_id = uuid::Uuid::new_v4().simple().to_string();
    //     let res = conn.execute(
    //         "INSERT INTO users (username, email,password) VALUES (?1, ?2, ?3)",
    //         (&user.name, &user.email, &user.password),
    //     );
    //     match res {
    //         Ok(_) =>{
    //             let _id = conn.last_insert_rowid();
    //             Ok(User{id:_id as i32,name:user.name,email:user.email})
    //         }
    //         Err(msg) =>{
    //             Err(FieldError::new(
    //                             "Failed to create new user",
    //                             graphql_value!({ "internal_error": msg.to_string() }),
    //                         ))
    //         }
    //     }
    // }

    fn create_storage(context: &Context, storage: StorageInput) -> FieldResult<Storage> {
        let conn = context.db_pool.get().unwrap();
        let now: DateTime<Utc> = Utc::now();

        // 格式化时间
        let formatted_time = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let res = conn.execute(
            "INSERT INTO storages (user_id,storage_name,storage_path,storage_type,storage_url,added_time,storage_usage) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (&storage.user_id,&storage.storage_name, &storage.storage_path,&storage.storage_type,&storage.storage_url,&formatted_time,&storage.storage_usage),
        );
        match res {
            Ok(_) =>{
                let _id = conn.last_insert_rowid();
                Ok(Storage {
                    id: _id as i32,
                    user_id:storage.user_id,
                    storage_name:storage.storage_name,
                    storage_path:storage.storage_path,
                    storage_type:storage.storage_type,
                    storage_url:storage.storage_url,
                    access_key:storage.access_key,
                    secret_key:storage.secret_key,
                    bucket_name:storage.bucket_name,
                    added_time:formatted_time,
                    storage_usage:storage.storage_usage,
                })
            }
            Err(msg) =>{
                Err(FieldError::new(
                                "Failed to create new user",
                                graphql_value!({ "internal_error": msg.to_string() }),
                            ))
            }
        }

        // let insert: Result<Option<Row>, DBError> = conn.exec_first(
        //     "INSERT INTO product(id, user_id, name, price) VALUES(:id, :user_id, :name, :price)",
        //     params! {
        //         "id" => &new_id,
        //         "user_id" => &product.user_id,
        //         "name" => &product.name,
        //         "price" => &product.price.to_owned(),
        //     },
        // );

        // match insert {
        //     Ok(_opt_row) => Ok(Product {
        //         id: new_id,
        //         user_id: product.user_id,
        //         name: product.name,
        //         price: product.price,
        //     }),
        //     Err(err) => {
        //         let msg = match err {
        //             DBError::MySqlError(err) => err.message,
        //             _ => "internal error".to_owned(),
        //         };
        //         Err(FieldError::new(
        //             "Failed to create new product",
        //             graphql_value!({ "internal_error": msg }),
        //         ))
        //     }
        // }
    }

    fn update_user(context: &Context, user: UserInput, id:String) -> FieldResult<User>{
        let conn = context.db_pool.get().unwrap();
        
        let res = conn.execute(
            "UPDATE users SET wb = ?2, half_size = ?3, quality = ?4, lut_id = ?5 where id = ?1",
            (&id,&user.wb,&user.half_size,&user.quality,&user.lut_id),
        );
        match res {
            Ok(_) =>{
                // let _id = conn.last_insert_rowid();
                Ok(
                    User{
                        id: id.parse().unwrap(),
                        name: user.name,
                        email: user.email,
                        wb: user.wb,
                        half_size: user.half_size,
                        quality: user.quality,
                        lut_id: user.lut_id,
                    }
                )
            }
            Err(msg) =>{
                Err(FieldError::new(
                                "Failed to create new user",
                                graphql_value!({ "internal_error": msg.to_string() }),
                            ))
            }
        }
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, EmptySubscription::new())
}