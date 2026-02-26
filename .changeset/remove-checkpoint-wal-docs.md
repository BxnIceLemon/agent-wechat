---
"@agent-wechat/wechat": patch
---

Remove unnecessary WAL checkpoint task. WeChat DBs use WAL journal mode where readers and writers never block each other, and WeChat handles its own checkpoints.
