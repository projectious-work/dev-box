-- svg.yazi — SVG previewer for yazi
-- Converts SVG to PNG using resvg, then renders via the built-in image previewer.
-- Requires: resvg in PATH.
-- Falls back to chafa (via image previewer) if conversion fails.

local function fail(msg)
	return Err(msg)
end

return {
	entry = function(self, job)
		local cache = ya.file_cache(job)
		if not cache then
			return fail("No cache path")
		end

		-- Only render if the cache file does not already exist
		if cache:exists() then
			return Image:new(job, cache):show()
		end

		-- Convert SVG → PNG via resvg
		-- --width / --height: cap output to preview dimensions
		local ok, err, code = Command("resvg")
			:args({
				"--width",
				tostring(job.area.w * 4),
				"--height",
				tostring(job.area.h * 4),
				tostring(job.file.url),
				tostring(cache),
			})
			:stdout(Command.NULL)
			:stderr(Command.NULL)
			:status()

		if not ok then
			return fail("resvg not found or failed (code " .. tostring(code) .. "): " .. tostring(err))
		end

		return Image:new(job, cache):show()
	end,
}
