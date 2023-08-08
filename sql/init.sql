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
) RETURNS SETOF api.time_point_or_summary AS $$
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
$$ LANGUAGE 'plpgsql';

