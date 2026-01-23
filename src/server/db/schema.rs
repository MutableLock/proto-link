// @generated automatically by Diesel CLI.

diesel::table! {
    chat_roles (id) {
        id -> Unsigned<Bigint>,
    }
}

diesel::table! {
    chats (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        owner_id -> Unsigned<Bigint>,
    }
}

diesel::table! {
    chats_users (id) {
        id -> Unsigned<Bigint>,
        user_id -> Unsigned<Bigint>,
        chat_id -> Unsigned<Bigint>,
        role_id -> Unsigned<Bigint>,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        login -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        password_hash -> Blob,
    }
}

diesel::joinable!(chats -> users (owner_id));
diesel::joinable!(chats_users -> chat_roles (role_id));
diesel::joinable!(chats_users -> chats (chat_id));
diesel::joinable!(chats_users -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(chat_roles, chats, chats_users, users,);
