CREATE SCHEMA api;

CREATE ROLE web_anon NOLOGIN;

GRANT USAGE ON SCHEMA api TO web_anon;
-- GRANT SELECT ON api.todos TO web_anon;

CREATE ROLE AUTHENTICATOR NOINHERIT LOGIN PASSWORD 'asdf';
GRANT web_anon TO authenticator;
