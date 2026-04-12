-- =============================================================================
-- Yazi init.lua — aibox defaults
-- Runs on every Yazi startup. Register plugins that need setup here.
-- =============================================================================

-- git.yazi: show git status in the file list with explicit, visible signs.
-- Fetcher registration is in yazi.toml [plugin.prepend_fetchers].
th.git = th.git or {}
th.git.modified_sign = "M"
th.git.added_sign = "A"
th.git.deleted_sign = "D"
th.git.updated_sign = "U"
th.git.untracked_sign = "?"
th.git.ignored_sign = "I"

require("git"):setup {}
