--- @since 26.1.22
-- dir-preview.yazi — columnar directory preview with git status
-- Columns: git | icon+name | size | date | owner | permissions

local M = {}

local function uid_map()
	local map = {}
	local f = io.open("/etc/passwd")
	if f then
		for line in f:lines() do
			local name, uid = line:match("^([^:]+):[^:]*:(%d+)")
			if uid then map[tonumber(uid)] = name end
		end
		f:close()
	end
	return map
end

local function git_status(dir)
	local map = {}
	-- Get the directory's path relative to the repo root
	local prefix_out = Command("git"):cwd(tostring(dir))
		:arg({ "rev-parse", "--show-prefix" }):output()
	local prefix = prefix_out and prefix_out.stdout:gsub("%s+$", "") or ""
	local out = Command("git"):cwd(tostring(dir))
		:arg({ "--no-optional-locks", "-c", "core.quotePath=",
		       "status", "--porcelain", "-unormal", "--no-renames", "--ignored=matching", "--", "." })
		:output()
	if not out then return map end
	for line in out.stdout:gmatch("[^\r\n]+") do
		local signs, path = line:sub(1, 2), line:sub(4):gsub('"', ""):gsub("/$", "")
		-- Strip the repo-relative prefix to get paths relative to this directory
		if prefix ~= "" and path:sub(1, #prefix) == prefix then
			path = path:sub(#prefix + 1)
		end
		local base = path:match("^([^/]+)")
		if base and not map[base] then map[base] = signs end
	end
	return map
end

-- Map porcelain two-char codes to single-char signs matching the main list
local GIT_SIGNS = {
	["!!"] = "I", ["??"] = "?",
	["A "] = "A", ["AM"] = "A",
	[" M"] = "M", ["M "] = "M", ["MM"] = "M",
	[" D"] = "D", ["D "] = "D",
	["UU"] = "U",
}
-- Read theme git colors (set by yazi theme.toml [git] section)
local t = th.git or {}
-- Direct status (files): theme-aware styles
local GIT_STYLES = {
	["?"] = t.untracked or ui.Style():fg("magenta"),
	["I"] = t.ignored or ui.Style():fg("darkgray"),
	["A"] = t.added or ui.Style():fg("green"),
	["M"] = t.modified or ui.Style():fg("yellow"),
	["D"] = t.deleted or ui.Style():fg("red"),
	["U"] = t.updated or ui.Style():fg("yellow"),
}
-- Inherited status (directories): same styles but dimmed
local GIT_STYLES_DIM = {
	["?"] = (t.untracked or ui.Style():fg("magenta")):dim(),
	["I"] = t.ignored or ui.Style():fg("darkgray"),
	["A"] = (t.added or ui.Style():fg("green")):dim(),
	["M"] = (t.modified or ui.Style():fg("yellow")):dim(),
	["D"] = (t.deleted or ui.Style():fg("red")):dim(),
	["U"] = (t.updated or ui.Style():fg("yellow")):dim(),
}

local function fmt_size(n)
	local s
	if not n or n < 0 then s = "-"
	elseif n < 1024 then s = string.format("%d", n)
	elseif n < 1048576 then s = string.format("%.0fK", n / 1024)
	elseif n < 1073741824 then s = string.format("%.1fM", n / 1048576)
	else s = string.format("%.1fG", n / 1073741824)
	end
	return string.format("%6s", s)
end

local function pad(s, w) return s .. string.rep(" ", math.max(0, w - #s)) end
local function trunc(s, w) return #s <= w and s or s:sub(1, w - 1) .. "~" end

function M:peek(job)
	local files, err = fs.read_dir(job.file.url, { resolve = true })
	if not files then
		ya.preview_widget(job, ui.Text { ui.Line(tostring(err or "empty")) }:area(job.area))
		return
	end

	table.sort(files, function(a, b)
		if a.cha.is_dir ~= b.cha.is_dir then return a.cha.is_dir end
		return a.name:lower() < b.name:lower()
	end)

	local git = git_status(job.file.url)
	local owners = uid_map()
	local total, limit = #files, job.area.h
	-- fixed: git(2) + spc(1) + icon(2) + spc(1) + size(6) + spc(1) + date(13) + spc(1) + owner(9) + spc(1) + perm(10) = 47
	local name_w = math.max(8, job.area.w - 47)
	local lines = {}

	for i = job.skip + 1, math.min(total, job.skip + limit) do
		local f = files[i]
		local c = f.cha
		local name = f.name .. (c.is_dir and "/" or "")
		local raw = git[f.name] or ""
		local gs = GIT_SIGNS[raw] or ""
		local is_inherited = c.is_dir and gs ~= "" and gs ~= "I"
		local gs_style = is_inherited and GIT_STYLES_DIM[gs] or GIT_STYLES[gs]
		local ignored = gs == "I"
		gs = is_inherited and (gs:lower() .. " ") or (gs ~= "" and (gs .. " ") or "  ")
		local icon = f:icon()
		local size
		if c.is_dir then
			local children = fs.read_dir(f.url, {})
			local n = children and #children or 0
			size = string.format("%6s", "[" .. n .. "]")
		else
			size = fmt_size(c.len)
		end
		local date = c.mtime and os.date("%b %d %H:%M", math.floor(c.mtime)) or "            "
		local owner = pad(trunc(owners[c.uid] or tostring(c.uid or "?"), 8), 8)
		local perm = c:perm() or "----------"
		local ign = ignored and GIT_STYLES["I"] or nil

		lines[#lines + 1] = ui.Line {
			gs_style and ui.Span(gs):style(gs_style) or ui.Span(gs), ui.Span(" "),
			icon and (ign and ui.Span(icon.text .. " "):style(ign) or ui.Span(icon.text .. " "):style(icon.style)) or ui.Span("  "),
			ign and ui.Span(pad(trunc(name, name_w), name_w)):style(ign) or ui.Span(pad(trunc(name, name_w), name_w)),
			ui.Span(" " .. size):style(ign or ui.Style():fg("green")),
			ui.Span(" " .. date):style(ign or ui.Style():fg("cyan")),
			ui.Span(" " .. owner):style(ign or ui.Style():fg("yellow")),
			ui.Span(" " .. perm):dim(),
		}
	end

	if job.skip > 0 and total <= job.skip then
		ya.emit("peek", { math.max(0, total - limit), only_if = job.file.url, upper_bound = true })
	else
		ya.preview_widget(job, ui.Text(lines):area(job.area))
	end
end

function M:seek(job) require("code"):seek(job) end

return M
