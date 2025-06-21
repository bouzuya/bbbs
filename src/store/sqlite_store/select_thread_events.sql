SELECT
    at,
    content,
    id,
    kind,
    thread_id,
    version
FROM
    thread_events
WHERE
    thread_id = ?
ORDER BY
    version ASC

