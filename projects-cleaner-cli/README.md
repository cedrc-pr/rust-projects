# Helper

```
Project deposit cleaner, it searches for project directories and deletes temporary files and executables

Usage: cargo run -- [OPTIONS] [DIRS]...

Arguments:
  [DIRS]...  Directories to ckeck

Options:
  -f, --force               Don't ask to keep or delete for : node_modules/, dist/, target/
  -m, --mrclean             Use mrclean
  -a, --add <ADD>...        to add extensions to the config file
  -r, --remove <REMOVE>...  to remove extensions form the config file
  -h, --help                Print help
```
