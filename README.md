# rsc

Rusty smart cleaner is a cross-platform Rust tool for cleaning operating system files.

It provides an easy way to remove unwanted files from your projects, using strategies such as gitignore.
By default it's navigating between the folders and clean them based on their `.gitignore` files.

# Installation
To install rsc, please visit the [rsc website](https://thewh1teagle.github.io/rsc/)- which is generated by oranda!

# Usage

```console
Usage: rsc [OPTIONS] <PATH>

Arguments:
  <PATH>  Root Path to clean from

Options:
  -d, --delete         Enable deletion
  -q, --quiet          Quiet mode
  -i, --ignore-errors  Ignore errors such as permission denied
  -h, --help           Print help
  -V, --version        Print version
```

# Contributing
Feel free to open a new issue or pull request if you notice something off or have a new feature request!

# Warning ⚠️

I am not responsible for any files you might lose. By default, it doesn't delete files and runs in 'dry' mode.

I recommend running it once to see, and only then pass `--delete` to enable deletion.
