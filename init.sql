-- Unique times for entries to be plotted
CREATE TABLE times (
  value mpq PRIMARY KEY
);

-- Spans of time which can have a human-readable description, and can be
-- associated with time points. Their visibility can optionally be bounded
-- by a threshold (i.e. description is only visible after they've zoomed
-- out by so much, or visible when they're so close, etc.)
CREATE TABLE summaries (
  id serial PRIMARY KEY,
  left_bound mpq, -- where should the manual summary be relevant?
  right_bound mpq,
  min_threshold mpq, -- when should the manual summary be relevant?
  max_threshold mpq,
  CHECK
    (
      left_bound IS NOT NULL
      OR
      right_bound IS NOT NULL
      AND
      (
        (
          left_bound IS NOT NULL
          AND
          right_bound IS NOT NULL
          AND
          left_bound <= right_bound
        )
        OR
        TRUE
      )
    ) -- there has
  -- to be at least one bound
);

-- Human readable events in time, pointed to the times table
CREATE TABLE time_points (
  id serial PRIMARY KEY,
  value mpq NOT NULL,
  CONSTRAINT fk_time_point
    FOREIGN KEY(value)
    REFERENCES times(value)
);

-- Shorthand for inserting a time point
CREATE PROCEDURE insert_time_point(
    value mpq
  ) LANGUAGE SQL AS $$
INSERT INTO times(value) VALUES (value) ON CONFLICT DO NOTHING;
INSERT INTO time_points(value) VALUES (value);
$$;

-- Many-to-many relation of time points to summaries
CREATE TABLE time_point_summary_relations (
  id serial PRIMARY KEY,
  time_point INTEGER NOT NULL,
  summary INTEGER NOT NULL,
  CONSTRAINT fk_time_point
    FOREIGN KEY(time_point)
    REFERENCES time_points(id),
  CONSTRAINT fk_summary
    FOREIGN KEY(summary)
    REFERENCES summaries(id)
);

-- Select time points within a window, and their next & previous values
CREATE FUNCTION select_time_points_with_neighbors(
    left_window mpq,
    right_window mpq
  ) RETURNS TABLE (
    value mpq,
    prev_value mpq,
    next_value mpq
  ) AS $$
SELECT
  value,
  LAG(value) OVER (ORDER BY value) prev_value,
  LEAD(value) OVER (ORDER BY value) next_value
FROM times
WHERE value > left_window AND value < right_window
ORDER BY value
$$ LANGUAGE SQL;

-- Joins `select_time_points_with_neighbors` with the actual time points.
CREATE FUNCTION select_time_points(
    left_window mpq,
    right_window mpq
  ) RETURNS TABLE (
    id INTEGER, -- each of these are nullable
    value mpq,
    prev_value mpq,
    next_value mpq
  ) AS $$
WITH times_with_lag_and_lead AS (
  SELECT * FROM
    select_time_points_with_neighbors(
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
FULL OUTER JOIN time_points
  ON time_points.value = times_with_lag_and_lead.value
$$ LANGUAGE SQL;

-- Translates the difference between the points to whether or not they are
-- within the supplied threshold for the window (should they be summarized
-- or not
CREATE FUNCTION select_time_points_with_thresholds(
  left_window mpq,
  right_window mpq,
  threshold mpq
) RETURNS TABLE (
  id INTEGER,
  value mpq,
  in_threshold_left BOOLEAN,
  in_threshold_right BOOLEAN
) AS $$
WITH times_with_lag_and_lead AS (
  SELECT * FROM
    select_time_points(
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
$$ LANGUAGE SQL;

-- Row type for the return value of the complete selection
CREATE TYPE time_point_or_summary AS (
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
CREATE FUNCTION select_time_points_and_summaries(
  left_window mpq,
  right_window mpq,
  threshold mpq
) RETURNS SETOF time_point_or_summary AS $$
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
    FROM select_time_points_with_thresholds(
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
    SELECT * FROM summaries
      WHERE
        (
          summaries.left_bound <= right_window
          AND
          summaries.left_bound >= left_window
        )
        OR
        (
          summaries.right_bound <= right_window
          AND
          summaries.right_bound >= left_window
        )
  LOOP
    SELECT COUNT(*)
      INTO count_so_far
      FROM time_point_summary_relations
      RIGHT OUTER JOIN time_points
      ON time_point_summary_relations.time_point = time_points.id   
      WHERE
        time_point_summary_relations.summary = summary.id
        AND time_points.value <= right_window
        AND time_points.value >= left_window;
    summary_visible := ARRAY(
      SELECT result.time_point_id
      FROM result
      INNER JOIN time_point_summary_relations
      ON time_point_summary_relations.time_point = result.time_point_id
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
$$ LANGUAGE 'plpgsql';

-- CREATE FUNCTION select_summarized_time_points(
--   left_window mpq,
--   right_window mpq,
--   threshold mpq
-- ) RETURNS TABLE (
--   value mpq, -- midpoint
--   count INTEGER
--   -- id INTEGER -- summary identifier if applicable
-- ) AS $$
--     WITH times_with_lag_and_lead AS (
--       SELECT * FROM
--         select_time_points_with_neighbors(
--           left_window,
--           right_window
--         )
--     ), times_closer_than_threshold AS (
--       SELECT 
--         times_with_lag_and_lead.value,
--         ABS(times_with_lag_and_lead.value - prev_value) < threshold
--           AS in_threshold_left,
--         ABS(next_value - times_with_lag_and_lead.value) < threshold
--           AS in_threshold_right
--       FROM
--         times_with_lag_and_lead
--     ), times_with_group_code AS (
--       SELECT
--         COUNT(is_too_close) OVER w AS group_code,
--         MIN(times_closer_than_threshold.value) OVER w
--           +
--         MAX(times_closer_than_threshold.value) OVER w
--           / mpq('2') AS middle,
--         times_closer_than_threshold.value
--       FROM
--         times_closer_than_threshold
--       WINDOW w AS (ORDER BY times_closer_than_threshold.value)
--     )
--     SELECT
--       SUM(times_with_group_code.middle) / mpq(COUNT(*)) AS value,
--       COUNT(*) AS count
--       -- id should match in its selection criteria
--     FROM times_with_group_code
--     GROUP BY times_with_group_code.group_code
--     ORDER BY times_with_group_code.group_code
-- $$ LANGUAGE SQL;
