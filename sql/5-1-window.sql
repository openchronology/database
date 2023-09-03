-- Row type for the return value of the complete selection
CREATE TYPE api.time_point_or_summary AS (
  time_point_id INTEGER,
  time_point_value mpq,
  time_point_timeline INTEGER,
  summary_min mpq,
  summary_max mpq,
  summary_count INTEGER, -- represents the count _not_ currently visible
  summary_next_threshold mpq, -- represents the next breakpoint for this summary
  summary_visible INTEGER[], -- the associated points that are currently visible
  -- FIXME return greatest difference between points - next breakpoint
  summary_id INTEGER
  -- maybe something like "first 5 ids" in an array?
);

-- Select all time points and summaries relevant to the window and threshold
-- FIXME threshold should be relative to window size, not a constant - (0,1)
CREATE FUNCTION api.select_time_points_and_summaries(
  session_id uuid
) RETURNS SETOF api.time_point_or_summary AS $$
DECLARE
  time_point record;
  count_so_far INTEGER;
  summary_min mpq;
  summary_next_threshold mpq;
  summary record;
  summary_visible INTEGER[];
  left_window mpq;
  right_window mpq;
  threshold mpq;
BEGIN
  -- drops this relation after computation is complete
  CREATE TEMP TABLE result (
    time_point_id INTEGER,
    time_point_value mpq,
    time_point_timeline INTEGER,
    summary_min mpq,
    summary_max mpq,
    summary_count INTEGER,
    summary_next_threshold mpq,
    summary_visible INTEGER[],
    summary_id INTEGER,
    CHECK (
      (
        (
          -- not a timeline
          time_point_id IS NULL
          AND time_point_value IS NULL
          AND time_point_timeline IS NULL
        )
        AND
        (
          -- is a summary
          summary_min IS NOT NULL
          AND summary_max IS NOT NULL
          AND summary_count IS NOT NULL
          AND summary_next_threshold IS NOT NULL
        )
        AND
        (
          (
            -- is a general summary
            summary_visible IS NOT NULL
            AND summary_id IS NOT NULL
          )
          OR
          (
            -- is a specific summary
            summary_visible IS NULL
            AND summary_id IS NULL
          )
        )
      )
      OR
      (
        (
          -- is a timeline
          time_point_id IS NOT NULL
          AND time_point_value IS NOT NULL
          AND time_point_timeline IS NOT NULL
        )
        AND
        (
          -- but not a summary
          summary_min IS NULL
          AND summary_max IS NULL
          AND summary_count IS NULL
          AND summary_next_threshold IS NULL
          AND summary_visible IS NULL
          AND summary_id IS NULL
        )
      )
    )
  ) ON COMMIT DROP;

  -- Record time points for general summaries within this window - an accumulator
  CREATE TEMPORARY TABLE not_visible_general (
    id INTEGER NOT NULL,
    value mpq NOT NULL,
    timeline INTEGER NOT NULL,
    -- the greatest threshold around this point
    max_threshold mpq NOT NULL
  ) ON COMMIT DROP;

  -- Record _all_ time points that are going to be summarized within this window
  CREATE TEMPORARY TABLE not_visible (
    id INTEGER NOT NULL,
    value mpq NOT NULL,
    timeline INTEGER NOT NULL,
    -- the greatest threshold around this point
    max_threshold mpq NOT NULL
  ) ON COMMIT DROP;

  -- loop over all time points in this window and their summary potential
  FOR time_point IN SELECT * FROM
    api.select_time_points_with_thresholds(session_id)
  LOOP
    IF
      NOT time_point.in_threshold_left AND NOT time_point.in_threshold_right
    THEN
    -- it's a bona-fide time_point
      INSERT INTO result(
        time_point_id,
        time_point_value,
        time_point_timeline
      ) VALUES (
        time_point.id,
        time_point.value,
        time_point.timeline
      );
    ELSIF
      NOT time_point.in_threshold_left AND time_point.in_threshold_right
    THEN
    -- it's the start of a summary
      count_so_far := 1;
      summary_min := time_point.value;
      -- record this one for specific summary identification
      INSERT INTO not_visible(
        id,
        value,
        timeline,
        max_threshold
      ) VALUES (
        time_point.id,
        time_point.value,
        time_point.timeline,
        time_point.threshold_right
      );
      -- record this one for accumulation as a general summary
      INSERT INTO not_visible_general(
        id,
        value,
        timeline,
        max_threshold
      ) VALUES (
        time_point.id,
        time_point.value,
        time_point.timeline,
        time_point.threshold_right
      );
    ELSIF
      time_point.in_threshold_left AND NOT time_point.in_threshold_right
    THEN
    -- it's the end of a summary
      IF count_so_far IS NULL
      THEN
        -- yet this is the first one we've seen, and will be the last, therefore it's a point
        INSERT INTO result(
          time_point_id,
          time_point_value,
          time_point_timeline
        ) VALUES (
          time_point.id,
          time_point.value,
          time_point.timeline
        );
      ELSE
        -- there's others in this summary so its safe to store within the summary.
        INSERT INTO not_visible(
          id,
          value,
          timeline,
          max_threshold
        ) VALUES (
          time_point.id,
          time_point.value,
          time_point.timeline,
          time_point.threshold_left
        );
        -- also accumulate this for general summaries
        INSERT INTO not_visible_general(
          id,
          value,
          timeline,
          max_threshold
        ) VALUES (
          time_point.id,
          time_point.value,
          time_point.timeline,
          time_point.threshold_left
        );
        -- get the largest threshold within this summary - utilize the accumulator
        SELECT MAX(not_visible_general.max_threshold)
          INTO summary_next_threshold
          FROM not_visible_general;
        -- clear the entries not visible so the next summary won't conflict with this one
        DELETE FROM not_visible_general;
        -- store the compiled summary
        INSERT INTO result(
          summary_min,
          summary_max,
          summary_count,
          summary_next_threshold,
          summary_visible,
          summary_id
        ) VALUES (
          summary_min,
          time_point.value,
          count_so_far + 1,
          summary_next_threshold,
          NULL,
          NULL
        );
      END IF;
    ELSE
      -- it's in a summary, we need to record this point as not being visible
      INSERT INTO not_visible(
        id,
        value,
        timeline,
        max_threshold
      ) VALUES (
        time_point.id,
        time_point.value,
        time_point.timeline,
        -- both are guaranteed to be non-null
        GREATEST(time_point.threshold_left, time_point.threshold_right)
      );
      -- also record for accumulation
      INSERT INTO not_visible_general(
        id,
        value,
        timeline,
        max_threshold
      ) VALUES (
        time_point.id,
        time_point.value,
        time_point.timeline,
        -- both are guaranteed to be non-null
        GREATEST(time_point.threshold_left, time_point.threshold_right)
      );
      IF count_so_far IS NULL
      THEN
        -- this is actually the first one we've seen - proceed as if in_threshold_left is false
        count_so_far := 1;
        summary_min := time_point.value;
      ELSE
        -- we've seen another point before this
        count_so_far := count_so_far + 1;
      END IF;
    END IF;
  END LOOP;

  -- Shouldn't use accumulator anymore.
  DROP TABLE not_visible_general;

