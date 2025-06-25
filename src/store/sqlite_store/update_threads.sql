UPDATE
    threads
SET
    last_message_content = ?
    , last_message_created_at = ?
    , last_message_number = last_message_number + 1
    , replies_count = replies_count + 1
    , version = ?
WHERE
    id = ?
AND
    version = ?
