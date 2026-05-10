# kmlcli — KML/KMZ Terminal Viewer

## Summary

CLI tool in Rust that renders KML/KMZ files in the terminal. Two modes: full TUI with interactive navigation + braille map rendering, and non-interactive subcommands for scripting.

No equivalent tool exists in the ecosystem.

## Stack

| Crate | Usage |
|-------|-------|
| `kml` | parsing KML/KMZ (built-in zip support) |
| `ratatui` + `crossterm` | TUI framework + terminal backend |
| `clap` | CLI argument parsing |
| `serde` + `serde_json` | JSON output for non-interactive commands |
| `color-eyre` | error handling |

No external braille crate — `ratatui::widgets::canvas::Canvas` with `Marker::Braille` handles rendering natively.

## CLI Interface

```
kmlcli <file>                    # launches TUI (default)
kmlcli view <file>               # same, explicit
kmlcli info <file>               # dump document metadata (JSON stdout)
kmlcli list <file>               # list placemarks/folders (JSON)
kmlcli tree <file>               # textual structure tree
kmlcli export <file> -f geojson  # conversion (stretch goal)
```

- Accepts `.kml` and `.kmz` files
- Non-interactive commands output JSON by default
- `--format json|table` flag for non-interactive commands

## TUI Layout

```
┌─ Tree ───────────┬─ Map ────────────────┐
│ ▼ Document       │                      │
│   ▼ Folder 1     │   ⣿⡇    ⢀⣀         │
│     ● Placemark A│  ⣿⡇  ⣠⣾⣿⡇        │
│     ● Placemark B│  ⣿⣿⣶⣿⣿⡿⠁         │
│   ▶ Folder 2     │                      │
│     ─ LineString │        ● A           │
│     ◻ Polygon    │                      │
├───────────────────┴──────────────────────┤
│ Placemark A | 48.1173°N 1.6778°W        │
│ desc: "Point de départ" | style: #red   │
├──────────────────────────────────────────┤
│ [q]uit [/]search [tab]focus [+/-]zoom   │
└──────────────────────────────────────────┘
```

### Panels

| Panel | Content | Size |
|-------|---------|------|
| Tree (left) | Navigable folder/placemark/geometry tree | ~30% width |
| Map (right) | Braille geometry rendering, zoom/pan | ~70% width |
| Details (bottom) | Selected element info — coords, description, style | 2-3 lines |
| Status bar | Keybindings, filename, stats | 1 line |

### Keybindings

| Key | Action |
|-----|--------|
| `j/k` or arrows | Navigate tree |
| `Enter` | Expand/collapse folder, center map on element |
| `Tab` | Switch focus between Tree and Map |
| `+/-` or scroll | Zoom map |
| `h/j/k/l` (map focus) | Pan map |
| `/` | Search in tree |
| `q` | Quit |

### Map Behavior

- Tree selection highlights corresponding element on map (distinct color)
- Auto-fit zoom on content at load
- Non-selected elements visible but dimmed

## Architecture

```
main.rs
├── cli.rs          # clap — arg parsing, subcommand routing
├── parser.rs       # kml crate → internal model
├── model.rs        # KmlDocument, Feature, Geometry, Style
├── tui/
│   ├── app.rs      # app state, event loop
│   ├── tree.rs     # navigable tree widget
│   ├── map.rs      # braille canvas widget
│   ├── details.rs  # selected element info widget
│   └── input.rs    # keyboard handler
└── commands/
    ├── info.rs     # info subcommand (JSON stdout)
    ├── list.rs     # list subcommand
    └── tree.rs     # tree subcommand (text)
```

### Internal Model

```rust
struct KmlDocument {
    name: Option<String>,
    features: Vec<Feature>,
    styles: HashMap<String, Style>,
}

enum Feature {
    Folder { name: String, features: Vec<Feature> },
    Placemark {
        name: String,
        geometry: Geometry,
        style_id: Option<String>,
        description: Option<String>,
    },
}

enum Geometry {
    Point(Coord),
    LineString(Vec<Coord>),
    Polygon(Vec<Vec<Coord>>),  // outer ring + inner rings
    MultiGeometry(Vec<Geometry>),
}
```

### Data Flow

```
.kml/.kmz file
  → parser.rs (kml crate + zip for kmz)
  → KmlDocument (internal model)
  → TUI (app.rs takes ownership)
  → or non-interactive command (serialize to JSON via serde)
```

## Map Rendering

### Projection

Simplified Mercator — WGS84 lat/lon to 2D terminal coordinates.

```rust
fn project(coord: &Coord, bounds: &BoundingBox) -> (f64, f64) {
    // lon → x linear
    // lat → y via mercator (ln(tan(π/4 + lat/2)))
    // normalize to [0, canvas_width] x [0, canvas_height]
}
```

### Viewport

`center: Coord` + `zoom_level: f64`. Zoom multiplies/divides the visible bounding box. Pan shifts the center.

### Rendering by Geometry Type

| Type | Rendering |
|------|-----------|
| Point | Single dot, label if zoomed enough |
| LineString | Braille segments between each coord pair |
| Polygon | Outline segments, no fill (braille fill is unreadable) |
| MultiGeometry | Recursive |

### Colors

KML styles define colors in `aabbggrr` format. Mapped to nearest 256 terminal colors. Selected element highlighted (yellow/white bright), rest dimmed.

### Performance

Only geometries visible in viewport are rendered. Simple bounding box filter — no spatial indexing needed.

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Malformed KML | Clear error message with line/position if available, exit 1 |
| Corrupt KMZ | Same |
| Empty file / no geometry | TUI launches, empty tree, empty map, message in details panel |
| Single point | Center on it with default zoom |
| All points same location | Detect, cap zoom to avoid division by zero |
| Out-of-range coords | Render anyway, not our job to filter |
| Terminal too small | Degraded layout (hide tree). Below 40x10: "terminal too small" message |
| Resize | Ratatui handles reflow, recalculate map viewport |
| Non-interactive parse error | stderr + exit 1 |
| No file argument | Standard clap usage message |
