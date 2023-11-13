[![release](https://github.com/timonviola/dotman.rs/actions/workflows/ci.yml/badge.svg?branch=trunk)](https://github.com/timonviola/dotman.rs/actions/workflows/ci.yml)

# DOTMAN (yet another dotfile manager)

Sync configs semi-automatically.

## Why?
- I want to learn rust
- No more manual managing of vim, tmux, git aliases etc. between computers.

# How to use
create a dotman.toml file under your $HOME directory. (Is you want to change the location/name of this file change `DOTMAN_HOME` env var.)


# What it does?
Centralizes your config files to store them in a single folder (repo).

`dotman` goes through the `dotman.toml` file entries and creates symbolic links is your file system.

# What it does NOT?
- no git workflow automation
- ?

# Get started
- install
- set up dotman.toml:
- run `dotman link` (creates symbolic links)
- run `dotman link -d (--delete) (delete links if exist)
- run `dotman link tmux (only create link for target entry)
- `dotman show` (list managed files)
- `dotman show --unmanaged` (show you the files is the current repo which are not managed/not part of `dotman.toml`)

# Tips
If you make changes to your local config make sure you push those
If you use another workstation, remember to pull your latest changes

If there are new config files you need to create links for those on your system.

## Dangling links
dotman works with symlinks, a dangling link is when the file referenced by the symlink, does not exist.



# Development

## v1.0.0
- [ ] read toml file
- [ ] create symbolic links (from toml definition)

# Symlinks

dotman provides you the option to use hardlinks and softlinks (default). The following is a quick summary to get you up to speed and give some background.

## What are symlinks?
Symlinks can be interpreted as pointers on the filesystem. Instead of copying the contents of the file we just point to them.

You can find out more about symlinks on the (manpage)[https://linux.die.net/man/7/symlink].

## Softlinks vs hardlinks
__you should really read the manpage, it's better written than this__
A Hard Link is a copy of the original file that serves as a pointer to the same. file, allowing it to be accessed even if the original file is deleted or relocated. Unlike soft links, modifications to hard-linked files affect other files, and the hard link remains active even if the source file is deleted from the system.

The soft link serves as a pointer to another file without the file's actual contents. It allows the users to delete or the soft links without affecting the original file's contents. You may also use soft links to link files across the file system. Generally, the soft link is an alias for the original file or directory.

| Softlink | Hardlink |
| -------- | -------- |
| `ln -s` | `ln` |
| modifications not reflected in original | modifications reflected in original |
| does not use inode | uses inode |
| less performant | more performant |
| any partition (outside file system) | only on own partition |
| can link to directory and file | can link only to file |


# Roadmap
## V1.0.0
- [ ] implement symlink feature
    - remove MyPath bonanza to use onlf PathBuf
    - implement tests
- [ ] subcommands rework:
    - return Result
    - purge, link return number of links changed
    - add interactive mode

- [ ] check if `target` is file or dir
    - if dir: use `source` file name and create symlink at `target` location
      elif file: use `target` as a fully qualified path
    - use `try_exists` to travers sylinks (https://doc.rust-lang.org/stable/std/path/struct.Path.html#method.try_exists)
    - if target symlink exists do nothing
      else create (symlink)[https://doc.rust-lang.org/std/fs/fn.soft_link.html]
