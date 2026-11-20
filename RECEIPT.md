# Package receipt — ironfurnaces 0.2.2

| field | value |
|---|---|
| version | `0.2.2` |
| file | `ironfurnaces.hmod` |
| bytes | 281,821 |
| sha256 | `71590264a3b1857466ff1b032856e1990fc2427ab564667e247438662b1250d7` |
| source | this tag — the tree here is the source that produced the archive |
| game | 26.3 |
| horizon | `0b14f8acfd97fa2c750e42795249dcc981178ca4` |
| horizon-mixins | `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` |

## Verification of these exact bytes

`cargo test` 28 passed / 0 failed (the two new assertions failed 24/2 before the fix) · `clippy --all-targets -- -D warnings` 0 warnings · `fmt --check` clean · two builds `cmp`-identical · `check` exit 0 · `fits --build 0b14f8a` `made for` exit 0 · `fits` `0a875da fits` exit 0 · `verify-sdk-package.py` 1/1 · launcher leg `PASS` + `EXIT:0`; control `EXIT:1`; `SELFTEST=1` `EXIT:1`; `SELFTEST=2` (two-upgrade inversion) `EXIT:1`; the pre-fix run `EXIT:1` (`the two upgrades did not move the contents into the replacement in the same operation`); `check-e2e-logs --strict` 5/5.

The archive carries upstream's third-party credits verbatim in `NOTICE.txt` at its root (Apache-2.0), and the e2e asserts them before launching.
