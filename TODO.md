TODO
====

- Make the whole database aware of sessions - only allow users to see differences
  in what they're requesting (changing their zoom level, precision, or offset),
  similar to a multiplayer game in which one host is the decision maker for

  - implies tracking multiple sessions per user, would have to be able to tell
    what changes in someone's view

- Provide breakpoints in summaries - the largest difference between points to let
  the summary break apart - the difference at which the threshold would have to
  be less than in order to break apart the summary.

Questionable Ideas
------------------

1. Should a defined summary be summarized if its left and right bounds are lower
   than the threshold?
2. Should there be an option to ignore min / max thresholds for summaries, to
   know the context of the timeline? Say you're super zoomed in, inside a bunch
   of summaries that are past their min threshold, would it be helpful for a
   user to see what context they're inside? What about if a user is super zoomed
   out, where their threshold is larger than the max threshold - would it be
   useful to see all timespans in a timeline, even though summaries are only
   theoretically relevant to a zoom level?
