local key           = KEYS[1]
local window        = ARGV[1]
local incrementBy   = ARGV[2] -- increment rate per request at a given value, default is 1

local r = redis.call("INCRBY", key, incrementBy)
if r == incrementBy then
-- The first time this key is set, the value will be equal to incrementBy.
-- So we only need the expire command once
redis.call("PEXPIRE", key, window)
end

return r
