local high_queue = KEYS[1]
local medium_queue = KEYS[2]
local low_queue = KEYS[3]
local jobs_hash = KEYS[4]

local high_limit = tonumber(ARGV[1])
local medium_limit = tonumber(ARGV[2])
local low_limit = tonumber(ARGV[3])

local job_id = ARGV[4]
local priority = ARGV[5]
local job_json = ARGV[6]

local high_len = redis.call("LLEN", high_queue)
local medium_len = redis.call("LLEN", medium_queue)
local low_len = redis.call("LLEN", low_queue)

local function add_job_atomically(queue, job_id, job_json)
    redis.call("RPUSH", queue, job_id)
    redis.call("HSET", jobs_hash, job_id, job_json)
    return 1
end

if priority == "High" then
    if high_len < high_limit then
        return add_job_atomically(high_queue, job_id, job_json)
    elseif medium_len < medium_limit then
        return add_job_atomically(medium_queue, job_id, job_json)
    elseif low_len < low_limit then
        return add_job_atomically(low_queue, job_id, job_json)
    else
        return 0
    end
end

if priority == "Medium" then
    if medium_len < medium_limit then
        return add_job_atomically(medium_queue, job_id, job_json)
    elseif low_len < low_limit then
        return add_job_atomically(low_queue, job_id, job_json)
    else
        return 0
    end
end

if priority == "Low" then
    if low_len < low_limit then
        return add_job_atomically(low_queue, job_id, job_json)
    else
        return 0
    end
end