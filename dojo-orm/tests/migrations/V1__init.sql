CREATE TABLE users
(
    id         uuid PRIMARY KEY,
    name       TEXT      NOT NULL,
    email      TEXT      NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

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

CREATE TABLE test
(
    id         uuid PRIMARY KEY,
    items      jsonb[]   NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);