UPDATE
    thread_event_streams
SET
    version = ?
WHERE
    id = ?
AND
    version = ?
