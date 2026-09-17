# Belmarsh

A command-line tool for analyzing the inner module dependencies of a repository. Belmarsh
walks a codebase, resolves imports between files, groups files into modules, and reports on
statistics, dependency graphs, and validation issues such as circular dependencies and
improper barrel-file imports.

## Installation

Belmarsh is a Rust binary built with Cargo.

### Build from source

```sh
git clone https://github.com/cleavera/belmarsh.git
cd belmarsh
cargo build --release
```

The compiled binary will be available at `target/release/belmarsh`.

### Install to your PATH

To install `belmarsh` so it can be run from anywhere:

```sh
cargo install --path .
```

This installs the `belmarsh` executable to `~/.cargo/bin` (make sure that directory is on
your `PATH`). You can then run it from any directory:

```sh
belmarsh --help
```

## Usage

```sh
belmarsh <COMMAND> <REPOSITORY_PATH> [OPTIONS]
```

`<REPOSITORY_PATH>` is the path to the repository (or subdirectory) you want to analyze.

### Common options

These options are available on all subcommands:

| Option | Description |
| --- | --- |
| `--skip-folders <FOLDER_NAME>` | Folder(s) to skip when walking the repository (repeatable). Defaults to `node_modules`. |
| `--module-mapping <ALIAS:PATH>` | Map an import alias to a path (repeatable), e.g. `--module-mapping @app:./src/app`. |

### Commands

#### `statistics`

Generate statistics about the repository's modules and their external imports.

```sh
belmarsh statistics ./path/to/repo
```

Prints the number of modules, the total number of imports made from outside their own
module, and the total number of files checked.

#### `graph`

Output a module dependency graph in [Graphviz DOT](https://graphviz.org/doc/info/lang.html)
format.

```sh
belmarsh graph ./path/to/repo > graph.dot
dot -Tpng graph.dot -o graph.png
```

#### `inspect`

List all files that draw in a dependency from outside their own module.

```sh
belmarsh inspect ./path/to/repo
```

Use `--filter-from <MODULE_NAME>` to restrict the output to dependencies originating from a
specific module.

#### `validate`

Run one or more validation checks against the repository and report any failures:

```sh
belmarsh validate ./path/to/repo
```

By default (no flags) all checks run. Pass one or more flags to run only specific checks:

| Flag | Description |
| --- | --- |
| `--circular-modules` | Detect circular dependencies between modules. |
| `--circular-files` | Detect circular dependencies between individual files. |
| `--external-barrel-imports` | Detect external imports that don't go through a barrel file. |
| `--barrel-imports-barrel` | Detect barrel files that import another barrel file. |

Example, running only the circular dependency checks:

```sh
belmarsh validate ./path/to/repo --circular-modules --circular-files
```

## Example

```sh
belmarsh validate . --module-mapping @app:./src/app --skip-folders dist --skip-folders node_modules
```

## Development

Run the test suite with:

```sh
cargo test
```
