- option to flatten output, such that it appears like this:
    [{"file_name":"todo.md","my_attr_1":"my_val_1","my_attr_2":"my_val_2"}]
- jsonl output option- some tools handle jsonl and it's probably easier to parse one object per
    line than to parse a huge array
- setsfattr
  - take data in the same format we produce it from `getsfattr`
- make cli like this (i.e. release three binaries: `sfattr`, `getsfattr` and `setfsattr`):
  - getsfattr
  - setsfattr
  - sfattr get
  - sfattr set
    - support something like setfattr set --kv key=value key2=value2 key3=value3?
    - read `man setfattr` on `-v` value encoding and `-x` attribute removal
- glob files? or just lean on shell globs?
- test
- lint
- GH Actions build/release
- Check license is compatible with dependencies
- OS support
  - Windows support? (i.e. explicit, with some GH actions tests or something)
  - Mac support? (i.e. explicit, with some GH actions tests or something)
  - Explicitly don't support Windows? (that's where the OsStr complexity comes from)
  - Just ignore all extended attributes that can't be represented as utf-8? (this is the current
      behaviour)
- options?
  - output data types?
    - https://serde.rs/#data-formats
    - table? ("raw text")
    - delimited (i.e. csv/ish)
    - or just let users use CLI converters?
- man page
- help text
- support attributes that can be represented as other than unicode- instead of the filter_map, use
    a map that transforms control characters to a different, probably string representation;
    probably the hexadecimal representation setfattr/getfattr use, e.g. \x0A for chr 0A
    - https://github.com/serde-rs/json/issues/550
- make calls to xattr::get concurrent?
- some nix stuff in the readme
  - `nix run 'github:partiallyordered/getsfattr *`
