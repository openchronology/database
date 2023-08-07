#!/bin/bash

echo '
#!/bin/bash
set -e

.venv/bin/pgxn load -d $POSTGRES_DB pgmp

psql -v ON_ERROR_STOP=1 \
  --username "$POSTGRES_USER" \
  --dbname "$POSTGRES_DB" <<-EOSQL
' > init.sh

cat init.sql | sed 's/\$\$/\\\$\\\$/g' >> init.sh

echo "EOSQL" >> init.sh
