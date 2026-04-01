CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(20) NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    winner UUID,
    aborted BOOL NOT NULL,
    replay JSON NOT NULL,
    played_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE match_players (
    id SERIAL PRIMARY KEY,
    match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE match_requests (
    sender_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    sender_robot_id UUID REFERENCES robots(id) ON DELETE SET NULL,
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

CREATE TABLE robots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edited_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    code VARCHAR(10000) NOT NULL DEFAULT ''
);

-- Enforce the match request limit
CREATE FUNCTION enforce_match_request_limit() RETURNS TRIGGER AS $$
    BEGIN
        IF (SELECT COUNT(*) FROM match_requests WHERE sender_id = NEW.sender_id) >= 5 THEN
            RAISE EXCEPTION USING MESSAGE = 'Maximum number of match requests reached', ERRCODE = '23Z01';
        END IF;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER match_request_limit BEFORE INSERT ON match_requests
    FOR EACH ROW EXECUTE FUNCTION enforce_match_request_limit();

-- Cleanup expired match request on insert
CREATE FUNCTION cleanup_expired_match_request() RETURNS TRIGGER AS $$
    BEGIN
        DELETE FROM match_requests
            WHERE sender_id = NEW.sender_id
            AND receiver_id = NEW.receiver_id
            AND expires_at <= now();
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER cleanup_expired_match_request BEFORE INSERT ON match_requests
    FOR EACH ROW EXECUTE FUNCTION cleanup_expired_match_request();

-- Enforce the robot limit
CREATE FUNCTION enforce_robot_limit() RETURNS TRIGGER AS $$
    BEGIN
        IF (SELECT COUNT(*) FROM robots WHERE user_id = NEW.user_id) >= 10 THEN
            RAISE EXCEPTION USING MESSAGE = 'Maximum number of robots reached', ERRCODE = '23Z01';
        END IF;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER robot_limit BEFORE INSERT ON robots
    FOR EACH ROW EXECUTE FUNCTION enforce_robot_limit();
