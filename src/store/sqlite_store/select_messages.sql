SELECT
      content
    , created_at
    , thread_id
    , number
FROM
    messages
WHERE
    thread_id = ?
ORDER BY
    number ASC
