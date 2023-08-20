#!/bin/bash

echo '
#!/bin/bash
set -e

.venv/bin/pgxn load -d $POSTGRES_DB pgmp

psql -v ON_ERROR_STOP=1 \
  --username "$POSTGRES_USER" \
  --dbname "$POSTGRES_DB" <<-EOSQL
' > init.sh

set -a
source .env

cat sql/* \
    | sed 's/\${\(\w\w*\)}/QWERTY1{\1}/g' \
    | sed 's/\$/QWERTY2/g; s/`/\\`/g' \
    | sed 's/QWERTY1{\(\w\w*\)}/\${\1}/g' \
    | envsubst \
    | sed 's/QWERTY2/\\\$/g' \
    >> init.sh

echo "EOSQL" >> init.sh
