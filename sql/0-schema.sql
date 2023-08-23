CREATE SCHEMA api;

-- guest_group is the anonymous group assumed by authenticator_user
CREATE ROLE guest_group INHERIT NOLOGIN;
-- user_group is granted to normal users
CREATE ROLE user_group INHERIT NOLOGIN;
-- mod_group is granted to users who are moderators
CREATE ROLE mod_group INHERIT NOLOGIN;
-- admin_group is granted to users who are admins, and can assign moderator rights
CREATE ROLE admin_group INHERIT NOLOGIN CREATEROLE BYPASSRLS;
-- Heirarchy of users
GRANT guest_group TO user_group;
GRANT user_group TO mod_group;
GRANT mod_group TO admin_group;

GRANT USAGE ON SCHEMA api TO guest_group, user_group, mod_group, admin_group;

-- Creates user
CREATE OR REPLACE PROCEDURE touch_user(IN uname text) LANGUAGE plpgsql AS
$procedure$
BEGIN
  EXECUTE format(
    'DO $do$ '
    'BEGIN '
    'IF EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = ''%1$s'') '
    'THEN RAISE NOTICE ''Role "%1$s" already exists. Skipping.''; '
    'ELSE '
    'CREATE ROLE %1$I NOLOGIN; '
    'GRANT %1$I TO authenticator_user; '
    'GRANT USAGE ON SCHEMA api TO %1$I; '
    'GRANT ALL ON api.times TO %1$I; '
    'GRANT ALL ON api.time_points TO %1$I; '
    'GRANT USAGE, SELECT ON api.time_points_id_seq TO %1$I; '
    'GRANT ALL ON api.timelines TO %1$I; '
    'GRANT USAGE, SELECT ON api.timelines_id_seq TO %1$I; '
    'GRANT ALL ON api.sessions TO %1$I; '
    'GRANT SELECT ON api.sessions_precomputed TO %1$I; '
    'GRANT user_group TO %1$I; '
    'END IF; '
    'END '
    '$do$;',
    uname);
END
$procedure$;

CREATE FUNCTION get_roles(uname text) RETURNS TABLE(rolname text) AS $$
  WITH RECURSIVE cte AS (
    SELECT oid
      FROM pg_roles
     WHERE rolname = uname
           UNION ALL
    SELECT m.roleid
      FROM cte
           JOIN pg_auth_members m ON m.member = cte.oid
    )
  SELECT oid::regrole::text AS rolname FROM cte
$$ LANGUAGE SQL;
