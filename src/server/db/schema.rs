// @generated automatically by Diesel CLI.

diesel::table! {
    challenges (id) {
        id -> Unsigned<Bigint>,
        user_id -> Unsigned<Bigint>,
        #[max_length = 255]
        challenge_answer -> Varchar,
        #[max_length = 255]
        challenge -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        login -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        password_hash_argon2 -> Varchar,
        #[max_length = 255]
        password_salt -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(challenges, users,);
