CREATE TABLE api.timelines (
  id serial PRIMARY KEY,
  author text NOT NULL DEFAULT current_user
  -- title, description, etc.
);
GRANT ALL ON api.timelines TO guest_group;

ALTER TABLE api.timelines ENABLE ROW LEVEL SECURITY;
CREATE POLICY timelines_ids_policy ON api.timelines TO guest_group
  USING (author = current_user);
CREATE POLICY timelines_ids_policy ON api.timelines TO mod_group
  USING TRUE;
