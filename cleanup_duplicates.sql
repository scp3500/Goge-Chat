-- 查找重复的消息（相同的 contact_id, session_id, role, content, 且时间相近）
SELECT 
    id, 
    contact_id, 
    session_id, 
    role, 
    substr(content, 1, 50) as content_preview,
    created_at,
    COUNT(*) OVER (PARTITION BY contact_id, session_id, role, content) as duplicate_count
FROM social_messages
WHERE duplicate_count > 1
ORDER BY contact_id, session_id, created_at;

-- 删除重复的消息（保留 id 最小的那条）
DELETE FROM social_messages
WHERE id NOT IN (
    SELECT MIN(id)
    FROM social_messages
    GROUP BY contact_id, session_id, role, content, datetime(created_at, 'start of minute')
);

-- 验证清理结果
SELECT 
    contact_id,
    session_id,
    COUNT(*) as message_count
FROM social_messages
GROUP BY contact_id, session_id
ORDER BY contact_id, session_id;
