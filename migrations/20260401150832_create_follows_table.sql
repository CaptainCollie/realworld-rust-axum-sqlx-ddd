-- Add migration script here
CREATE TABLE follows (
    follower_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    followed_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (follower_id, followed_id)
);

CREATE INDEX follows_follower_id_idx ON follows (follower_id);
CREATE INDEX follows_followed_id_idx ON follows (followed_id);