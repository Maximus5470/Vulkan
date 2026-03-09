local high_queue = KEYS[1]
local medium_queue = KEYS[2]
local low_queue = KEYS[3]

local high_limit = tonumber(ARGV[1])
local medium_limit = tonumber(ARGV[2])
local low_limit = tonumber(ARGV[3])

local job_id = ARGV[4]
local priority = ARGV[5]

local high_len = redis.call("LLEN", high_queue)
local medium_len = redis.call("LLEN", medium_queue)
local low_len = redis.call("LLEN", low_queue)

if priority == "High" then
    if high_len < high_limit then
        redis.call("RPUSH", high_queue, job_id)
        return 1
    elseif medium_len < medium_limit then
        redis.call("RPUSH", medium_queue, job_id)
        return 1
    elseif low_len < low_limit then
        redis.call("RPUSH", low_queue, job_id)
        return 1
    else
        return 0
    end
end

if priority == "Medium" then
    if medium_len < medium_limit then
        redis.call("RPUSH", medium_queue, job_id)
        return 1
    elseif low_len < low_limit then
        redis.call("RPUSH", low_queue, job_id)
        return 1
    else
        return 0
    end
end

if priority == "Low" then
    if low_len < low_limit then
        redis.call("RPUSH", low_queue, job_id)
        return 1
    else
        return 0
    end
end