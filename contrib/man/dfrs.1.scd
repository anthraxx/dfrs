dfrs(1)

# NAME

dfrs - Display file system space usage using graphs and colors

# SYNOPSIS

*dfrs* [_OPTION_]...  [_FILE_]...

# DESCRIPTION

*dfrs* displays the amount of disk space available on the file system containing each file name argument. If no file name is given, the space available on all currently mounted file systems is shown.

*dfrs*(1) is a tool similar to *df*(1) except that it is able to show a graph along with the data and is able to use colors.

Without any argument, size is displayed in human-readable format.

# OPTIONS

Show information about the file system on which each _FILE_ resides, or all regular file systems by default.

*-a*, *--more*
	Show more file systems; use *-a* twice to show all

*-aa*, *--all*
	Show all file systems

*-c*
	Force colors even if stdout is not a tty. This is useful with *watch -c*

*--color* [_WHEN_]
	Colorize the output; _WHEN_ can be 'auto' (default if omitted), 'always', or 'never'

*-i*, *--inodes*
	Show inode instead of block usage

*-h*, *--human-readable*
	Print sizes in powers of 1024 (e.g., 1023M)

*-H*, *--si*
	Print sizes in powers of 1000 (e.g., 1.1G)

*--total*
	Produce and show a grand total

*-l*, *--local*
	Limit listing to local file systems

*--no-aliases*
	Do not resolve file system shorthand aliases (e.g., LVM)

*--mounts* [_FILE_]
	File to get mount information from (e.g., /proc/mounts or /etc/mtab)

*-V*, *--version*
	Prints version information

*--help*
	Show a short help text.

# BUGS
	Bugs can be reported on the bug tracker _https://github.com/anthraxx/dfrs/issues_

# AUTHORS
	Levente Polyak
