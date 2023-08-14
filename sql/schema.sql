CREATE SCHEMA api;

-- Heirarchy of users
CREATE ROLE guest_group INHERIT NOLOGIN;
CREATE ROLE mod_group INHERIT NOLOGIN;
CREATE ROLE admin_group INHERIT NOLOGIN CREATEROLE BYPASSRLS;
GRANT guest_group TO mod_group;
GRANT mod_group TO admin_group;

GRANT USAGE ON SCHEMA api TO guest_group, mod_group, admin_group;

CREATE ROLE authenticator_user NOINHERIT LOGIN PASSWORD '${PGRST_AUTHENTICATOR_PW}';
GRANT guest_group, mod_group, admin_group TO authenticator_user;

CREATE ROLE test_user INHERIT NOLOGIN;
GRANT test_user TO authenticator_user;
GRANT admin_group TO test_user;
