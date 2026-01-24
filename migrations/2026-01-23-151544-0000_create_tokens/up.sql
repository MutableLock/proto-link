-- Your SQL goes here
CREATE TABLE tokens
(
    id         BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    token      BIGINT UNSIGNED NOT NULL UNIQUE,
    user_id    BIGINT UNSIGNED NOT NULL,
    expires_at DATETIME        NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users (id)
);