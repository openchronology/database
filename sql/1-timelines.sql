CREATE TABLE api.timelines (
  id serial PRIMARY KEY,
  author text NOT NULL DEFAULT current_user
  -- title, description, etc.
);
GRANT SELECT ON api.timelines TO guest_group;

ALTER TABLE api.timelines ENABLE ROW LEVEL SECURITY;
-- all timelines are visible to guests
CREATE POLICY timelines_select_policy
  ON api.timelines
  FOR SELECT
  USING (TRUE);
CREATE POLICY timelines_update_policy
  ON api.timelines
  FOR UPDATE
  USING (current_user = author OR 'mod_group' IN (SELECT get_roles(text(current_user))));
-- all users can create timelines
CREATE POLICY timelines_insert_policy
  ON api.timelines
  FOR INSERT
  WITH CHECK ('user_group' IN (SELECT get_roles(text(current_user))));
-- only a timeline owned can be edited
CREATE POLICY timelines_delete_policy
  ON api.timelines
  FOR DELETE
  USING (current_user = author OR 'mod_group' IN (SELECT get_roles(text(current_user))));
