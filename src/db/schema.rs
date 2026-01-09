// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        login -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        password_hash_pbkdf2 -> Varchar,
        #[max_length = 255]
        password_salt -> Varchar,
    }
}
