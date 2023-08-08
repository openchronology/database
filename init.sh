
#!/bin/bash
set -e

.venv/bin/pgxn load -d $POSTGRES_DB pgmp

psql -v ON_ERROR_STOP=1 \
  --username "$POSTGRES_USER" \
  --dbname "$POSTGRES_DB" <<-EOSQL

CREATE SCHEMA api;

CREATE ROLE web_anon NOLOGIN;

GRANT USAGE ON SCHEMA api TO web_anon;
-- GRANT SELECT ON api.todos TO web_anon;

CREATE ROLE AUTHENTICATOR NOINHERIT LOGIN PASSWORD 'asdf';
GRANT web_anon TO authenticator;
-- Unique times for entries to be plotted
CREATE TABLE api.times (
  value mpq PRIMARY KEY
);
GRANT ALL ON api.times TO web_anon;

-- Human readable events in time, pointed to the times table
CREATE TABLE api.time_points (
  id serial PRIMARY KEY,
  value mpq NOT NULL,
  CONSTRAINT fk_time_point
    FOREIGN KEY(value)
    REFERENCES api.times(value)
    ON DELETE CASCADE
);
GRANT ALL ON api.time_points TO web_anon;
GRANT USAGE, SELECT ON SEQUENCE api.time_points_id_seq TO web_anon;

-- Shorthand for inserting a time point
CREATE FUNCTION api.insert_time_point_func() RETURNS TRIGGER AS \$\$
BEGIN
  INSERT INTO api.times(value) VALUES (NEW.value) ON CONFLICT DO NOTHING;
  RETURN NEW;
END;
\$\$ LANGUAGE 'plpgsql';

CREATE TRIGGER insert_time_point_trigger
  BEFORE INSERT ON api.time_points
  FOR EACH ROW
  EXECUTE FUNCTION api.insert_time_point_func();

CREATE FUNCTION api.update_time_point_func() RETURNS TRIGGER AS \$\$
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
\$\$ LANGUAGE 'plpgsql';

CREATE TRIGGER update_time_point_trigger
  BEFORE UPDATE ON api.time_points
  FOR EACH ROW
  EXECUTE FUNCTION api.update_time_point_func();

CREATE FUNCTION api.delete_time_point_func() RETURNS TRIGGER AS \$\$
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
\$\$ LANGUAGE 'plpgsql';

CREATE TRIGGER delete_time_point_trigger
  BEFORE DELETE ON api.time_points
  FOR EACH ROW
  EXECUTE FUNCTION api.delete_time_point_func();
