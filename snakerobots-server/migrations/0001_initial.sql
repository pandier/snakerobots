CREATE TABLE users (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    username VARCHAR(20) NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE matches (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    seed BIGINT NOT NULL,
    winner_index INT,
    played_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE match_players (
    "index" INT NOT NULL,
    match_id INT NOT NULL,
    user_id INT REFERENCES users(id) ON DELETE SET NULL,
    moves BYTEA NOT NULL,
    PRIMARY KEY ("index", match_id)
);
