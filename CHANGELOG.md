<a name="v1.1.1"></a>
### v1.1.1 - 2023-07-01
- updated dependencies
- stripped binary (smaller)

<a name="v1.1.0"></a>
### v1.1.0 - 2021-12-05
- option to replace staged files with symlinks (unix only) - Fix #2

<a name="v1.0.1"></a>
### v1.0.1 - 2021-12-05
- option to write the report in a JSON file after staging phase - Fix #3

<a name="v1.0.0"></a>
### v1.0.0 - 2021-10-02
No reason not to call this a 1.0

<a name="v0.2.1"></a>
### v0.2.1 - 2021-07-14
- backdown logs a few things. To have log generated launch backdown with `BACKDOWN_LOG=debug backdown your/dir`
- change hash algorithm from SHA-256 to BLAKE3, which is slightly faster with same guarantees

<a name="v0.2.0"></a>
### v0.2.0 - 2021-07-12
- backdown proposes to remove in 1 question all duplicates with name like "thing (2).AVI" or "thing (3rd copy).png" when they're in the same directory than the "source"

<a name="v0.1.0"></a>
### v0.1.0 - 2021-07-11
- first public release
