local WINDOWS = package.config:sub(1, 1) == "\\"
local DEBUG = true
package.cpath = package.cpath .. ";./target/"..(DEBUG and "debug/" or "release/")..(WINDOWS and "?.dll;" or "lib?.so;")

---@param value any
---@param parts string[]?
---@param depth integer?
local function tostring_value(value, parts, depth)
	parts = parts or {}
	if type(value) == "string" then
		local quote_mark = "\""
		value = value:gsub("\n", "\\n"):gsub("\r", "\\r"):gsub("\t", "\\t")
		if value:find("\"") then
			if value:find("'") then
				value = value:gsub("\"", "\\\"")
			else
				quote_mark = "'"
			end
		end
		table.insert(parts, ("%s%s%s"):format(quote_mark, value, quote_mark))
	elseif type(value) == "table" and depth ~= -1 then
		depth = depth or 0
		table.insert(parts, "{\n")
		for i, v in pairs(value) do
			table.insert(parts, string.rep("    ", depth+1))
			tostring_value(i, parts, -1)
			table.insert(parts, " = ")
			tostring_value(v, parts, depth+1)
			table.insert(parts, ",\n")
		end
		table.insert(parts, string.rep("    ", depth) .. "}")
	else
		table.insert(parts, tostring(value))
	end
	return parts
end


---@type LuaNotify
local LuaNotify = require "luanotify"

-- Create a new watcher.
local watcher = LuaNotify.new()
print("watcher", assert(watcher))

-- Filter only txt files. (any valid glob acording to https://docs.rs/glob/latest/glob/)
print(assert(watcher:filter_by_glob("*.txt")))

print("watch", watcher:watch("./I_DONT_EXIST", true))

-- Watch a directory or a file recursively.
print("watch", assert(watcher:watch(".", true)))
-- unwatching does not work on sub-dirs of a watched directory.
print("unwatch", assert(watcher:unwatch("./target")))  -- This does nothing

-- We have began watching, so lets create/modify some stuff!
print("Testing file open in write and writing some data.")
local f = assert(io.open("t.txt", "w"))
for i=1,10 do
	f:write("testing...")
end
f:close()

-- Windows can't detect opens and reads.
-- Linux can.
print("Testing file open in read")
local f = assert(io.open("t.txt", "r"))
f:close()

print("Now lets retrive all the data.")
while true do
	-- Will return fs event data if available, otherwise will return nil.
	local event = watcher:poll()
	if event then
		print(table.concat(tostring_value(event), ""))
	else
		break
	end
end
