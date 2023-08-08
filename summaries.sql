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
