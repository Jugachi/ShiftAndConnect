-- Add migration script here
CREATE TABLE games (
    room_code VARCHAR(6) PRIMARY KEY,
    game_mode VARCHAR(10) NOT NULL,
    board JSONB NOT NULL,
    current_player INT NOT NULL
);