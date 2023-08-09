Open Chronology Database
========================

This project is to define the database used in Open Chronology, its
implementation, and provide some technical discussion in some of its topics.

Building
--------

There is a `Dockerfile` associated with this project, and it's easiest to
just follow the `docker-compose.yml` file:

```bash
./build-init.sh && docker compose up
```

`build-init.sh` will generate a complete `init.sh` script to load the
extensions to PostgreSQL and also initialize the database with everything in
`sql/`. There is a slight order requirement between those SQL files - see the
`build-init.sh` script for details.

Philosophy
----------

Unline most websites and web servers, this Database needs to act as the
authoritative figure in all things security. Most web servers use executive
programming functionality (like PHP, .NET, Ruby, etc.) to accomplish web
security - if we were to do that, there's no guarantee that excessive use
of the database would be prevented, and we believe that encoding those usage
requirements directly into the database is most effective.

This is accomplished by using [PostgREST](https://postgrest.org/en/stable/)
as the HTTP-facing component, while PostgreSQL does all the heavy lifting
with schemas, row-layer security, `CHECK` cluases and other constraints,
stored procedures, and view tables.

Concepts
--------

All time is on the Rational number plane $\mathbf{Q}$. Values are stored
in PostgreSQL by using the [pgmp](https://www.varrazzo.com/pgmp/) extension.

Viewing the timeline could be seen as a 2-dimensional pagination system:

- _left_ and _right_ pagination is captured by the left and right bounds of the
  viewing window, where "scrubbing" (changing the offset) of the window would
  cause entries on the timeline to go in and out of the visible window.
- _in_ and _out_ pagination is done by "summarizing" the entries that are closer
  together than a specific "threshold" value, returning simply a count of
  entries that are beyond comprehensible view.

Glossary
--------

| Term            | Definition                                                                                                                                                                                                                                                                                                                                                                                                                   |
|:----------------|:-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Window          | View of a timeline, bounded by a left-bound $l$, a right-bound $r$, and a precision $p$, where $l < r$ and $0\\% < p \leq 100\\%$.                                                                                                                                                                                                                                                                                           |
| Time Point      | A single instance on the timeline, with time value being a rational number in $\mathbf{Q}$.                                                                                                                                                                                                                                                                                                                                  |
| Summary         | A manually-entered human readable summary instance, with an optional left-bound $l$ and right-bound $r$ (at least one has to be defined), and an optional minimum threshold $t_{min}$ and maximum threshold $t_{max}$ (no definition requirements). It also has a count $c$ of items not being viewed currently (being summarized), and an array of items $v$ currently in view that have been associated with this summary. |
| General Summary | A summary generated by the current window, not definted by anyone. This allows the user to see what isn't currently visible based on the current zoom level. It has a left-bound $l$, right-bound $r$, and count $c$ of items being summarized.                                                                                                                                                                              |
| Threshold       | A distance between points, measured as $\mathbf{Q}$.                                                                                                                                                                                                                                                                                                                                                                         |
| Precision       | A percentage of the window size, between $0\\%$ and $100\\%$.                                                                                                                                                                                                                                                                                                                                                                |

