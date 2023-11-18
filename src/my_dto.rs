extern crate dto_mapper;
use dto_mapper::DtoMapper;

#[derive(DtoMapper,Debug,Default,Clone)]
#[mapper(dto="LoginDto", map=[("username:login",true), ("password",true)], derive=(Debug,Default) )]
#[mapper(dto="ProfileDto" , ignore=["password"] , map=[("username:login",true),("firstname:name",false),("age",false)])]
pub struct User{
    username: String,
    password: String,
    email : Option<String>,
    firstname: Option<String>,
    lastname: Option<String>,
    age : u8,
}

impl User{
    pub fn new(username: String, password: String, email: String,age: u8) -> Self {
        Self{
            username,
            password,
            email: Some(email),
            age,
            ..Self::default()
        }
    }
}