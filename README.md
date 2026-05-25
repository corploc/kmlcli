# kmlcli

KML/KMZ viewer in the terminal. Full TUI with braille map rendering, OpenStreetMap tile background, and interactive navigation.

No equivalent tool exists.

## Features

- **Full TUI** — braille-rendered map with OpenStreetMap tiles as background
- **KML/KMZ support** — points, lines, polygons, multi-geometry, folders, styles
- **Place names** — countries, cities, roads, water bodies from MVT tiles, filtered by zoom
- **Non-interactive mode** — `info`, `list`, `tree` subcommands with JSON output
- **Floating tree panel** — toggle with `t`, navigate with arrow keys
- **Zoom levels 0-14** — from world view to street level

## Install

```bash
cargo install --path .
```

Or grab a binary from [releases](https://github.com/corploc/kmlcli/releases).

## Usage

```bash
# TUI viewer (default)
kmlcli file.kml
kmlcli file.kmz

# Non-interactive
kmlcli info file.kml          # document metadata (JSON)
kmlcli list file.kml          # all placemarks/folders (JSON)
kmlcli tree file.kml          # structure tree
```

## Controls

| Key | Action |
|-----|--------|
| `scroll` | Pan vertical |
| `shift+scroll` | Pan horizontal |
| `ctrl+scroll` | Zoom |
| `hjkl` | Pan map |
| `+/-` | Zoom in/out |
| `Up/Down` | Navigate tree |
| `Enter` | Expand folder / center on element |
| `t` / `Tab` | Toggle tree panel |
| `q` | Quit |

## Stack

Rust. `ratatui` + `crossterm` for TUI, `kml` crate for parsing, `prost` for MVT protobuf decoding, `reqwest` for tile fetching from [OpenFreeMap](https://openfreemap.org).

## Architecture

```
KML/KMZ file
  -> parser (kml crate -> internal model)
  -> TUI (ratatui)
       -> map widget (braille canvas + tile background + KML overlay)
       -> floating tree panel
  -> or JSON output (info/list/tree subcommands)

Tiles: OpenFreeMap MVT -> protobuf decode -> prerender segments
       -> LRU cache (64 tiles) -> 4 parallel fetch workers
```

## License

MIT OR Apache-2.0
