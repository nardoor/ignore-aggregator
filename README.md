# Ignore-aggregator

## Foreword

My issue is that I couldn't figure out how to have `tar` ignore `.gitignore` pointed files in subdirectories.

This project consists in 30 lines of rust to solve this very specific problem.

There is most certainly some more efficient way to use `tar` and to ignore unwanted files indicated by `.gitignore` files.

However after trying a bit I couldn't succeed.
I wasn't satisfied with neither `--exclude-ignore=".gitignore"` nor `--exclude-vcs-ignores`.

- It seems `--exclude-vcs-ignores` doesn't recursively read .gitignore.

## What it does

I wanted a way to aggregate all `.gitignore`(s) in a recursive way under some directory.

It allows me to use the generated file as an input for my backup `tar` command.

```bash
# the directory I would like to backup
❯ tree DIR_TO_BACKUP
├── project1
│   ├── build
│   └── .gitignore
└── subdir
    ├── project2
    │   ├── .gitignore
    │   └── target
    └── project3
        ├── .gitignore
        └── node_modules

❯ bat DIR_TO_BACKUP/❯ bat **/.gitignore
───────┬────────────────────────────────────────────────
       │ File: project1/.gitignore
───────┼────────────────────────────────────────────────
   1   │ /build
───────┴────────────────────────────────────────────────
───────┬────────────────────────────────────────────────
       │ File: subdir/project2/.gitignore
───────┼────────────────────────────────────────────────
   1   │ target
───────┴────────────────────────────────────────────────
───────┬────────────────────────────────────────────────
       │ File: subdir/project3/.gitignore
───────┼────────────────────────────────────────────────
   1   │ node_modules/*
───────┴────────────────────────────────────────────────

❯ cd DIR_TO_BACKUP

❯ ignore-aggregator -r . -o TO_IGNORE
Scanning for .gitingore files
Found 3 git ignore files
"./subdir/project3/.gitignore"
"./subdir/project2/.gitignore"
"./project1/.gitignore"

❯ bat TO_IGNORE
───────┬────────────────────────────────────────────────
       │ File: TO_IGNORE
───────┼────────────────────────────────────────────────
   1   │ ./subdir/project3/node_modules/*
   2   │ ./subdir/project2/target
   3   │ ./project1/build
───────┴────────────────────────────────────────────────

# now I can feed this to my `tar` command
❯ tar --exclude-from=TO_IGNORE -cvaf backup.tar.gz  
```
