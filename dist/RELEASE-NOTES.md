# aibox v0.17.8

Patch release: migration briefing fix for sequential vs duplicate migration files.
No processkit version change — still compatible with v0.8.0.

## Migration briefing: sequential vs duplicate files

Fixes a gap in agent guidance when multiple CLI migration documents exist in
`context/migrations/`.

**The scenario:** An agent never reviewed the `0.17.5→0.17.6` migration document.
The owner then upgraded to v0.17.7, creating a `0.17.6→0.17.7` document. Under the
previous guidance ("the last file is authoritative, close earlier ones"), the agent
would discard the `0.17.5→0.17.6` document — missing the changes from that release.

**The fix:** The checklist now distinguishes two cases:

- **Same `from→to` range** (e.g. two `0.17.5-to-0.17.6.md` files from retried syncs):
  the most recent is authoritative; mark older ones as cancelled
- **Different ranges** (e.g. `0.17.5-to-0.17.6.md` alongside `0.17.6-to-0.17.7.md`):
  these are sequential migrations — both must be reviewed in chronological order,
  neither should be discarded
