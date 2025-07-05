SELECT
      created_at
    , first_message_content
    , first_message_created_at
    , first_message_number
    , id
    , last_message_content
    , last_message_created_at
    , last_message_number
    , replies_count
    , version
FROM
    threads
ORDER BY
    last_message_created_at DESC
