# rmd

A terminal-based Markdown document viewer written in Rust. Browse and read Markdown files with a tree sidebar and syntax-highlighted content view.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **Tree sidebar** - Navigate directories and Markdown files
- **Syntax highlighting** - Monokai Dark theme for headings, code blocks, lists, tables, and more
- **Vim keybindings** - Full vim-style navigation (`hjkl`, `gg/G`, `Ctrl+u/d/b/f`, `/`, `n/N`)
- **Document search** - Search within documents with match highlighting
- **Global search** - Search across all Markdown files in the directory
- **Table rendering** - Unicode box-drawing characters for clean table display
- **Word wrapping** - Smart text wrapping at 120 characters (tables excluded)
- **Persistent settings** - Configurable options saved across sessions
- **Mouse support** - Scroll through documents with mouse wheel

## Installation

### From source

```bash
git clone https://github.com/yourusername/rmd.git
cd rmd
cargo build --release
```

The binary will be at `target/release/rmd`.

### Install to PATH

```bash
cargo install --path .
```

## Usage

```bash
# View Markdown files in current directory
rmd

# View Markdown files in a specific directory
rmd ./docs

# View Markdown files in an absolute path
rmd /path/to/markdown/files
```

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `l` / `→` / `Enter` | Open file / Expand directory |
| `h` / `←` | Collapse directory / Go to parent |
| `Tab` | Switch focus between sidebar and content |

### Scrolling

| Key | Action |
|-----|--------|
| `gg` | Go to top |
| `G` | Go to bottom |
| `Ctrl+u` | Half page up |
| `Ctrl+d` | Half page down |
| `Ctrl+b` | Full page up |
| `Ctrl+f` | Full page down |

### Search

| Key | Action |
|-----|--------|
| `/` | Search in document |
| `Ctrl+s` | Search all files |

### General

| Key | Action |
|-----|--------|
| `v` | Enter select mode (for copying text) |
| `?` | About |
| `q` / `Esc` | Quit |
| `Ctrl+p` | Settings |

## Development

### Prerequisites

- Rust 1.70 or later
- Cargo

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run directly
cargo run

# Run with arguments
cargo run -- ./docs
```

### Project Structure

```
src/
├── main.rs        # Entry point, CLI parsing, event loop
├── app.rs         # Application state management
├── file_tree.rs   # Directory tree structure for MD files
├── markdown.rs    # Markdown parsing and rendering
└── ui.rs          # Terminal UI rendering
```

### Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [clap](https://github.com/clap-rs/clap) - Command-line argument parsing
- [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) - Markdown parsing
- [walkdir](https://github.com/BurntSushi/walkdir) - Directory traversal

### Testing

```bash
cargo test
```

### Linting

```bash
cargo clippy
```

### Formatting

```bash
cargo fmt
```

## Supported Markdown Elements

- Headings (H1-H6) with visual underlines for H1/H2
- Paragraphs
- Tables (with Unicode box-drawing borders)
- Code blocks (fenced and indented)
- Inline code
- Unordered lists (bullet points)
- Ordered lists (numbered)
- Blockquotes
- Horizontal rules
- Emphasis and strong text
- Links and images (displayed as text)

## License

This project is released under the [Unlicense](https://unlicense.org/) - you can use, modify, distribute, and do whatever you want with this code. No restrictions, no attribution required.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
