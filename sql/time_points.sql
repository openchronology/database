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
GRANT USAGE, SELECT ON SEQUENCE api.time_points_id_seq TO guest_group;

-- Shorthand for inserting a time point
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
