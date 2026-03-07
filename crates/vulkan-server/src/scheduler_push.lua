local high_queue = KEYS[1]
local medium_queue = KEYS[2]
local low_queue = KEYS[3]

local high_limit = tonumber(ARGV[1])
local medium_limit = tonumber(ARGV[2])
local low_limit = tonumber(ARGV[3])

local job_id = ARGV[4]
local score = tonumber(ARGV[5])
local priority = ARGV[6]

local high_len = redis.call("ZCARD", high_queue)
local medium_len = redis.call("ZCARD", medium_queue)
local low_len = redis.call("ZCARD", low_queue)

if priority == "High" then
    if high_len < high_limit then
        redis.call("ZADD", high_queue, score, job_id)
        return 1
    elseif medium_len < medium_limit then
        redis.call("ZADD", medium_queue, score, job_id)
        return 1
    elseif low_len < low_limit then
        redis.call("ZADD", low_queue, score, job_id)
        return 1
    else
        return 0
    end
end

if priority == "Medium" then
    if medium_len < medium_limit then
        redis.call("ZADD", medium_queue, score, job_id)
        return 1
    elseif low_len < low_limit then
        redis.call("ZADD", low_queue, score, job_id)
        return 1
    else
        return 0
    end
end

if priority == "Low" then
    if low_len < low_limit then
        redis.call("ZADD", low_queue, score, job_id)
        return 1
    else
        return 0
    end
end