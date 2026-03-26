-- eps.yazi — EPS previewer for yazi
-- Converts EPS to PNG using ghostscript, then renders via the built-in image previewer.
-- Requires: ghostscript (gs) in PATH.

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

		-- Convert EPS → PNG via ghostscript
		-- -dNOPAUSE -dBATCH: non-interactive batch mode
		-- -sDEVICE=png16m: 24-bit PNG output
		-- -r150: 150 DPI — enough detail without bloating cache
		-- -dEPSCrop: crop to the EPS bounding box
		-- -sOutputFile: write to cache path
		local ok, err, code = Command("gs")
			:args({
				"-q",
				"-dNOPAUSE",
				"-dBATCH",
				"-dSAFER",
				"-sDEVICE=png16m",
				"-r150",
				"-dEPSCrop",
				"-sOutputFile=" .. tostring(cache),
				tostring(job.file.url),
			})
			:stdout(Command.NULL)
			:stderr(Command.NULL)
			:status()

		if not ok then
			return fail("gs not found or failed (code " .. tostring(code) .. "): " .. tostring(err))
		end

		return Image:new(job, cache):show()
	end,
}
