-- Your SQL goes here
create table chats
(
    id   BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255),
    owner_id BIGINT UNSIGNED NOT NULL,

    FOREIGN KEY (owner_id) REFERENCES users(id)
);