//https://code.visualstudio.com/docs/languages/rust
extern crate dto_mapper;
mod my_dto;

//use my_dto::LoginDto;
use my_dto::{LoginDto, ProfileDto};

use crate::my_dto::User;


fn main() {
    //show_answer();

    let mut lg = LoginDto::default();
    lg.login = "user123".to_string();
    lg.password = "pass123".to_string();

    
    let mut pf = ProfileDto::default();
    pf.login ="user_pf".into();
    

    println!("dto={:?}",lg);
    println!("ProfileDto(not derive Default nor Debug) login={} , email={:?}",pf.login,pf.email);

    let user_from_lg : User = lg.into();
    println!("user_from_lg={:?}",user_from_lg);

    let user_from_pf : User = pf.into();
    println!("user_from_pf={:?}",user_from_pf);

    let user = User::new("user".into(), "password".into(), "user@mail.org".into(), 25);
    println!("{:?}",user);
    let lg_from_user : LoginDto = user.clone().into();

    println!("lg_from_user={:?}",lg_from_user);
    let pf_from_user : ProfileDto = user.clone().into();
    println!("pf_from_user= name={:?} age={:?}", pf_from_user.name, pf_from_user.age);
}
