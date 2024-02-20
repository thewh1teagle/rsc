# Rust Smart Cleaner

Rusty smart cleaner is a cross-platform Rust tool for cleaning operating system files.

It provides an easy way to remove unwanted files from your projects, using strategies such as gitignore.
By default it's navigating between the folders and clean them based on their `.gitignore` files.

# Downloads

Checkout [thewh1teagle.github.io/rsc](https://thewh1teagle.github.io/rsc/)

# Usage

```console
Usage: rsc [OPTIONS] <PATH>

Arguments:
  <PATH>  Root Path to clean from

Options:
  -d, --delete   Enable deletion
  -q, --quiet    Quiet mode
  -h, --help     Print help
  -V, --version  Print version
```

# Warning ⚠️

I am not responsible for any files you might lose. By default, it doesn't delete files and runs in 'dry' mode.

I recommend running it once to see, and only then pass `--delete` to enable deletion.
