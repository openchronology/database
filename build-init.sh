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

envsubst < sql/schema.sql | sed 's/\$\$/\\\$\\\$/g' >> init.sh
envsubst < sql/time_points.sql | sed 's/\$\$/\\\$\\\$/g' >> init.sh
envsubst < sql/summaries.sql | sed 's/\$\$/\\\$\\\$/g' >> init.sh
envsubst < sql/utils.sql | sed 's/\$\$/\\\$\\\$/g' >> init.sh
envsubst < sql/init.sql | sed 's/\$\$/\\\$\\\$/g' >> init.sh

echo "EOSQL" >> init.sh
