# :boom: @kemicofa/nuke_modules

:construction: _WIP: this has not yet been released_

TODOs:

- [ ] add tests
- [ ] add github workflow to test, build and publish on npm

A lightweight (648K) global CLI to safely purge all `node_modules` starting from your current working directory.

_note: works only on Unix like systems._

## Installation

```sh
npm -i -g @kemicofa/nuke_modules

nuke_modules --help
```

## Usage

```sh
nuke_modules [options]

A CLI to recursively purge all node_modules starting from your current working
directory.

Options:
      --version  Show version number                                   [boolean]
  -e, --emit     List all directories that would be purged.
                                                      [boolean] [default: false]
  -y, --yes      Auto accept delete confirmation.     [boolean] [default: false]
      --help     Show help                                             [boolean]
```
