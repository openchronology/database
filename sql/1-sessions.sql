CREATE TABLE api.sessions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  owner text NOT NULL DEFAULT current_user,
  -- center of window
  pos mpq NOT NULL DEFAULT mpq(0),
  -- distance from center to left / right bounds
  zoom mpq NOT NULL DEFAULT mpq(1) CHECK (zoom > mpq(0)),
  -- % of window space that items too close get grouped - "depth of field"
  field mpq NOT NULL DEFAULT mpq('1/10') CHECK (field > mpq(0) AND field < mpq(1)),
  last_interaction TIMESTAMP NOT NULL DEFAULT NOW()
);
GRANT SELECT, INSERT, UPDATE ON api.sessions TO guest_group;

CREATE VIEW api.sessions_precomputed AS
  SELECT
    *,
    pos - zoom AS left_window,
    pos + zoom AS right_window,
    zoom * mpq(2) * field AS threshold
  FROM api.sessions;
GRANT SELECT ON api.sessions_precomputed TO guest_group;

ALTER TABLE api.sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY sessions_policy
  ON api.sessions
    USING (current_user = owner OR 'mod_group' IN (SELECT get_roles(text(current_user))))
    WITH CHECK (current_user = owner OR 'mod_group' IN (SELECT get_roles(text(current_user))));

-- Don't let people make a session on behalf of another user
CREATE FUNCTION api.insert_session_func() RETURNS TRIGGER AS $$
BEGIN
  NEW.id := gen_random_uuid();
  NEW.owner := current_user;
  NEW.last_interaction := NOW();
  RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';
CREATE TRIGGER insert_session_trigger
  BEFORE INSERT ON api.sessions
  FOR EACH ROW
  EXECUTE FUNCTION api.insert_session_func();

-- Don't let people change their session id or owner
CREATE FUNCTION api.update_session_func() RETURNS TRIGGER AS $$
BEGIN
  NEW.id := OLD.id;
  NEW.owner := OLD.owner;
  NEW.last_interaction := NOW();
  -- FIXME doesn't call touch_session()
  RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';
CREATE TRIGGER update_session_trigger
  BEFORE UPDATE ON api.sessions
  FOR EACH ROW
  EXECUTE FUNCTION api.update_session_func();

-- Will be null if session is invalid
CREATE FUNCTION check_session(session_id uuid) RETURNS uuid AS $$
  SELECT id FROM api.sessions WHERE id = session_id AND owner = current_user
$$ LANGUAGE SQL;

-- Will be null if session is invalid
CREATE FUNCTION touch_session(session_id_pre uuid) RETURNS uuid AS $$
DECLARE
  session_id uuid;
BEGIN
  SELECT INTO session_id check_session(session_id_pre);
  UPDATE api.sessions SET last_interaction = NOW() WHERE id = session_id;
  RETURN session_id;
END;
$$ LANGUAGE 'plpgsql';
