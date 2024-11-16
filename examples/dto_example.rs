use derive_builder::Builder;
use dto_mapper::DtoMapper;
use serde::{Deserialize, Serialize};
#[allow(unused)]
use std::str::FromStr;
#[allow(unused)]

fn concat_str(s1: &str, s2: &str) -> String {
    s1.to_owned() + " " + s2
}

#[derive(DtoMapper, Debug, Default, Clone)]
#[mapper( dto="LoginDto"  , map=[ ("username:login",true) , ("password",true)] , derive=(Debug, Clone, PartialEq) )]
#[mapper( dto="ProfileDto" , ignore=["password"]  , derive=(Debug, Clone, PartialEq) )]
#[mapper( dto="PersonDto" , no_builder=true , map=[ ("firstname",true), ("lastname",true), ("email",false) ]  )]
#[mapper( dto="CustomDto" , no_builder=true , map=[ ("email",false) ] , derive=(Debug, Clone) ,
  new_fields=[( "name: String", "concat_str( self.firstname.as_str(), self.lastname.as_str() )" )]
)]
#[mapper(
    dto="CustomDtoWithAttribute" ,
    no_builder=true ,
    map=[ ("email", false, ["#[serde(rename = \"email_address\")]"] ) ],
    derive=(Debug, Clone, Serialize, Deserialize),
    new_fields=[
        (
            "name: String",
            "concat_str( self.firstname.as_str(), self.lastname.as_str() )",
            ["#[serde(rename = \"full_name\")]"],
        ),
        (
          "hidden_password: String",
         r#""*".repeat( self.password.len() )"#
        ),
    ],
    macro_attr=["serde(rename_all = \"UPPERCASE\")"]
)]
struct User {
    username: String,
    password: String,
    email: String,
    firstname: String,
    middle_name: Option<String>,
    lastname: String,
    age: u8,
}

fn main() {
    let user = User {
        firstname: "Dessalines".into(),
        lastname: "Jean Jacques".into(),
        email: "dessalines@gmail.com".into(),
        password: "password123".into(),
        ..User::default()
    };

    println!("{:?}", user);
    let custom_dto: CustomDto = user.clone().into();
    println!("{:?}", custom_dto);

    let custom_dto_attributes: CustomDtoWithAttribute = user.clone().into();
    println!("{:?}", custom_dto_attributes);
}
