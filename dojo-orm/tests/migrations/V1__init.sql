-- create user table
CREATE TABLE users
(
    id         uuid PRIMARY KEY,
    name       TEXT      NOT NULL,
    email      TEXT      NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_users_email ON users (email);

-- create product table
CREATE TYPE status AS ENUM ('admin', 'user');
CREATE TABLE products
(
    id         uuid PRIMARY KEY,
    name       TEXT      NOT NULL,
    detail     jsonb,
    price      int4,
    status     status,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- create test table
CREATE TABLE test
(
    id         uuid PRIMARY KEY,
    items      jsonb[]   NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- create movie table
CREATE TABLE movies
(
    id         uuid PRIMARY KEY,
    name       TEXT      NOT NULL,
    detail     TEXT      NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

ALTER TABLE movies
    ADD search tsvector GENERATED ALWAYS AS
        (TO_TSVECTOR('english', name) || ' ' || TO_TSVECTOR('english', detail)) STORED;
CREATE INDEX idx_search ON movies USING GIN (search);
