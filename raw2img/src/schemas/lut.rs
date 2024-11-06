use juniper::{graphql_object, GraphQLInputObject};
use crate::schemas::{root::Context,user::User};


#[derive(Default, Debug)]
pub struct Lut {
    pub id: i32,
    pub lut_name: String,
    pub path: String,
    pub comment: String
}

#[juniper::graphql_object(Context = Context)]
impl Lut {
    fn id(&self) -> &i32 {
        &self.id
    }
    fn lut_name(&self) -> &str {
        &self.lut_name
    }

    fn path(&self) -> &str{
        &self.path
    }
    fn comment(&self) -> &str{
        &self.comment
    }
}