-- Spans of time which can have a human-readable description, and can be
-- associated with time points. Their visibility can optionally be bounded
-- by a threshold (i.e. description is only visible after they've zoomed
-- out by so much, or visible when they're so close, etc.)
CREATE TABLE api.summaries (
  id serial PRIMARY KEY,
  left_bound mpq, -- where should the manual summary be relevant?
  right_bound mpq,
  min_threshold mpq, -- when should the manual summary be relevant?
  max_threshold mpq,
  CHECK
    (
      (
        -- there has to be at least one bound
        left_bound IS NOT NULL
        OR
        right_bound IS NOT NULL
        AND
        (
          (
            -- if they both exist, make sure they're monotonic
            left_bound IS NOT NULL
            AND
            right_bound IS NOT NULL
            AND
            left_bound <= right_bound
          )
          OR
          TRUE
        )
      )
      AND
      (
        (
          min_threshold IS NOT NULL
          AND
          max_threshold IS NOT NULL
          AND
          min_threshold < max_threshold -- if they were equal, it'd never be seen 
        )
        OR
        TRUE
      )
    )
);

GRANT ALL ON api.summaries TO web_anon;
GRANT USAGE, SELECT ON SEQUENCE api.summaries_id_seq TO web_anon;

-- Many-to-many relation of time points to summaries
CREATE TABLE api.time_point_summary_relations (
  id serial PRIMARY KEY,
  time_point INTEGER NOT NULL,
  summary INTEGER NOT NULL,
  CONSTRAINT fk_time_point
    FOREIGN KEY(time_point)
    REFERENCES api.time_points(id)
    ON DELETE CASCADE,
  CONSTRAINT fk_summary
    FOREIGN KEY(summary)
    REFERENCES api.summaries(id)
    ON DELETE CASCADE
);

GRANT ALL ON api.time_point_summary_relations TO web_anon;
GRANT USAGE, SELECT ON SEQUENCE api.time_point_summary_relations_id_seq TO web_anon;
-- Select time points within a window, and their next & previous values
CREATE FUNCTION api.select_time_points_with_neighbors(
    left_window mpq,
    right_window mpq
  ) RETURNS TABLE (
    value mpq,
    prev_value mpq,
    next_value mpq
  ) AS \$\$
SELECT
  value,
  LAG(value) OVER (ORDER BY value) prev_value,
  LEAD(value) OVER (ORDER BY value) next_value
FROM api.times
WHERE value > left_window AND value < right_window
ORDER BY value
\$\$ LANGUAGE SQL;

-- GRANT ALL ON api.select_time_points_with_neighbors TO web_anon;

-- Joins `select_time_points_with_neighbors` with the actual time points.
CREATE FUNCTION api.select_time_points(
    left_window mpq,
    right_window mpq
  ) RETURNS TABLE (
    id INTEGER, -- each of these are nullable
    value mpq,
    prev_value mpq,
    next_value mpq
  ) AS \$\$
WITH times_with_lag_and_lead AS (
  SELECT * FROM
    api.select_time_points_with_neighbors(
      left_window,
      right_window
    )
)
SELECT
  time_points.id,
  times_with_lag_and_lead.value,
  times_with_lag_and_lead.prev_value,
  times_with_lag_and_lead.next_value
FROM
  times_with_lag_and_lead
FULL OUTER JOIN api.time_points
  ON api.time_points.value = times_with_lag_and_lead.value
\$\$ LANGUAGE SQL;

-- GRANT ALL ON api.select_time_points TO web_anon;

-- Translates the difference between the points to whether or not they are
-- within the supplied threshold for the window (should they be summarized
-- or not
CREATE FUNCTION api.select_time_points_with_thresholds(
  left_window mpq,
  right_window mpq,
  threshold mpq
) RETURNS TABLE (
  id INTEGER,
  value mpq,
  in_threshold_left BOOLEAN,
  in_threshold_right BOOLEAN
) AS \$\$
WITH times_with_lag_and_lead AS (
  SELECT * FROM
    api.select_time_points(
      left_window,
      right_window
    )
)
SELECT 
  times_with_lag_and_lead.id,
  times_with_lag_and_lead.value,
  -- false values get translated to nulls
  ABS(times_with_lag_and_lead.value - prev_value) < threshold OR NULL
    AS in_threshold_left,
  -- false values get translated to nulls
  ABS(next_value - times_with_lag_and_lead.value) < threshold OR NULL
    AS in_threshold_right
FROM
  times_with_lag_and_lead
\$\$ LANGUAGE SQL;

-- GRANT ALL ON api.select_time_points_with_thresholds TO web_anon;
-- Row type for the return value of the complete selection
CREATE TYPE api.time_point_or_summary AS (
  time_point_id INTEGER,
  time_point_value mpq,
  summary_min mpq,
  summary_max mpq,
  summary_count INTEGER, -- represents the count _not_ currently visible
  summary_visible INTEGER[], -- the associated points that are currently visible
  -- FIXME return greatest difference between points - next breakpoint
  summary_id INTEGER
  -- maybe something like "first 5 ids" in an array?
);

-- Select all time points and summaries relevant to the window and threshold
-- FIXME threshold should be relative to window size, not a constant - (0,1)
CREATE FUNCTION api.select_time_points_and_summaries(
  left_window mpq,
  right_window mpq,
  threshold mpq
) RETURNS SETOF api.time_point_or_summary AS \$\$
DECLARE
  time_point record;
  count_so_far INTEGER;
  summary_min mpq;
  summary record;
  summary_visible INTEGER[];
BEGIN
  -- drops this relation after computation is complete
  CREATE TEMP TABLE result (
    time_point_id INTEGER,
    time_point_value mpq,
    summary_min mpq,
    summary_max mpq,
    summary_count INTEGER,
    summary_visible INTEGER[],
    summary_id INTEGER
  ) ON COMMIT DROP;
  -- loop over all time points in this window and their summary potential
  FOR time_point IN
    SELECT *
    FROM api.select_time_points_with_thresholds(
      left_window,
      right_window,
      threshold
    )
  LOOP
    IF
      time_point.in_threshold_left IS NULL
      AND
      time_point.in_threshold_right IS NULL
    THEN
    -- it's a bona-fide time_point
      INSERT INTO result(
        time_point_id,
        time_point_value,
        summary_min,
        summary_max,
        summary_count,
        summary_visible,
        summary_id
      ) VALUES (
        time_point.id,
        time_point.value,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
      );
    ELSIF
      time_point.in_threshold_right
      AND
      time_point.in_threshold_left IS NULL
    THEN
    -- it's the start of a summary
      count_so_far := 1;
      summary_min := time_point.value;
    ELSIF
      time_point.in_threshold_left
      AND
      time_point.in_threshold_right IS NULL
    THEN
    -- it's the end of a summary
      INSERT INTO result(
        time_point_id,
        time_point_value,
        summary_min,
        summary_max,
        summary_count,
        summary_visible,
        summary_id
      ) VALUES (
        NULL,
        NULL,
        summary_min,
        time_point.value,
        count_so_far + 1,
        NULL,
        NULL
      );
    ELSE
    -- it's in a summary
      count_so_far := count_so_far + 1;
    END IF;
  END LOOP;

  -- include manually written / human readable summaries
  FOR summary IN
    SELECT * FROM api.summaries
      WHERE
        (
          (
            COALESCE(summaries.left_bound <= right_window, FALSE)
            AND
            COALESCE(summaries.left_bound >= left_window, FALSE)
          )
          OR
          (
            COALESCE(summaries.right_bound <= right_window, FALSE)
            AND
            COALESCE(summaries.right_bound >= left_window, FALSE)
          )
        )
        AND
        COALESCE(summaries.min_threshold <= threshold, TRUE)
        AND
        COALESCE(summaries.max_threshold >= threshold, TRUE)
  LOOP
    SELECT COUNT(*)
      INTO count_so_far
      FROM api.time_point_summary_relations
      RIGHT OUTER JOIN api.time_points
      ON api.time_point_summary_relations.time_point = api.time_points.id   
      WHERE
        api.time_point_summary_relations.summary = summary.id
        AND api.time_points.value <= right_window
        AND api.time_points.value >= left_window;
    summary_visible := ARRAY(
      SELECT result.time_point_id
      FROM result
      INNER JOIN api.time_point_summary_relations
      ON api.time_point_summary_relations.time_point = result.time_point_id
    );
    INSERT INTO result(
      time_point_id,
      time_point_value,
      summary_min,
      summary_max,
      summary_count,
      summary_visible,
      summary_id
    ) VALUES (
      NULL,
      NULL,
      summary.left_bound,
      summary.right_bound,
      count_so_far - COALESCE(array_length(summary_visible, 1), 0),
      summary_visible,
      summary.id
    );
  END LOOP;

  RETURN QUERY SELECT * FROM result;
END
\$\$ LANGUAGE 'plpgsql';

-- GRANT ALL ON api.select_time_points_and_summaries TO web_anon;
EOSQL
