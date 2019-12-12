# Schedule-m8
gr8 scheduling m8.

## Motivation
I wanted a mock of quartz-http, primary motivation being that for
local development the microservice takes too much memory. I also
noticed that it is very slow to start up and shut down. This replaces
the http service with something that makes more sense for developing.

## Usage (docker)
Here's an example drop-in replacement for quartz-http:
```
docker run --rm -p 8090:8090 -e SCHEDULE_M8_BIND_ADDR='0.0.0.0:8090' aghost7/schedule-m8
```

## API

### POST->/api/job
Schedules a job. The message body's structure is the following:

```json
{
	"method": "POST",
	"body": "{}",
	"timestamp": 1494183499406,
	"url": "http://localhost:3000"
}
```

Where:
- `method` is the http method to send the callback to.
- `body` is the body of the response.
- `timestamp` is the unix epoch in milliseconds of the time the scheduler
is to send to request back.
- `url` is the address to send the request to.

Returns:
```json
{
	"method": "POST",
	"body": "{}",
	"timestamp": 1494183499406,
	"url": "http://localhost:3000",
	"id": "123-123-1234"
}
```

### POST -> /api/cron
Schedule a cron job. The message's structure is the following:
```json
{
	"method": "POST",
	"body": "{}"
	"timestamp": 1494183499406,
	"schedule": "@daily"
}
```
Where:
- `method` is the http method to send the callback to.
- `body` is the body of the response.
- `schedule` is a cron expression. For more information, see the [cron][cron]
crate.
- `url` is the address to send the request to.

[cron]: https://github.com/zslayton/cron

### DELETE -> /api/job/:id
Delete a job. Returns a 204 on success.

### DELETE -> /api/job
Delete all jobs.
