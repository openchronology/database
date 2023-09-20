TODO
====

- Make the whole database aware of sessions - only allow users to see differences
  in what they're requesting (changing their zoom level, precision, or offset),
  similar to a multiplayer game in which one host is the decision maker

  - implies tracking multiple sessions per user, would have to be able to tell
    what changes in someone's view

  - limit the speed at which a user can zoom / pan? What about exact movement?

  - current implementation:
    - user creates a session (window), can adjust the window position manually as a
      REST resource
    - user can re-query what's visible in the window at any point in time (re-request for instance)
    - visibility responses return the nearest breakpoints so (hopefully) a client won't
      unnecessarilly request updates
    - could also be aware of what entries it's already seen and not reply with those (hard)
  - ideal implementation:
    - user gets new entries based on their updated window
    - could also be aware of entries that it's already seen (hard)
    - maybe implemented with websockets? That way realtime additions are pushed to the sessions
      within visibility, and async responses are more reasonable (no pending / timeouts for HTTP)
      - would require yet another proxy server, protocol development

- Secure the api schema

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
