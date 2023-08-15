-- Unique times for entries to be plotted
CREATE TABLE api.times (
  value mpq PRIMARY KEY
);
GRANT ALL ON api.times TO guest_group;

-- Human readable events in time, pointed to the times table
CREATE TABLE api.time_points (
  id serial PRIMARY KEY,
  value mpq NOT NULL,
  timeline INTEGER NOT NULL,
  CONSTRAINT fk_time_point
    FOREIGN KEY(value)
    REFERENCES api.times(value)
    ON DELETE CASCADE,
  CONSTRAINT fk_timeline
    FOREIGN KEY(timeline)
    REFERENCES api.timelines(id)
    ON DELETE CASCADE
);
GRANT SELECT ON api.time_points TO guest_group;

ALTER TABLE api.time_points ENABLE ROW LEVEL SECURITY;
-- all time_points are visible to guests
CREATE POLICY time_points_select_policy
  ON api.time_points
  FOR SELECT
  USING (TRUE);
-- only a timeline owned can be edited or inserted
CREATE POLICY time_points_insert_policy
  ON api.time_points
  FOR INSERT
  WITH CHECK (
    timeline IN (
      SELECT api.timelines.id
        FROM api.timelines
      WHERE api.timelines.author = current_user
    )
    OR
    'mod_group' IN (
      SELECT get_roles(text(current_user))
    )
  );
CREATE POLICY time_points_update_policy
  ON api.time_points
  FOR UPDATE
  USING (
    timeline IN (
      SELECT api.timelines.id
        FROM api.timelines
      WHERE api.timelines.author = current_user
    )
    OR
    'mod_group' IN (
      SELECT get_roles(text(current_user))
    )
  );
CREATE POLICY time_points_delete_policy
  ON api.time_points
  FOR DELETE
  USING (
    timeline IN (
      SELECT api.timelines.id
        FROM api.timelines
      WHERE api.timelines.author = current_user
    )
    OR
    'mod_group' IN (
      SELECT get_roles(text(current_user))
    )
  );

-- Shorthand for inserting a unique time whenever inserting a time point
CREATE FUNCTION api.insert_time_point_func() RETURNS TRIGGER AS $$
BEGIN
  INSERT INTO api.times(value) VALUES (NEW.value) ON CONFLICT DO NOTHING;
  RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

CREATE TRIGGER insert_time_point_trigger
  BEFORE INSERT ON api.time_points
  FOR EACH ROW
  EXECUTE FUNCTION api.insert_time_point_func();

CREATE FUNCTION api.update_time_point_func() RETURNS TRIGGER AS $$
DECLARE
  old_count INTEGER;
BEGIN
  SELECT COUNT(*) INTO old_count
  FROM api.times
  WHERE api.times.value = OLD.value;
  IF old_count = 1
  THEN
    DELETE FROM api.times WHERE api.times.value = OLD.value;
  END IF;
  INSERT INTO api.times(value) VALUES (NEW.value) ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE 'plpgsql';

CREATE TRIGGER update_time_point_trigger
  BEFORE UPDATE ON api.time_points
  FOR EACH ROW
  EXECUTE FUNCTION api.update_time_point_func();

CREATE FUNCTION api.delete_time_point_func() RETURNS TRIGGER AS $$
DECLARE
  old_count INTEGER;
BEGIN
  SELECT COUNT(*) INTO old_count
  FROM api.times
  WHERE api.times.value = OLD.value;
  IF old_count = 1
  THEN
    DELETE FROM api.times WHERE times.value = OLD.value;
  END IF;
END;
$$ LANGUAGE 'plpgsql';

CREATE TRIGGER delete_time_point_trigger
  BEFORE DELETE ON api.time_points
  FOR EACH ROW
  EXECUTE FUNCTION api.delete_time_point_func();
