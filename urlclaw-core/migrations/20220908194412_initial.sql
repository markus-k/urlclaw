-- Add migration script here
CREATE TABLE short_urls (
    id UUID PRIMARY KEY,
    short VARCHAR(200),
    target VARCHAR(4096)
);

CREATE UNIQUE INDEX idx_shorturls_short ON short_urls(short);
