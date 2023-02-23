# skkmore

An implementation of a SKK dictionary server.

## Current features

- Supports UTF-8 only.
- Converts `▽きょう` into smth like `2023/02/23` or `2023-02-23`.
- The same applies to `おととい`, `きのう`, `あす`, `あした`, `あさって`.

## Example config

Spawn the server and add host/port to `dictionary_list`.

```
encoding=UTF-8,host=localhost,port=1178,type=server
file=$FCITX_CONFIG_DIR/skk/user.dict,mode=readwrite,type=file
file=/usr/share/skk/SKK-JISYO.L,mode=readonly,type=file
```

## Caveats

- The result of the conversion will be stored in `user.dict` 😞

