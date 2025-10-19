# bbcli

![](https://raw.githubusercontent.com/hako/bbcli/refs/heads/master/bbcli.gif)

Browse BBC News like a hacker.

A terminal-based BBC News reader built with Rust and [ratatui](https://github.com/ratatui-org/ratatui), featuring a compact, numbered list interface with vim-like navigation.

> **Note:** This is a complete rewrite of bbcli in Rust.

# installation

## Using eget (Recommended)

Download pre-built binaries:

```bash
eget hako/bbcli
```

## GitHub Releases

Download pre-built binaries for your platform from [GitHub Releases](https://github.com/hako/bbcli/releases):

Available for:
- **Linux**: x86_64 (gnu/musl), aarch64 (gnu/musl)
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x86_64
- **BSD**: FreeBSD, NetBSD

## Using cargo-binstall (Also Recommended)

Fast binary installation without compiling:

```bash
# Install cargo-binstall first if needed
# See: https://github.com/cargo-bins/cargo-binstall

cargo binstall bbc-news-cli
```

## From crates.io

```bash
# Install from crates.io
cargo install bbc-news-cli
```

## From source

```bash
git clone https://github.com/hako/bbcli
cd bbcli
cargo build --release
cargo install --path .
```

# terminal compatibility

bbcli works best in modern terminals with full Unicode and image protocol support:

**Recommended terminals:**
- [WezTerm](https://wezfurlong.org/wezterm/) - Excellent image support (Sixel, Kitty, iTerm2)
- [Ghostty](https://github.com/ghostty-org/ghostty) - Fast, GPU-accelerated with Kitty protocol
- [Rio](https://raphamorim.io/rio/) - Modern, hardware-accelerated
- [Alacritty](https://alacritty.org/) - Fast, minimal (Halfblocks mode for images)
- [Kitty](https://sw.kovidgoyal.net/kitty/) - Full Kitty graphics protocol support
- [iTerm2](https://iterm2.com/) (macOS) - Inline images protocol

**Image protocols:**
- Press `p` to cycle through: Auto / Halfblocks / Sixel / Kitty
- Auto mode detects your terminal's capabilities
- Halfblocks works in all terminals (Unicode-based)

# usage

```bash
# Launch interactive TUI
bbcli

# Or use CLI mode (see below)
bbcli list
```

# cli mode:

bbcli includes a powerful CLI mode for quick access to headlines and articles without launching the full TUI.

## List Headlines

```bash
# List top stories
bbcli list

# List from specific feed
bbcli --feed world list
bbcli --feed technology list
bbcli --feed business list
```

## Open Article in Browser

```bash
# Open article #3 in browser
bbcli open 3

# Open from specific feed
bbcli --feed technology open 1
```

## Show Article in Terminal

```bash
# Read article #2 in terminal (reader mode)
bbcli show 2

# Show article from specific feed
bbcli --feed world show 5
```

## Available Feeds

Use `-f` or `--feed` with any of these feeds:
- `top` (default) - Top Stories
- `world` - World News
- `uk` - UK News
- `business` / `biz` - Business
- `politics` / `pol` - Politics
- `health` - Health
- `education` / `edu` - Education & Family
- `science` / `sci` - Science & Environment
- `technology` / `tech` - Technology
- `entertainment` / `ent` - Entertainment & Arts
- Regional: `england`, `scotland`, `wales`, `northern ireland`
- International: `africa`, `asia`, `europe`, `latin america`, `middle east`, `us & canada`

# tui keyboard shortcuts:

## Navigation
| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `G` | Scroll to bottom |
| `l` | Go to latest (top) |

## Actions
| Key | Action |
|-----|--------|
| `o` | Open article in browser |
| `O` | Open article in new tab |
| `Space` | Jump to current ticker article |
| `r` | Refresh news |

## Views
| Key | Action |
|-----|--------|
| `Tab` | Toggle preview pane |
| `a` / `Enter` | Open full article view |
| `f` | Open feed selector |
| `?` | Toggle help menu |

## Settings
| Key | Action |
|-----|--------|
| `s` | Cycle sort order (Default / Newest First / Oldest First) |
| `t` | Toggle date format (humanized / absolute) |
| `T` | Cycle theme (light / dark) |
| `p` | Cycle image protocol (Auto / Halfblocks / Sixel / Kitty) |

## Other
| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |

## Feed Menu

Press `f` to open the feed menu and switch between different BBC News feeds:

- Top Stories (default)
- World
- UK
- Business
- Politics
- Health
- Education & Family
- Science & Environment
- Technology
- Entertainment & Arts
- Regional feeds (England, Northern Ireland, Scotland, Wales)
- International regions (Africa, Asia, Europe, Latin America, Middle East, US & Canada)

Navigate with `j`/`k` or arrow keys, press `Enter` to select, `f` or `Esc` to close.

## Features

### Preview Pane
Press `Tab` to toggle the preview pane, which displays a 80/20 split showing:
- Article image (with configurable image protocol support)
- Title and description
- Publication date and feed name

### Full Article View
Press `Enter` or `a` to open the full article view with reader mode:
- Clean, distraction-free article text
- Scrollable with `j`/`k` or arrow keys
- Press `Tab`, `Enter`, or `Esc` to return to list view

### Story Sorting
Press `s` to cycle through sort orders:
- **Default**: RSS feed order (as received)
- **Newest First**: Sort by publication date, newest at top
- **Oldest First**: Sort by publication date, oldest at top

Sort order persists across manual and automatic refreshes.

### Ticker Navigation
The footer displays breaking news headlines from Top Stories feed, rotating every ~10 seconds. Press `Space` to instantly jump to the current ticker article in the feed.

### Auto-Refresh
The app automatically refreshes news every 5 minutes, keeping stories visible during refresh (no loading placeholder).

### Themes
Press `T` to cycle between light and dark themes. The default theme can be set in your config file.

### Offline Mode
bbcli automatically caches feeds and articles to disk for offline reading:

To clear the cache:
```bash
rm -rf ~/.bbcli/cache/
```

# configuration:

Custom keybindings can be defined in either:

`$HOME/.bbcli`

Or:

`$HOME/.config/bbcli`

See `example-config.toml` for all available options:

```toml
# Theme: "light" or "dark" (default: "dark")
theme = "dark"

[keybindings]
quit = 'q'
open = 'o'
open_new_tab = 'O'
refresh = 'r'
latest = 'l'
scroll_up = 'k'
scroll_down = 'j'
scroll_bottom = 'G'
```

Copy the example config:

```bash
cp example-config.toml ~/.bbcli
```

Or create your own custom configuration with preferred keybindings and theme.

# credits

Inspired and extended by the original bbcli python version.

# license

MIT
