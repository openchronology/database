-- Select time points within a window, and their next & previous values
CREATE FUNCTION api.select_times_with_neighbors(
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
FROM api.times
WHERE value > left_window AND value < right_window
ORDER BY value
$$ LANGUAGE SQL;


-- Joins `select_times_with_neighbors` with the actual time points.
CREATE FUNCTION api.select_time_points(
    left_window mpq,
    right_window mpq
  ) RETURNS TABLE (
    id INTEGER, -- each of these are nullable
    value mpq,
    timeline INTEGER,
    prev_value mpq,
    next_value mpq
  ) AS $$
WITH times_with_lag_and_lead AS (
  SELECT * FROM
    api.select_times_with_neighbors(
      left_window,
      right_window
    )
)
SELECT
  time_points.id,
  times_with_lag_and_lead.value,
  time_points.timeline,
  times_with_lag_and_lead.prev_value,
  times_with_lag_and_lead.next_value
FROM
  times_with_lag_and_lead
INNER JOIN api.time_points
  ON api.time_points.value = times_with_lag_and_lead.value
$$ LANGUAGE SQL;


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
  timeline INTEGER,
  in_threshold_left BOOLEAN,
  in_threshold_right BOOLEAN
) AS $$
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
  times_with_lag_and_lead.timeline,
  -- false values get translated to nulls
  ABS(times_with_lag_and_lead.value - prev_value) < threshold OR NULL
    AS in_threshold_left,
  -- false values get translated to nulls
  ABS(next_value - times_with_lag_and_lead.value) < threshold OR NULL
    AS in_threshold_right
FROM
  times_with_lag_and_lead
$$ LANGUAGE SQL;

