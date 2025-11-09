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

- Create some form of 'hash' to verify wheter we should push or not the
data to the remote.

> This can be done by creating a hash field in the path_config and updating it
every time a push occurs.

- Sync preview
    Can be done with the remote explorer command but
    showing the diffs

- remote browser/explorer:

    ```bash
    rclone remote ls <remote_name>:<path> # rclone lsf
    ```

    ```bash
    rclone remote tree <remote_name>:<path> # rclone tree
    ```

    To explore remote files, but integrated in the CLI.
