# tmux-thumbs

![](https://travis-ci.com/fcsonline/tmux-thumbs.svg?branch=master)

A lightning fast version of [tmux-fingers](https://github.com/Morantron/tmux-fingers) written in [Rust](https://www.rust-lang.org/) for copy pasting with vimium/vimperator like hints.

## Usage

Press ( <kbd>prefix</kbd> + <kbd>Space</kbd> ) to highlist in you current tmux
visible pane all text that match specific pattern. Then press the highlighted
letter hint to yank the text in your tmux buffer.

### Matched patterns

- File paths
- File in diff
- Git SHAs
- IPFS CID's
- Colors in hex
- Numbers ( 4+ digits )
- Hex numbers
- Markdown urls
- IP4 addresses
- kubernetes resources
- UUIDs

These are the list of matched patterns that will be highlighted by default. If
you want to highlight a pattern that is not in this list you can add one or
more with `--regexp` parameter.

## Demo

[![demo](https://asciinema.org/a/232775.png?ts=1)](https://asciinema.org/a/232775?autoplay=1)

## Tmux integration

Clone the repo:

```
git clone https://github.com/fcsonline/tmux-thumbs ~/.tmux/plugins/tmux-thumbs
```

Compile it with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```
cd ~/.tmux/plugins/tmux-thumbs
cargo build --release
```

Source it in your `.tmux.conf`:

```
run-shell ~/.tmux/plugins/tmux-thumbs/tmux-thumbs.tmux
```

Reload TMUX conf by running:

```
tmux source-file ~/.tmux.conf
```

## Using Tmux Plugin Manager

You can add this line to your list of [TPM](https://github.com/tmux-plugins/tpm) plugins in `.tmux.conf`:

```
set -g @plugin 'fcsonline/tmux-thumbs'
```

To be able to install the plugin just hit <kbd>prefix</kbd> + <kbd>I</kbd>. You should now be able to use
the plugin!

## Configuration

If you want to customize how is shown your tmux-thumbs hints those all available
parameters to set your perfect profile.

NOTE: for changes to take effect, you'll need to source again your `.tmux.conf` file.

* [@thumbs-key](#thumbs-key)
* [@thumbs-alphabet](#thumbs-alphabet)
* [@thumbs-reverse](#thumbs-reverse)
* [@thumbs-unique](#thumbs-unique)
* [@thumbs-position](#thumbs-position)
* [@thumbs-regexp-N](#thumbs-regexp-N)
* [@thumbs-command](#thumbs-command)
* [@thumbs-upcase-command](#thumbs-upcase-command)
* [@thumbs-bg-color](#thumbs-bg-color)
* [@thumbs-fg-color](#thumbs-fg-color)
* [@thumbs-hint-bg-color](#thumbs-hint-bg-color)
* [@thumbs-hint-fg-color](#thumbs-hint-fg-color)
* [@thumbs-select-fg-color](#thumbs-select-fg-color)
* [@thumbs-select-bg-color](#thumbs-select-bg-color)
* [@thumbs-contrast](#thumbs-contrast)

### @thumbs-key

`default: space`

Choose which key is used to enter in thumbs mode.

For example:

```
set -g @thumbs-key F
```

### @thumbs-alphabet

`default: qwerty`

Choose which set of characters is used to build hints. Review all [available alphabets](#Alphabets)

For example:

```
set -g @thumbs-alphabet dvorak-homerow
```

### @thumbs-reverse

`default: disabled`

Choose in which direction you want to assign hints. Useful to get shorter hints closer to the cursor.

For example:

```
set -g @thumbs-reverse
```

### @thumbs-unique

`default: disabled`

Choose if you want to assign the same hint for the same matched strings.

For example:

```
set -g @thumbs-unique
```

### @thumbs-position

`default: left`

Choose where do you want to show the hint in the matched string. Options (left, right).

For example:

```
set -g @thumbs-position right
```

### @thumbs-regexp-N

Add extra patterns to match. This parameter can have multiple instances.

For example:

```
set -g @thumbs-regexp-1 '[a-z]+@[a-z]+.com' # Match emails
set -g @thumbs-regexp-2 '[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:' # Match MAC addresses
```

### @thumbs-command

`default: 'tmux set-buffer {}'`

Choose which command execute when you press a hint. `tmux-thumbs` will replace `{}` with the picked hint.

For example:

```
set -g @thumbs-command 'echo -n {} | pbcopy'
```

### @thumbs-upcase-command

`default: 'tmux set-buffer {} && tmux paste-buffer'`

Choose which command execute when you press a upcase hint. `tmux-thumbs` will replace `{}` with the picked hint.

For example:

```
set -g @thumbs-upcase-command 'echo -n {} | pbcopy'
```

### @thumbs-bg-color

`default: black`

Sets the background color for matches

For example:

```
set -g @thumbs-bg-color blue
```

### @thumbs-fg-color

`default: green`

Sets the foreground color for matches

For example:

```
set -g @thumbs-fg-color green
```

### @thumbs-hint-bg-color

`default: black`

Sets the background color for hints

For example:

```
set -g @thumbs-hint-bg-color blue
```

### @thumbs-hint-fg-color

`default: yellow`

Sets the foreground color for hints

For example:

```
set -g @thumbs-hint-fg-color green
```

### @thumbs-select-fg-color

`default: blue`

Sets the foreground color for selection

For example:

```
set -g @thumbs-select-fg-color red
```

### @thumbs-select-bg-color

`default: black`

Sets the background color for selection

For example:

```
set -g @thumbs-select-bg-color red
```

### @thumbs-contrast

`default: 0`

Displays hint character in square brackets for extra visibility.

For example:

```
set -g @thumbs-contrast 1
```

#### Colors

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

#### Alphabets

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

## Extra features

- **Arrow navigation:** You can use the arrows to move around between all matched items.
- **Auto paste:** If your last typed hint character is uppercase, you are going to pick and paste the desired hint.
- **Multi selection:** If you run thumb with multi selection mode you will be able to choose multiple hints pressing the desired letter and `Space` to finalize the selection.

## Tmux compatibility

This is the known list of versions of `tmux` compatible with `tmux-thumbs`:

| Version | Compatible |
|:-------:|:----------:|
|   3.0a  |      ❓    |
|   2.9a  |     ✅     |
|   2.8   |      ❓    |
|   2.7   |      ❓    |
|   2.6   |     ✅     |
|   2.5   |      ❓    |
|   2.4   |      ❓    |
|   2.3   |      ❓    |
|   1.8   |      ❓    |
|   1.7   |      ❓    |

If you can check hat `tmux-thumbs` is or is not compatible with some specific version of `tmux`, let me know.

## Standalone `thumbs`

This project started as a `tmux` plugin but after reviewing it with some
friends we decided to explore all the possibilities of decoupling thumbs from
`tmux`. You can install it with a simple command:

```
cargo install thumbs
```

And those are all available options:

```
thumbs 0.3.0
A lightning fast version copy/pasting like vimium/vimperator

USAGE:
    thumbs [FLAGS] [OPTIONS]

FLAGS:
    -c, --contrast    Put square brackets around hint for visibility
    -h, --help        Prints help information
    -m, --multi       Enable multi-selection
    -r, --reverse     Reverse the order for assigned hints
    -u, --unique      Don't show duplicated hints for the same match
    -V, --version     Prints version information

OPTIONS:
    -a, --alphabet <alphabet>                          Sets the alphabet [default: qwerty]
        --bg-color <background_color>                  Sets the background color for matches [default: black]
        --fg-color <foreground_color>                  Sets the foregroud color for matches [default: green]
    -f, --format <format>
            Specifies the out format for the picked hint. (%U: Upcase, %H: Hint) [default: %H]

        --hint-bg-color <hint_background_color>        Sets the background color for hints [default: black]
        --hint-fg-color <hint_foreground_color>        Sets the foregroud color for hints [default: yellow]
    -p, --position <position>                          Hint position [default: left]
    -x, --regexp <regexp>...                           Use this regexp as extra pattern to match
        --select-bg-color <select_background_color>    Sets the background color for selection [default: black]
        --select-fg-color <select_foreground_color>    Sets the foreground color for selection [default: blue]
    -t, --target <target>                              Stores the hint in the specified path
```


If you want to enjoy terminal hints, you can do things like this without `tmux`:

```
> alias pick='thumbs -u -r | xsel --clipboard -i'
> git log | pick
```

Or multi selection:

```
> git log | thumbs -m
1df9fa69c8831ac042c6466af81e65402ee2a007
4897dc4ecbd2ac90b17de95e00e9e75bb540e37f
```

Standalone `thumbs` has some similarities to [FZF](https://github.com/junegunn/fzf).

## Background

As I said, this project is based in [tmux-fingers](https://github.com/Morantron/tmux-fingers). Morantron did an extraordinary job, building all necessary pieces in Bash to achieve the text picker behaviour. He only deserves my gratitude for all the time I have been using [tmux-fingers](https://github.com/Morantron/tmux-fingers).

During a [Fosdem](https://fosdem.org/) conf, we had the idea to rewrite it to another language. He had these thoughts many times ago but it was hard to start from scratch. So, we decided to start playing with Node.js and [react-blessed](https://github.com/Yomguithereal/react-blessed), but we detected some unacceptable latency when the program booted. We didn't investigate much about this latency.

During those days another alternative appeared, called [tmux-picker](https://github.com/RTBHOUSE/tmux-picker), implemented in python and reusing many parts from [tmux-fingers](https://github.com/Morantron/tmux-fingers). It was nice, because it was fast and added original terminal color support.

I was curious to know if this was possible to be written in [Rust](https://www.rust-lang.org/), and soon I realized that was something doable. The ability to implement tests for all critic parts of the application give you a great confidence about it. On the other hand, Rust has an awesome community that lets you achieve this kind of project in a short period of time.

## Roadmap

- [X] Support multi selection
- [X] Decouple `tmux-thumbs` from `tmux`
- [ ] Code [Kitty](https://github.com/kovidgoyal/kitty) plugin, now that `thumbs` can run standalone

## Contribute

This project started as a side project to learn Rust, so I'm sure that is full
of mistakes and areas to be improve. If you think you can tweak the code to
make it better, I'll really appreaciate a pull request. ;)

# License

[MIT](https://github.com/fcsonline/tmux-thumbs/blob/master/LICENSE)
