CREATE UNIQUE INDEX idx_users_lower_username ON "users" (LOWER(username));
CREATE INDEX idx_users_ranked_elo ON "users" (elo DESC) WHERE ranked = true;

CREATE INDEX idx_robots_user_id ON "robots" (user_id);

CREATE INDEX idx_sessions_user_id ON "sessions" (user_id);
CREATE INDEX idx_sessions_expires_at ON "sessions" (expires_at);

CREATE INDEX idx_matches_played_at ON "matches" (played_at DESC);
CREATE INDEX idx_matches_ranked ON "matches" (ranked);

CREATE INDEX idx_match_players_user_id ON "match_players" (user_id);
CREATE INDEX idx_match_players_match_id ON "match_players" (match_id);

CREATE INDEX idx_match_requests_receiver_id ON "match_requests" (receiver_id);
CREATE INDEX idx_match_requests_expires_at ON "match_requests" (expires_at);

CREATE INDEX idx_match_queue_pending ON "match_queue" (queued_at) WHERE result IS NULL;
CREATE INDEX idx_match_queue_worker_id ON "match_queue" (worker_id);
