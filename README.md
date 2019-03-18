A CLI to control the i3 tiling window manager.

# Introduction

The idea of this tool is that it provides an extensible collection of commands, and handles sending
them through the i3 ipc. Currently it has 1 command - open a terminal in the current working
directory.

# Usage

```sh
i3ctl <subcommand>
where
   <subcommand> = {term | ..more might be added }
```

# Commands

 - `term <terminal name>` - creates a new terminal window with cwd set to the current directory.

   I map this to a key parttern in vim so I can open a terminal without having to then navigate to
   my current directory.


