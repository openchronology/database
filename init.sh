#!/bin/bash
set -e

.venv/bin/pgxn load -d $POSTGRES_DB pgmp

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
CREATE TABLE foo_table (id serial, value mpq);
EOSQL
