
-- special authenticator user used by PostgREST
CREATE ROLE authenticator_user NOINHERIT LOGIN PASSWORD '${PGRST_AUTHENTICATOR_PW}';
GRANT guest_group TO authenticator_user;

-- special test user for test runner
CALL touch_user('test_user_user');
CALL touch_user('test_mod_user');
CALL touch_user('test_admin_user');
GRANT mod_group TO test_mod_user;
GRANT admin_group TO test_admin_user;
