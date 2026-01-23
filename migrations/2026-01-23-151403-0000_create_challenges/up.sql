-- Your SQL goes here
CREATE TABLE challenges(
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    challenge BLOB NOT NULL,
    solution BLOB NOT NULL,
    user_id BIGINT UNSIGNED NOT NULL,
    nonce BLOB NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);