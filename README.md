# tmux-thumbs

![](https://travis-ci.com/fcsonline/tmux-thumbs.svg?branch=master)

A lightning fast version of [tmux-fingers](https://github.com/Morantron/tmux-fingers) written in [Rust](https://www.rust-lang.org/) for copy pasting with vimium/vimperator like hints.

:warning: This plugin is active development.

## Matched patterns

- File paths
- File in diff
- Git SHAs
- Colors in hex
- Numbers ( 4+ digits )
- Hex numbers
- IP4 addresses
- kubernetes resources
- UUIDs

## Install

The easiest way right now is to install with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```
cargo install tmux-thumbs
tmux-thumbs -
```

or download the source code and compile it:

```
git clone git@github.com:fcsonline/tmux-thumbs.git && cd tmux-thumbs
cargo build --release
```


## Configuration

All `tmux-thumbs` configuration works settings custom paramenters to `tmux-thumbs` command.

Example:

```
tmux-thumbs -a qwerty -r -u
```

You can review all available options executing:

```
> tmux-thumbs --help

tmux-thumbs 0.2.2
A lightning fast version of tmux-fingers, copy/pasting tmux like vimium/vimperator

USAGE:
    tmux-thumbs [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -r, --reverse    Reverse the order for assigned hints
    -u, --unique     Don't show duplicated hints for the same match
    -V, --version    Prints version information

OPTIONS:
    -a, --alphabet <alphabet>                          Sets the alphabet [default: qwerty]
        --bg-color <background_color>                  Sets the background color for matches [default: black]
        --command <command>                            Pick command [default: tmux set-buffer {}]
        --fg-color <foreground_color>                  Sets the foregroud color for matches [default: green]
        --hint-bg-color <hint_background_color>        Sets the background color for hints [default: black]
        --hint-fg-color <hint_foreground_color>        Sets the foregroud color for hints [default: yellow]
    -p, --position <position>                          Hint position [default: left]
    -x, --regexp <regexp>...                           Use this regexp as extra pattern to match
        --select-fg-color <select_foreground_color>    Sets the foregroud color for selection [default: blue]
        --upcase-command <upcase_command>              Upcase command [default: tmux paste-buffer]

```

### Arguments

- **alphabet:** Choose which set of characters is used to build hints. Default [qwerty]
- **reverse:** Choose in which direction you want to assign hints. Useful to get shorter hints closer.
- **unique:** Choose if you want to assign the same hint for the same matched strings.
- **position:** Choose where do you want to show the hint in the matched string. Options (left, right). Default [left]
- **regexp:** Add extra pattern to match. This paramenter can have multiple instances.
- **command:** Choose whish command execute when you press a hint
- **upcase-command:** Choose which command execute when you press a upcase hint

- **bg-color:** Sets the background color for matches [default: black]
- **fg-color:** Sets the foregroud color for matches [default: green]
- **hint-bg-color:** Sets the background color for hints [default: black]
- **hint-fg-color:** Sets the foregroud color for hints [default: yellow]
- **select-fg-color:** Sets the foregroud color for selection [default: blue]

## Alphabets

This is the list of available alphabets:

- `numeric`: 1234567890
- `abcd`: abcd
- `qwerty`: asdfqwerzxcvjklmiuopghtybn
- `qwerty-homerow`: asdfjklgh
- `qwerty-left-hand`: asdfqwerzcxv
- `qwerty-right-hand`: jkluiopmyhn
- `azerty`: qsdfazerwxcvjklmuiopghtybn
- `azerty-homerow`: qsdfjkmgh
- `azerty-left-hand`: qsdfazerwxcv
- `azerty-right-hand`: jklmuiophyn
- `qwertz`: asdfqweryxcvjkluiopmghtzbn
- `qwertz-homerow`: asdfghjkl
- `qwertz-left-hand`: asdfqweryxcv
- `qwertz-right-hand`: jkluiopmhzn
- `dvorak`: aoeuqjkxpyhtnsgcrlmwvzfidb
- `dvorak-homerow`: aoeuhtnsid
- `dvorak-left-hand`: aoeupqjkyix
- `dvorak-right-hand`: htnsgcrlmwvz
- `colemak`: arstqwfpzxcvneioluymdhgjbk
- `colemak-homerow`: arstneiodh
- `colemak-left-hand`: arstqwfpzxcv
- `colemak-right-hand`: neioluymjhk

## Colors

This is the list of available colors:

- black
- red
- green
- yellow
- blue
- magenta
- cyan
- white
- default

## Extra features

- **Arrow navigation:** You can use the arrows to move arround between all matched items.
- **Auto paste:** If your last typed hint character is uppercase, you are going to pick and paste the desired hint.

# License

[MIT](https://github.com/fcsonline/tmux-thumbs/blob/master/LICENSE)
