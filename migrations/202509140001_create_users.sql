CREATE TABLE
    IF NOT EXISTS users (
        id BLOB PRIMARY KEY,
        name TEXT NOT NULL,
        email TEXT NOT NULL UNIQUE,
        created_at TEXT NOT NULL
    );