-- FIXME use the temp table already generated?
-- FIXME use api.select_time_points_with_thresholds to get next breakpoint?
  -- gets the computed bounds and threshold
  SELECT INTO left_window, right_window, threshold
              api.sessions_precomputed.left_window,
              api.sessions_precomputed.right_window,
              api.sessions_precomputed.threshold
  FROM api.sessions_precomputed
  WHERE api.sessions_precomputed.id = session_id;

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
    -- get only the relations for this current summary
    CREATE VIEW current_relations AS
      SELECT * FROM api.time_point_summary_relations
        WHERE api.time_point_summary_relations.summary = summary.id;
    -- get the max thresholds for this summary
    CREATE VIEW current_thresholds AS
      SELECT not_visible.max_threshold
        FROM current_relations
        INNER JOIN not_visible
        ON current_relations.time_point = not_visible.id;
    -- count the ones that aren't currently visible
    SELECT COUNT(*) INTO count_so_far FROM current_thresholds;
    -- now select the ones that actually _are_ currently visible
    summary_visible := ARRAY(
      SELECT result.time_point_id
      FROM result
      INNER JOIN current_relations
      ON current_relations.time_point = result.time_point_id
    );
    SELECT MAX(current_thresholds.max_threshold)
      INTO summary_next_threshold
      FROM current_thresholds;
    -- store the specific summary
    INSERT INTO result(
      summary_min,
      summary_max,
      summary_count,
      summary_next_threshold,
      summary_visible,
      summary_id
    ) VALUES (
      summary.left_bound,
      summary.right_bound,
      count_so_far - COALESCE(array_length(summary_visible, 1), 0),
      summary_next_threshold,
      summary_visible,
      summary.id
    );
    DROP VIEW current_thresholds;
    DROP VIEW current_relations;
  END LOOP;

  RETURN QUERY SELECT * FROM result;
END
$$ LANGUAGE 'plpgsql';

