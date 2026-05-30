1. `rg` is 100% sure in the `PATH`, call it like:

```bash
rg '<regex expr...>' '<file path>'
```

2. Prefer bash commands.

To output 1~10 lines of `file.txt`,

```bash
bash -c "sed -n '1,10p' 'file.txt'"
```

3. Stay the code idiom and clean.
4. Avoid unsafe blocks whenever possible.
5. A single module should never take more than 200 lines, until used strictly for data definition.
6. Typing your errors using thiserror.
7. For logging use `spdlog-rs`
8. For Command line support (only if explictly needed) use `argh`.
