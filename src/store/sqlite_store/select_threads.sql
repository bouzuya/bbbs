SELECT
      created_at
    , id
    , last_message_content
    , last_message_created_at
    , last_message_number
    , replies_count
    , version
FROM
    threads
WHERE
    thread_id = ?
