### Changelog
All related changes will be logged here.

## [0.3.1] 2024-12-28
- Fixing bug that erroneously removing white space from attributes
  For example, before,  an attribute like this:
```rust
#[validate(email(message = "invalid email address")]
```
would be interpolated into that:
```rust
#[validate(email(message = "invalidemailaddress")]
```
This change addresses that and fixes it.

## [0.3.0] 2024-11-16
@fatihaziz: A new Contributor has joined.dto_mapper.
A special thanks that new contributor @fatihaziz that started the work on macro attributes for struct and fields
- Adding macro attributes features on struct, existing mapped fields and new fields

### Changed

- **BREAKING:** Remove `unstringify` library crate dependency. it is no longer required for this version. You can uninstall it
by issuing this command from your project. And remove any reference of it from your code base.
```shell
cargo remove unstringify
```

## [0.2.0] 2024-04-25
- New computed field features added in order to create new fields from a base struct
- **BREAKING:** Adding required dependency for `unstringify` library crate. It requires adding this crate to your project
and import it where you need to use dto_mapper macro.

## [0.1.3] 2023-12-22
- New computed field features added in order to create new fields from a base struct
- **BREAKING:** Adding required dependency for `derive_builder` library crate. It requires adding this crate to your project
and import it where you need to use dto_mapper macro.

## [0.1.2] 2023-11-20
Initial version and publishing of dto_mapper crate.
- Adding special macro mapper for dto mapping from a base struct. You can derive a struct from an existing one in order
to use it for data transfer object design patterns. It has features that no existing rust dto lib crates has exposed in
order to use dto design patterns in a flexible manner like "mapstruct" from Java world.

