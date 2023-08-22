-- Translates the difference between the points to whether or not they are
-- within the supplied threshold for the window (should they be summarized
-- or not
CREATE FUNCTION api.select_time_points_with_thresholds(
    session_id uuid
  ) RETURNS TABLE (
    id INTEGER,
    value mpq,
    timeline INTEGER,
    in_threshold_left BOOLEAN,
    in_threshold_right BOOLEAN
  ) AS $$
DECLARE
  left_window mpq;
  right_window mpq;
  threshold mpq;
BEGIN
  SELECT INTO left_window, right_window, threshold
         api.sessions_precomupted.left_window,
         api.sessions_precomupted.right_window,
         api.sessions_precomupted.threshold
  FROM api.sessions_precomupted
  WHERE id = session_id;
  RETURN QUERY
    -- join the `times` with `time_points` and determine whether they're in threshold
    SELECT
      time_points.id,
      selected_times_with_neighbors.value,
      time_points.timeline,
      ABS(selected_times_with_neighbors.value - selected_times_with_neighbors.prev_value)
        < threshold OR NULL
        AS in_threshold_left,
      ABS(selected_times_with_neighbors.next_value - selected_times_with_neighbors.value)
        < threshold OR NULL
        AS in_threshold_right
    FROM
      (
        -- get points from `times` table with their neighbors
        SELECT
          value,
          LAG(value) OVER (ORDER BY value) prev_value,
          LEAD(value) OVER (ORDER BY value) next_value
          FROM api.times
        WHERE value > left_window AND value < right_window
        ORDER BY value
      ) selected_times_with_neighbors
    -- join `times` with `time_points` on the value
    INNER JOIN api.time_points
      ON api.time_points.value = selected_times_with_neighbors.value;
END
$$ LANGUAGE 'plpgsql';
