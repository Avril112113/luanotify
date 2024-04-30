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


---@class LuaNotify.Watcher
---@field watch fun(self, path:string, recursive:boolean?):boolean,string?
---@field unwatch fun(self, path:string):boolean,string?
---@field poll fun(self):LuaNotify.Event?
---@field filter_by_glob fun(self, glob:string)

---@class LuaNotify.Event

---@class LuaNotify
---@field new fun():LuaNotify.Watcher
local LuaNotify = require "luanotify"


local watcher = LuaNotify.new()
print(assert(watcher))

print(watcher:filter_by_glob("*.txt"))

print(assert(watcher:watch(".", true)))


local f = assert(io.open("t.txt", "w"))
f:write("testing...")
f:close()

local f = assert(io.open("t.txt", "r"))
f:close()

while true do
	local event = watcher:poll()
	if event then
		print(table.concat(tostring_value(event), ""))
	end
end
