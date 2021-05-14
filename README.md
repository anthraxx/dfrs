# dfrs

[![Build Status](https://img.shields.io/github/workflow/status/anthraxx/dfrs/CI)](https://github.com/anthraxx/dfrs/actions) [![Latest release](https://img.shields.io/github/v/release/anthraxx/dfrs)](https://github.com/anthraxx/dfrs/releases) [![crates.io version](https://img.shields.io/crates/v/dfrs.svg)](https://crates.io/crates/dfrs) [![License](https://img.shields.io/github/license/anthraxx/dfrs)](https://github.com/anthraxx/dfrs/blob/main/LICENSE)

Display file system space usage using graphs and colors

![](contrib/screenshot.png)

*dfrs* displays the amount of disk space available on the file system
containing each file name argument. If no file name is given, the space
available on all currently mounted file systems is shown.

*dfrs*(1) is a tool similar to *df*(1) except that it is able to show a graph
along with the data and is able to use colors.

Without any argument, size is displayed in human-readable format.

## Installation

<a href="https://repology.org/project/dfrs/versions"><img align="right" src="https://repology.org/badge/vertical-allrepos/dfrs.svg" alt="Packaging status"></a>

    cargo install dfrs

### Arch Linux

    pacman -S dfrs

### Debian sid/bullseye

    apt install dfrs

### Alpine

    apk add dfrs

## License

MIT
