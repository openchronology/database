FROM postgres:15

SHELL ["/bin/bash", "-c"]
RUN apt-get update; \
      apt-get install -y python-is-python3 \
      python3-venv build-essential libgmp-dev \
      postgresql-server-dev-15 python3-six cron sudo
RUN python -m venv .venv
RUN source .venv/bin/activate
RUN .venv/bin/pip install pgxnclient
RUN .venv/bin/pgxn install pgmp

EXPOSE 5432
# CMD ["/cron-then-postgres.sh"]
CMD ["postgres"]
