local key = KEYS[1]
local window = ARGV[1]

local r = redis.call("INCR", key)
if r == 1 then 
-- The key will upsert and incremented by one by default.
-- Set the key to expire at a given time 
redis.call("PEXPIRE", key, window)
end
    
return r
