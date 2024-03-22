local currentKey  = KEYS[1]           -- identifier including prefixes
local previousKey = KEYS[2]           -- key of the previous bucket
local tokens      = tonumber(ARGV[1]) -- tokens per window
local now         = ARGV[2]           -- current timestamp in milliseconds
local window      = ARGV[3]           -- interval in milliseconds
local incrementBy = ARGV[4]           -- increment rate per request at a given value, default is 1

local requestsInCurrentWindow = redis.call("GET", currentKey)
if requestsInCurrentWindow == false then
  requestsInCurrentWindow = 0
end

local requestsInPreviousWindow = redis.call("GET", previousKey)
if requestsInPreviousWindow == false then
  requestsInPreviousWindow = 0
end
local percentageInCurrent = ( now % window ) / window
-- weighted requests to consider from the previous window
requestsInPreviousWindow = math.floor(( incrementBy - percentageInCurrent ) * requestsInPreviousWindow)
if requestsInPreviousWindow + requestsInCurrentWindow >= tokens then
  return -1
end

local newValue = redis.call("INCRBY", currentKey, incrementBy)
if newValue == incrementBy then
  -- The first time this key is set, the value will be equal to incrementBy.
  -- So we only need the expire command once
  redis.call("PEXPIRE", currentKey, window * 2 + 1000) -- Enough time to overlap with a new window + 1 second
end
return tokens - ( newValue + requestsInPreviousWindow )
