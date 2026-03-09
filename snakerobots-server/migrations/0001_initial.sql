CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(20) NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seed BIGINT NOT NULL,
    winner_index INT,
    aborted BOOL NOT NULL,
    played_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE match_players (
    "index" INT NOT NULL,
    match_id UUID NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    moves TEXT NOT NULL,
    PRIMARY KEY ("index", match_id)
);

CREATE TABLE match_requests (
    sender_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    receiver_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (sender_id, receiver_id)
);

CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);
