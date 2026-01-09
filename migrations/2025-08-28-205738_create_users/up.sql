-- Your SQL goes here
create table users(
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    login VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    password_hash_pbkdf2 VARCHAR(255) NOT NULL,
    password_salt VARCHAR(255) NOT NULL
);