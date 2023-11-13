//https://code.visualstudio.com/docs/languages/rust
extern crate dto_mapper;



use dto_mapper::DtoMapper;

#[derive(DtoMapper,Debug)]
#[mapper(dto="ProfileDto", map=[("username: login",true), ("password",true)], derive=(Debug,Default,Eq) )]
#[mapper(dto="LoginDto" , map=[("username:login",true),("firstname:name",false)] , ignore=["password", "age"]) ]
struct User{
    username: String,
    password: String,
    email : Option<String>,
    firstname: Option<String>,
    lastname: Option<String>,
    age : u8,
}

fn main() {
    //show_answer();
}
