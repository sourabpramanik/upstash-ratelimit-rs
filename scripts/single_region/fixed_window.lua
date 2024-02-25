local key     = KEYS[1]
local window  = ARGV[1]

local value = redis.call('INCR', key)
if value == 1 then 
-- The first time this key is set, the value will be 1.
-- So we only need the expire command once
redis.call('PEXPIRE', key, window)
end

return value
