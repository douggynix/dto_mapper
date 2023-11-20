# dto_mapper
This is a library to create dynamic DTOs for rust based project. It has the same purpose with the known Java DTO Mapper Library called Model Java used mainly in Java SpringBoot applications.

DTO stands for Data Transfer Object. It is a Software Design pattern that involves mapping a parent object to a new one by reusing some of its properties into the newly created one.
For example, if one application is manipulating sensitive data such as credit card information, passwords, or any other high sensitive information that shouldn't be sent(serialized) over the network or 
displayed to the user in any form. So, applications need a way to map this Entity to a new one by removing the properties or fields they wouldn't want to expose to a user in JSON or any other format.

This library makes it handy. It helps annotate a structure with special attributes to extract fields to compose new structure objects for re-use. This library is based upon the **Syn** Library
and the power of macro derive attributes embedded in rust. The dtos are generated during compile time. There is no overhead during runtime for this as the final binary will contain the dto structure
resulted after buil time. I would recommend using Visual Studio code with rust analyzer plugin installed for auto-completion and a nicer developer experience with preview of field names when hovering 
over a dto structure.

# Usage
This library can be used by web applications which used to do data transformation between database and the controller that sends information to the user using DAO(Data Access Object) pattern.
Let's consider a User Entity used by an application to store into and retrieve records for a user from a database. Let's describe our User entity like this:
    struct User{
        username: String,
        password: String,
        email: String,
        firstname: String,
        middle_name: Option<String>,
        lastname: String,
        age: u8,
    }`
`

