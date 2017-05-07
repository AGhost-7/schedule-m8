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
docker run --rm -e SCHEDULE_M8_BIND_ADDR='0.0.0.0:8090' aghost7/schedule-m8
```

## API
The api only currently supports submitting a callback. You cannot
cancel a scheduled job.

### POST->/api/v1/schedule
Schedules a job. The message body's structure is the following:

```json
{
	"payload": "{}",
	"timestamp": 1494183499406,
	"url": "http://localhost:3000"
}
```

Where:
- `payload` is the body of the response.
- `timestamp` is the unix epoch in milliseconds of the time the scheduler
is to send to request back.
- `url` is the address to send the request to.

## Future development
Queue persistence is probably next on the list. Likely to use leveldb
to achieve this.
