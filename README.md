# getsfattr

Like `getfattr` but structured, hence `getsfattr`. Instead of an interface that's tricky to use,
just dumps all extended file-system attributes to stdout as json like this:
```json
[
  {
    "file_name": "README.md",
    "attrs": {
      "user.myattr": "somevalue"
    }
  }
]
```

## Get
### Build Locally
```sh
cargo build
```

## Support
The author's machine only. Should work on Linux machines generally. File an issue if you like.
