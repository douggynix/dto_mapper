# dto_mapper
This is a library to create dynamic DTOs for rust based project. It has the same purpose with the known Java DTO Mapper Library called Model Java used mainly in Java SpringBoot applications.

DTO stands for Data Transfer Object. It is a Software Design pattern that involves mapping a parent object to a new one by reusing some of its properties into the newly created one.
For example, if one application is manipulating sensitive data such as credit card information, passwords, or any other high sensitive information that shouldn't be sent(serialized) over the network or 
displayed to the user in any form. So, applications need a way to map this Entity to a new one by removing the properties or fields they wouldn't want to expose to a user in JSON or any other format.

This library makes it handy. It helps annotate a structure with special attributes to extract fields to compose new structure objects for re-use. This library is based upon the **Syn** Library
and the power of macro derive attributes embedded in rust. The dtos are generated during compile time. There is no overhead during runtime for this as the final binary will contain the dto structure
resulted after buil time. I would recommend using Visual Studio code with rust analyzer plugin installed for auto-completion and a nicer developer experience with preview of field names when hovering 
over a dto structure.

# Summary
This library can be used by web applications which used to do data transformation between database and the controller that sends information to the user using DAO(Data Access Object) pattern.
Let's consider a User Entity used by an application to store into and retrieve records for a user from a database. Let's describe our User entity like this:
```rust
    struct User{
        username: String,
        password: String,
        email: String,
        firstname: String,
        middle_name: Option<String>,
        lastname: String,
        age: u8,
    }
```
If we have to load a 'user' record from a database, we wouldn't want to send all this information back to a webpage for different scenario. We would want to remove the **password** information from the user if we need to reuse it to send it to a webpage. One will insist that we can use Json Serializer Library and annotate the password field in such a way we can ignore it. Well, let's push it a little bit further. If we have a Backend application that serves client for Authentication request,Profile Information request, or to request other information in form of other data mix that would want only the firstname, lastname and the age of that person. Json annotation wouldn't help. And we would have to create by hands each of those Objects and repeated the same information and implement proper methods to convert one to another. That sounds like too much work.
This is where DTO Mapper Library comes handy.

# Installation
You can use this instruction to install the latest version of **dto_mapper** library to your project
```shell
cargo add dto_mapper
```
And import it to the rust source file you need to use it:
```rust
use dto_mapper::DtoMapper;
```

# Example
Let's say we want to create 3 special entities for our application
- LoginDto that will contain only 2 fields from **User** such as _**username**_ and _**password**_. we would like to rename _**username**_ to _**login**_ in LoginDto
- ProfileDto that will contain all fields from **User** and  will ignore only the **password** field.
- PersonDto that will contain only 3 fields from  **User** such as _**firstname**_, _**lastname**_, and _**email**_. But we would like to make the _**email**_ field optional such that its final data type will be  _**Option<T>**_. That is if email is **String** from User, it will result in _**Option<String>**_

It takes only those lines below to get this work done. And the conversion are being done automatically between one dto type to the original struct and vice versa.

  ```rust
    use dto_mapper::DtoMapper;

    #[derive(DtoMapper,Default,Clone)]
    #[mapper( dto="LoginDto"  , map=[ ("username:login",true) , ("password",true)] , derive=(Debug, Clone, PartialEq) )]
    #[mapper( dto="ProfileDto" , ignore=["password"]  , derive=(Debug, Clone, PartialEq) )]
    #[mapper( dto="PersonDto" , map=[ ("firstname",true), ("lastname",true), ("email",false) ]  )]
    struct User{
        username: String,
        password: String,
        email: String,
        firstname: String,
        middle_name: Option<String>,
        lastname: String,
        age: u8,
    }

  let login_dto : LoginDto = LoginDto::default();
  let profile_dto: ProfileDto = ProfileDto::default();
  ```

Let's consider we have a User struct value and we would like to convert it back to a dto object:
```rust
        let user = User{
            username : "dessalines".to_string(),
            email: "dessalines@mail.ht".to_string(),
            password: "XXXXXXXXXXXXX".into(),
            firstname: "Dessalines".to_string(),
            lastname: "Jean jacques".to_string(),
            age: 50,
            ..User::default()
        };

        //clone user as into moves user after into() operation in order to reuse user in subsequent calls
        let lg_dto_user : LoginDto = user.clone().into();
        let pf_dto_user: ProfileDto = user.clone().into();

        println!("User to LoginDto = {:?}",lg_dto_user);
        println!("User to ProfileDto = {:?}",pf_dto_user);
```

Let's consider now we have a **PersonDto** and we'd like to convert it back partially to a **User** object knowing that **PersonDto** misses some fields:
```rust
        let person = PersonDto{
            firstname: "Dessalines".to_string(),
            lastname: "Jean Jacques".to_string(),
            email: Some("dessalines@mail.ht".to_string()),
        };

        let user_from_person: User = person.into();
```

Here is how **vscode** prints the code generated by DTO mapper for the **LoginDto** and **ProfileDto**
```Rust
pub struct LoginDto {
    pub login: String,
    pub password: String,
} // size = 48 (0x30), align = 0x8

pub struct ProfileDto {
    pub username: String,
    pub email: String,
    pub firstname: String,
    pub middle_name: Option<String>,
    pub lastname: String,
    pub age: u8,
} // size = 128 (0x80), align = 0x8

pub struct PersonDto {
    pub email: Option<String>,
    pub firstname: String,
    pub lastname: String,
} // size = 72 (0x48), align = 0x8
```
# Description of macro derive and attributes for Dto Mapper
First, DTO Mapper library requires that the source struct implements Default traits as the library relies on it to implement Into Traits for conversion between DTO and Source struct.
If not it will result in error in your IDE. It is a must to to derive or implement Default for your source struct.
```rust
#[derive(DtoMapper,Default,Clone)]
struct SourceStruct{ }
```
- ## `#[mapper()]` attributes
  **mapper** attributes can be repeated for as many dtos needed to be created. Each mapper represents a concrent dto struct.  
  - **Required fields** will result in build errors if not present.
    - **dto** : name for the dto that will result into a struct with the same name. Example : `dto="MyDto"` will result into a struct named **MyDto**.
      dto names must be unique. Otherwise, it will result into build errors.
    - **map** : an array of field names from the original struct to include or  map to the new dto as fields. `map=[("fieldname:new_fieldname", required_flag )]`.
      `fieldname:new_fieldname` will rename the source field to the new one. It is not mandatory to rename. you can have `map=[("fieldname",true)]`
      `required_flag` can be true or false. if required_flag is false it will make the field an **Option** type in the dto
  - **Optional fields**
    - **ignore** : an array of fieldnames not to include in the destination dtos. `ignore=["field1", "field1"]`
      if **ignore** is present , then **map** field becomes optional. Except if needed rename destination fields for the dto
    - **derive** : a list of of macro to derive from. `derive=(Debug,Clone)`
