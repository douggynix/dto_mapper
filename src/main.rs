//https://code.visualstudio.com/docs/languages/rust
extern crate dto_mapper;



use dto_mapper::DtoMapper;

#[derive(DtoMapper,Debug)]
#[mapper(map=[(" username: login",true)],
derive=(Debug,Default,Eq), 
dto="ProfileDto", include_all=true, except=["password", "age"])]
#[mapper(dto="LoginDto" , map=[("username",true),("password",true)])]
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
