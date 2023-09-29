use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use ulid::Ulid;

pub fn add_user(name: String, password: String, email: String) -> (String, String) {
    let id = Ulid::new().to_string();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.clone().as_bytes(), &salt)
        .unwrap()
        .to_string();
    let user_insert = format!(
        "insert into USERS (id, name, email) values ('{}', '{}', '{}');",
        id, name, email
    );

    let password_insert = format!(
        "insert into PASSWORDS (userid, salt, hash) values ('{}', '{}', '{}');",
        id, salt, password_hash,
    );
    (user_insert, password_insert)
}

pub fn migration() -> String {
    let (user_insert, password_insert) = add_user(
        String::from("admin"),
        String::from("password"),
        String::from("admin@z.com"),
    );
    let (user_insert_2, password_insert_2) = add_user(
        String::from("user"),
        String::from("password"),
        String::from("user@z.com"),
    );
    format!(
        "{} \n {} \n {} \n  {}",
        user_insert, password_insert, user_insert_2, password_insert_2
    )
}
