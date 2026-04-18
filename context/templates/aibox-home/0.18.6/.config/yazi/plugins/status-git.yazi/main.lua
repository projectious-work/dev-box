--- @since 26.1.22
-- status-git.yazi — git branch + summary and disk free in status bar

local M = {}

local cached_git = ""
local cached_disk = ""

local save = ya.sync(function(_, git, disk)
	cached_git = git
	cached_disk = disk
	ui.render()
end)

local get_cwd = ya.sync(function()
	return tostring(cx.active.current.cwd)
end)

local function do_refresh()
	local cwd = get_cwd()

	-- Git branch + status
	local branch_out = Command("git"):cwd(cwd)
		:arg({ "rev-parse", "--abbrev-ref", "HEAD" })
		:stdout(Command.PIPED):stderr(Command.NULL):output()
	local branch = branch_out and branch_out.status and branch_out.status.success
		and branch_out.stdout:gsub("%s+$", "") or nil

	local git = ""
	if branch then
		local ps_out = Command("git"):cwd(cwd)
			:arg({ "--no-optional-locks", "status", "--porcelain", "-unormal" })
			:stdout(Command.PIPED):stderr(Command.NULL):output()

		local added, modified, deleted, untracked = 0, 0, 0, 0
		if ps_out and ps_out.stdout then
			for line in ps_out.stdout:gmatch("[^\n]+") do
				local xy = line:sub(1, 2)
				if xy == "??" then untracked = untracked + 1
				elseif xy:match("D") then deleted = deleted + 1
				elseif xy:match("[AC]") then added = added + 1
				else modified = modified + 1 end
			end
		end

		local parts = { " " .. branch }
		if added > 0 then parts[#parts + 1] = "+" .. added end
		if modified > 0 then parts[#parts + 1] = "~" .. modified end
		if deleted > 0 then parts[#parts + 1] = "-" .. deleted end
		if untracked > 0 then parts[#parts + 1] = "?" .. untracked end
		git = table.concat(parts, " ")
	end

	-- Disk free
	local disk = ""
	local out = Command("df"):arg({ "-h", cwd })
		:stdout(Command.PIPED):stderr(Command.NULL):output()
	if out and out.stdout then
		local i = 0
		for line in out.stdout:gmatch("[^\n]+") do
			i = i + 1
			if i == 2 then
				local avail = line:match("%S+%s+%S+%s+%S+%s+(%S+)")
				disk = avail and (avail .. " free") or ""
				break
			end
		end
	end

	save(git, disk)
end

function M:entry()
	do_refresh()
end

function M:fetch()
	do_refresh()
	return false
end

function M:setup()
	Status:children_add(function()
		if cached_git == "" then return ui.Span("") end
		return ui.Span(cached_git .. " "):fg("blue")
	end, 500, Status.LEFT)

	Status:children_add(function()
		if cached_disk == "" then return ui.Span("") end
		return ui.Span(" " .. cached_disk .. " "):fg("green")
	end, 500, Status.RIGHT)
end

return M
