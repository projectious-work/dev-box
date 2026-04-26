"""Signal modules for pk-retro.

Each module exports:

    def collect(ctx: dict) -> dict: ...

where ``ctx`` contains at minimum::

    {
        "repo_root": Path,        # resolved repo root
        "release": str,           # e.g. "v0.18.2"
        "since": str | None,      # ISO datetime or git ref (window start)
        "until": str | None,      # ISO datetime or git ref (window end)
        "verbose": bool,
        "mcp": dict,              # callable stubs (name -> callable)
    }

Returns a module-specific dict; the aggregator merges them into sections.
Unknown keys are silently ignored by the aggregator.
"""
