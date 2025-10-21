# Notes

- Sync Preview

```bash
rclone sync <remote_name>:remote/path local/path \
    --progress --checksum --delete-during --transfers=8 \
    --checkers 16 --dry-run --log-level INFO \
    --log-file=rclone-sync.log
```

- Execute sync

```bash
rclone sync <remote_name>:remote/path local/path \
    --progress --checksum --delete-during --transfers=8 \
    --log-level INFO --log-file=rclone.sync.log
```

- Execute pull

```bash
rclone copy <remote_name>:remote/path local/path \
    --progress --checksum --transfers=4 --checkers=8 \
    --update --log-level INFO --log-file=rclone.copy.log
```
