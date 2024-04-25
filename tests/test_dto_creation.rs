#[cfg(test)]
mod test_dto_creation {
    use derive_builder::Builder;
    use dto_mapper::DtoMapper;
    #[allow(unused)]
    use std::str::FromStr;
    #[allow(unused)]
    use unstringify::unstringify;

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
    struct User {
        username: String,
        password: String,
        email: String,
        firstname: String,
        middle_name: Option<String>,
        lastname: String,
        age: u8,
    }

    #[test]
    fn test_multiple_dto_creation() {
        let login_dto: LoginDto = LoginDto::default();
        let profile_dto: ProfileDto = ProfileDto::default();
        assert!(login_dto.login.is_empty());
        assert!(login_dto.password.is_empty());

        assert!(profile_dto.username.is_empty());
        assert!(profile_dto.email.is_empty());
        assert_eq!(0, profile_dto.age);
    }
    #[test]
    fn test_optional_creation() {
        let person_dto: PersonDto = PersonDto::default();
        //email field should be of Option type

        assert!(person_dto.email.is_some() || person_dto.email.is_none());
    }

    #[test]
    fn test_struct_into_dto() {
        let user = User {
            username: "dessalines".to_string(),
            email: "dessalines@mail.ht".to_string(),
            password: "XXXXXXXXXXXXX".into(),
            firstname: "Dessalines".to_string(),
            lastname: "Jean jacques".to_string(),
            age: 50,
            ..User::default()
        };

        //clone user as into moves user after into() operation in order to reuse user in subsequent calls
        let lg_dto_user: LoginDto = user.clone().into();
        let pf_dto_user: ProfileDto = user.clone().into();

        println!("User to LoginDto = {:?}", lg_dto_user);
        println!("User to ProfileDto = {:?}", pf_dto_user);
        assert_eq!(
            LoginDto {
                login: user.username.to_string(),
                password: user.password,
            },
            lg_dto_user
        );
        //values of user field is being moved only for the test scenario. if used further, use .to_string()
        assert_eq!(
            ProfileDto {
                username: user.username.to_string(),
                email: user.email.to_string(),
                firstname: user.firstname,
                middle_name: None,
                lastname: user.lastname,
                age: user.age,
            },
            pf_dto_user
        );
    }

    #[test]
    fn test_dto_into_struct() {
        let person = PersonDto {
            firstname: "Dessalines".to_string(),
            lastname: "Jean Jacques".to_string(),
            email: Some("dessalines@mail.ht".to_string()),
        };

        let user_from_person: User = person.into();

        assert_eq!("Dessalines", user_from_person.firstname);
        assert_eq!("Jean Jacques", user_from_person.lastname);
        assert_eq!("dessalines@mail.ht", user_from_person.email);

        //assert all the remaining fields are all initialized with default value from User::default
        assert_eq!(User::default().username, user_from_person.username);
        assert_eq!(User::default().password, user_from_person.password);
        assert_eq!(User::default().middle_name, user_from_person.middle_name);
        assert_eq!(User::default().age, user_from_person.age);
    }

    #[test]
    fn test_dto_with_builder() {
        let mut login_dto_builder = LoginDtoBuilder::default();
        let login_dto = login_dto_builder
            .login("capois-lamort".into())
            .password("hello123".into())
            .build()
            .expect("Failed to build login dto");
        println!("LoginDto built with a builder: {:?}", login_dto);
    }

    #[test]
    fn test_custom_dto_expression() {
        let user = User {
            firstname: "Dessalines".into(),
            lastname: "Jean Jacques".into(),
            ..User::default()
        };

        println!("{:?}", user);
        let custom_dto: CustomDto = user.into();
        println!("{:?}", custom_dto);

        //char::try_from(32);
    }
}
