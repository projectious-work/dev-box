-- svg.yazi — SVG previewer for yazi
-- Converts SVG to PNG using resvg (x86_64) or rsvg-convert (aarch64 fallback).

return {
	entry = function(self, job)
		local cache = ya.file_cache(job)
		if not cache then
			return Err("No cache path")
		end

		if cache:exists() then
			return Image:new(job, cache):show()
		end

		-- Try resvg first (high quality, static binary — available on x86_64)
		local ok = Command("resvg")
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

		if ok then
			return Image:new(job, cache):show()
		end

		-- Fallback: rsvg-convert (from librsvg2-bin, available on all architectures)
		ok = Command("rsvg-convert")
			:args({
				"--width", tostring(job.area.w * 4),
				"--height", tostring(job.area.h * 4),
				"--keep-aspect-ratio",
				"--output", tostring(cache),
				tostring(job.file.url),
			})
			:stdout(Command.NULL)
			:stderr(Command.NULL)
			:status()

		if ok then
			return Image:new(job, cache):show()
		end

		return Err("SVG preview failed: install resvg or librsvg2-bin")
	end,
}
