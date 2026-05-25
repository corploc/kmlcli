# Review Fixes Post-v0.2.0 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Corriger les bugs et fragilités identifiés dans la review post-release 0.2.0 (debug log laissé en prod, busy-spin CPU, panics non-récupérés, robustesse parser/projection).

**Architecture:** Fixes ciblés, chacun isolable et testable indépendamment. Branche `fix/review-v0.2.0` (déjà créée). Un commit par tâche.

**Tech Stack:** Rust 2024, ratatui 0.30, reqwest 0.12 (blocking + rustls), kml 0.13, lru 0.16, color-eyre 0.6.

**Pré-requis vérification :** Avant de toucher au code, exécuter `cargo test` et noter la baseline (tout doit passer). À la fin du plan, `cargo test && cargo clippy -- -D warnings && cargo fmt --check` doivent passer.

**Note sur les faux positifs de la review :**
- `clip.rs` div-by-zero : impossible par invariant Cohen-Sutherland (si TOP/BOTTOM est dans `code_out`, les endpoints straddlent la frontière → `dy ≠ 0`). Tâche 10 ajoute un test de régression pour ancrer l'invariant.
- `ll2tile` pôles : `tan(±π/2)` en f64 ne diverge pas (PI/2 inexact), donc pas de panic. Coords absurdes possibles, traité en tâche 9 (P2).

---

## File Map

| Fichier | Responsabilité | Tâches |
|---------|----------------|--------|
| `src/tiles/fetch.rs` | Debug log, busy-spin, UA version, expect, cap prefetch | 1, 2, 3, 4, 5 |
| `src/tui/app.rs` | Debug log (event loop), tree scroll dynamique, panic hook order | 1, 8, 11 |
| `src/tui/map.rs` | Debug log (collect), cap visible_tiles | 1, 4 |
| `src/main.rs` | Panic hook avant `App::new` | 11 |
| `src/projection.rs` | Clamp lat sur pan_up/pan_down | 6 |
| `src/parser.rs` | Résolution `Kml::StyleMap` | 7 |
| `src/tiles/math.rs` | Clamp lat dans `ll2tile` | 9 |
| `src/clip.rs` | (lecture seule) | 10 (test) |
| `tests/clip_test.rs` | Régression Cohen-Sutherland axis-aligned | 10 |
| `tests/projection_test.rs` | Régression pan clamp | 6 |
| `tests/parser_test.rs` | Régression StyleMap | 7 |
| `tests/tile_math_test.rs` | Régression ll2tile pôles | 9 |

---

## Task 1: Supprimer le debug perf log `/tmp/kmlcli_perf.log` (ship-blocker)

**Files:**
- Modify: `src/tiles/fetch.rs:141-185`
- Modify: `src/tui/app.rs:128-153`
- Modify: `src/tui/map.rs:113-136`

Le log écrit sur disque à chaque frame ET chaque tile fetch, pollue `/tmp`, échoue silencieusement sur FS read-only. Aucune valeur en prod.

- [ ] **Step 1: Retirer le block perf log de `fetch.rs`**

Dans `src/tiles/fetch.rs`, supprimer lignes 141, 146-147, 164-185 (les `fetch_start`/`fetch_ms`, `decode_start`/`decode_ms`, et le block `OpenOptions::new()...open("/tmp/kmlcli_perf.log")`). Remplacer le corps du `if let Ok(response) = ...` par directement la logique de décompression/décodage sans timers ni log :

```rust
if let Ok(response) = client.get(&url).send()
    && response.status().is_success()
    && let Ok(bytes) = response.bytes()
{
    let rendered = if bytes.is_empty() {
        RenderedTile {
            segments: Vec::new(),
            labels: Vec::new(),
        }
    } else {
        let decompressed =
            decompress_gzip(&bytes).unwrap_or_else(|| bytes.to_vec());
        match Tile::decode(decompressed.as_slice()) {
            Ok(tile) => {
                let features = decode_tile(&tile, &coord);
                super::render::prerender_tile(&features)
            }
            Err(_) => continue,
        }
    };
    let mut cache_lock = cache.lock().unwrap();
    cache_lock.put(coord, rendered);
}
```

- [ ] **Step 2: Retirer le perf log de `tui/app.rs`**

Dans `src/tui/app.rs`, supprimer dans `event_loop` (lignes ~128-153) : la déclaration `use std::io::Write;`, `let mut perf_log = ...`, `let mut frame_count`, et tout le block `if frame_count.is_multiple_of(30) && ...`. Garder la mesure `frame_start`/`draw_elapsed` si non utilisée ailleurs → supprimer aussi.

Résultat attendu de `event_loop` après nettoyage :

```rust
fn event_loop(
    &mut self,
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>,
    signal_quit: &Arc<AtomicBool>,
) -> Result<()> {
    loop {
        terminal.draw(|f| self.draw(f))?;

        if signal_quit.load(Ordering::Relaxed) || self.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => {
                    let action = handle_key(key);
                    self.handle_action(action);
                }
                Event::Mouse(mouse) => {
                    let action = handle_mouse(mouse);
                    self.handle_action(action);
                }
                Event::FocusGained => {
                    let _ = enable_raw_mode();
                }
                _ => {}
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 3: Retirer le perf log de `tui/map.rs`**

Dans `src/tui/map.rs`, supprimer le block `// Perf log` (lignes ~113-136) entièrement, ainsi que les variables `t0`, `t_tiles`, `t_dedup`, `t_kml`, `tiles_cached`, `tiles_visible`, `zoom` si elles ne servent qu'au log. Vérifier avec grep avant suppression :

```bash
grep -n "t_tiles\|t_dedup\|t_kml\|tiles_cached\|tiles_visible" src/tui/map.rs
```

Si une variable est utilisée ailleurs, la garder. Sinon, supprimer.

- [ ] **Step 4: Vérifier la compile et les tests**

```bash
cargo build && cargo test
```

Expected: `Finished` + tous les tests passent.

- [ ] **Step 5: Vérifier qu'il ne reste plus aucune référence au log**

```bash
grep -rn "kmlcli_perf\|/tmp/kmlcli" src/
```

Expected: aucun résultat.

- [ ] **Step 6: Commit**

```bash
git add src/tiles/fetch.rs src/tui/app.rs src/tui/map.rs
git commit -m "fix: remove debug perf log left from development

Was writing to /tmp/kmlcli_perf.log on every frame and every tile fetch.
Pollutes /tmp, fails silently on read-only filesystems, no value in prod."
```

---

## Task 2: Fixer le busy-spin dans le thread dispatch tuiles

**Files:**
- Modify: `src/tiles/fetch.rs:84-88`

Quand le template d'URL n'est pas encore résolu, le thread fait `continue` sans sleep, brûlant 100% CPU jusqu'à résolution (réseau lent = secondes).

- [ ] **Step 1: Remplacer le `continue` par un sleep court**

Dans `src/tiles/fetch.rs`, dans le thread dispatch, remplacer :

```rust
// Wait for URL template
let has_template = url_template.lock().unwrap().is_some();
if !has_template {
    continue;
}
```

par :

```rust
// Wait for URL template (resolved by separate thread at startup)
let has_template = url_template.lock().unwrap().is_some();
if !has_template {
    std::thread::sleep(std::time::Duration::from_millis(50));
    continue;
}
```

- [ ] **Step 2: Vérifier compile**

```bash
cargo build
```

Expected: `Finished`.

- [ ] **Step 3: Commit**

```bash
git add src/tiles/fetch.rs
git commit -m "fix(tiles): sleep 50ms instead of busy-spinning on missing URL template

Dispatch thread was burning 100% CPU until resolve_tile_url completed."
```

---

## Task 3: User-Agent figé à `kmlcli/0.1`

**Files:**
- Modify: `src/tiles/fetch.rs:36`

- [ ] **Step 1: Remplacer la chaîne hardcodée**

Dans `src/tiles/fetch.rs`, remplacer :

```rust
.user_agent("kmlcli/0.1")
```

par :

```rust
.user_agent(concat!("kmlcli/", env!("CARGO_PKG_VERSION")))
```

- [ ] **Step 2: Vérifier compile**

```bash
cargo build
```

Expected: `Finished`.

- [ ] **Step 3: Commit**

```bash
git add src/tiles/fetch.rs
git commit -m "fix(tiles): use CARGO_PKG_VERSION in HTTP user-agent

Was hardcoded to kmlcli/0.1 even though crate is at 0.2.0."
```

---

## Task 4: Cap des tuiles visibles dans le prefetch (pas seulement au render)

**Files:**
- Modify: `src/tui/map.rs` (chercher le call à `prefetch`)
- Modify: `src/tui/app.rs:289-296` (callsite `visible_tiles`)

Le render cap à 16 via `.take(16)` dans `map.rs:55`, mais le prefetch envoie la liste entière au dispatch thread. À z5 sur une vue large, ça peut faire 1024 requêtes HTTP empilées.

- [ ] **Step 1: Localiser les appels à `visible_tiles`**

```bash
grep -rn "visible_tiles\|prefetch" src/
```

Identifier où la liste est construite et passée à `TileCache::prefetch`. Probablement `src/tui/app.rs` autour de la ligne 289.

- [ ] **Step 2: Définir une constante de cap partagée**

Dans `src/tiles/math.rs`, ajouter en haut du fichier :

```rust
/// Maximum number of tiles to fetch/render per frame.
/// Beyond this, the map degrades to partial rendering — better than blocking on HTTP fanout.
pub const MAX_VISIBLE_TILES: usize = 16;
```

- [ ] **Step 3: Appliquer le cap dans `app.rs` côté prefetch**

Au callsite identifié à l'étape 1, ajouter `.take(MAX_VISIBLE_TILES)` sur l'itérateur avant `.collect::<Vec<_>>()` ou ajouter `.truncate(MAX_VISIBLE_TILES)` sur la Vec construite. Importer `crate::tiles::math::MAX_VISIBLE_TILES`.

Exemple :

```rust
use crate::tiles::math::{visible_tiles, MAX_VISIBLE_TILES};

let coords: Vec<_> = visible_tiles(...).take(MAX_VISIBLE_TILES).collect();
self.tile_cache.prefetch(coords);
```

- [ ] **Step 4: Remplacer le `16` magique dans `map.rs:55` par la constante**

Dans `src/tui/map.rs:55`, remplacer le `.take(16)` par `.take(MAX_VISIBLE_TILES)`. Ajouter l'import.

- [ ] **Step 5: Vérifier compile + tests**

```bash
cargo build && cargo test
```

Expected: `Finished` + tests passent.

- [ ] **Step 6: Commit**

```bash
git add src/tiles/math.rs src/tui/app.rs src/tui/map.rs
git commit -m "fix(tiles): cap prefetch list at MAX_VISIBLE_TILES (16)

Previously the render layer capped at 16 but prefetch sent the full
visible_tiles list (potentially 1000+ at low zoom), fanning out
unbounded HTTP requests to workers."
```

---

## Task 5: `TileCache::new()` retourne `Result` (plus de `expect`)

**Files:**
- Modify: `src/tiles/fetch.rs:29-194`
- Modify: `src/tui/app.rs` (callsite `TileCache::new()`)

`reqwest::ClientBuilder::build().expect(...)` peut panic en cas d'échec TLS — panic dans `App::new` = terminal cassé sans cleanup.

- [ ] **Step 1: Changer la signature et propager**

Dans `src/tiles/fetch.rs`, transformer `pub fn new() -> Self` en :

```rust
pub fn new() -> color_eyre::Result<Self> {
    // ...
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("kmlcli/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    // ... reste du corps inchangé ...
    Ok(Self { cache, prefetch_tx })
}
```

Supprimer `impl Default for TileCache` (ne peut plus être `Default` avec `Result`).

- [ ] **Step 2: Mettre à jour le callsite**

Dans `src/tui/app.rs`, là où `TileCache::new()` est appelé (probablement dans `App::new`), propager le `?`. Si `App::new` ne retourne pas déjà `Result`, le faire :

```bash
grep -n "TileCache::new\|impl App" src/tui/app.rs
```

Adapter `App::new` pour retourner `color_eyre::Result<Self>` et mettre à jour `main.rs` si nécessaire.

- [ ] **Step 3: Vérifier compile + tests**

```bash
cargo build && cargo test
```

Expected: `Finished` + tests passent.

- [ ] **Step 4: Commit**

```bash
git add src/tiles/fetch.rs src/tui/app.rs src/main.rs
git commit -m "fix(tiles): propagate HTTP client build errors instead of panicking

reqwest::ClientBuilder::build() can fail on TLS misconfiguration.
A panic in App::new() bypassed terminal cleanup, leaving the TTY broken."
```

---

## Task 6: Clamper la latitude dans les opérations de pan

**Files:**
- Modify: `src/projection.rs:70-75`
- Test: `tests/projection_test.rs`

`pan_up`/`pan_down` modifient `center_lat` sans borne. Au-delà de ±85.05, le projection Mercator part en cacahuète.

- [ ] **Step 1: Écrire le test de régression**

Ajouter dans `tests/projection_test.rs` :

```rust
#[test]
fn pan_up_clamps_to_mercator_max() {
    let bbox = BoundingBox {
        min_lon: 0.0,
        max_lon: 1.0,
        min_lat: 84.0,
        max_lat: 85.0,
    };
    let mut vp = Viewport::from_bbox(&bbox);
    for _ in 0..1000 {
        vp.pan_up();
    }
    assert!(vp.center_lat <= 85.05, "center_lat = {} exceeded 85.05", vp.center_lat);
}

#[test]
fn pan_down_clamps_to_mercator_min() {
    let bbox = BoundingBox {
        min_lon: 0.0,
        max_lon: 1.0,
        min_lat: -85.0,
        max_lat: -84.0,
    };
    let mut vp = Viewport::from_bbox(&bbox);
    for _ in 0..1000 {
        vp.pan_down();
    }
    assert!(vp.center_lat >= -85.05, "center_lat = {} below -85.05", vp.center_lat);
}
```

Vérifier que `BoundingBox` et `Viewport` sont déjà importés en haut du fichier ; sinon les ajouter.

- [ ] **Step 2: Lancer les tests, ils doivent échouer**

```bash
cargo test --test projection_test pan_up_clamps pan_down_clamps
```

Expected: FAIL (center_lat dépasse les bornes).

- [ ] **Step 3: Ajouter le clamp dans `projection.rs`**

Dans `src/projection.rs`, remplacer `pan_up` et `pan_down` :

```rust
pub fn pan_up(&mut self) {
    self.center_lat = (self.center_lat + self.half_lat * PAN_FACTOR).min(85.05);
}
pub fn pan_down(&mut self) {
    self.center_lat = (self.center_lat - self.half_lat * PAN_FACTOR).max(-85.05);
}
```

- [ ] **Step 4: Relancer les tests**

```bash
cargo test --test projection_test
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/projection.rs tests/projection_test.rs
git commit -m "fix(projection): clamp center_lat to ±85.05 in pan_up/pan_down

Without clamp, repeated pan operations could push center_lat past the
Mercator projection limit, breaking tile coordinate math."
```

---

## Task 7: Résolution des `Kml::StyleMap`

**Files:**
- Modify: `src/parser.rs:44-94`
- Test: `tests/parser_test.rs`
- Test fixture: `src/fixtures/` (vérifier ce qui existe, ajouter si besoin)

KML produit par Google Maps utilise massivement `StyleMap` (mapping `normal`/`highlight` → `#styleId`). Actuellement le `_ => {}` fallback les drop silencieusement.

- [ ] **Step 1: Inspecter la structure existante**

```bash
ls src/fixtures/ tests/fixtures/ 2>/dev/null
grep -n "Kml::Style\b\|Kml::StyleMap" src/parser.rs
```

Comprendre où les styles sont actuellement accumulés (probablement une `HashMap<String, Style>`).

- [ ] **Step 2: Écrire le test de régression**

Créer une fixture KML minimale avec StyleMap. Dans `tests/fixtures/style_map.kml` :

```xml
<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
<Document>
  <Style id="redStyle">
    <LineStyle><color>ff0000ff</color><width>2</width></LineStyle>
  </Style>
  <StyleMap id="redMap">
    <Pair><key>normal</key><styleUrl>#redStyle</styleUrl></Pair>
    <Pair><key>highlight</key><styleUrl>#redStyle</styleUrl></Pair>
  </StyleMap>
  <Placemark>
    <name>p1</name>
    <styleUrl>#redMap</styleUrl>
    <LineString><coordinates>0,0 1,1</coordinates></LineString>
  </Placemark>
</Document>
</kml>
```

Dans `tests/parser_test.rs`, ajouter :

```rust
#[test]
fn style_map_resolves_to_normal_style() {
    let xml = std::fs::read_to_string("tests/fixtures/style_map.kml").unwrap();
    let doc = kmlcli::parser::parse_kml_str(&xml).expect("parse ok");
    let placemark = doc.find_placemark_by_name("p1").expect("placemark found");
    let style = placemark.resolved_style().expect("style resolved");
    assert_eq!(style.line_color, Some("ff0000ff".to_string()));
}
```

Adapter les noms d'API (`parse_kml_str`, `find_placemark_by_name`, `resolved_style`, `line_color`) à ceux réellement exposés par `src/lib.rs` et `src/model.rs`. Vérifier avant :

```bash
grep -n "pub fn\|pub struct" src/parser.rs src/model.rs src/lib.rs
```

Si une API helper n'existe pas, faire un test plus brut qui parcourt directement la structure retournée.

- [ ] **Step 3: Lancer le test, doit échouer**

```bash
cargo test --test parser_test style_map_resolves
```

Expected: FAIL (style non résolu, soit `None`, soit assertion sur structure vide).

- [ ] **Step 4: Implémenter la résolution `StyleMap`**

Dans `src/parser.rs`, dans la fonction qui itère sur les éléments KML (probablement `convert_kml` autour de la ligne 44), ajouter une branche pour `Kml::StyleMap`. Approche : maintenir un `HashMap<String, String>` qui mappe `styleMapId` → `normalStyleId`, puis quand on résout un `styleUrl` sur un placemark, suivre la chaîne `StyleMap → Style`.

Pseudocode (adapter aux types exacts de la crate `kml`) :

```rust
// Nouveau champ dans le builder/accumulateur:
// style_maps: HashMap<String, String>  // mapId -> normalStyleId

Kml::StyleMap(sm) => {
    if let Some(id) = sm.id {
        // Chercher la Pair avec key == "normal"
        let normal_url = sm.pairs.iter()
            .find(|p| p.key == "normal")
            .map(|p| p.style_url.trim_start_matches('#').to_string());
        if let Some(target) = normal_url {
            style_maps.insert(id, target);
        }
    }
}
```

Puis à la résolution d'un `styleUrl` sur Placemark : si l'ID match un `style_map`, suivre vers le vrai style. Implémenter en consultant la signature réelle de `kml::types::StyleMap` :

```bash
cargo doc --open  # ou grep dans ~/.cargo/registry/src/*/kml-0.13*/
```

- [ ] **Step 5: Relancer le test**

```bash
cargo test --test parser_test
```

Expected: PASS.

- [ ] **Step 6: Vérifier qu'aucun autre test ne casse**

```bash
cargo test
```

Expected: tous PASS.

- [ ] **Step 7: Commit**

```bash
git add src/parser.rs tests/parser_test.rs tests/fixtures/style_map.kml
git commit -m "feat(parser): resolve Kml::StyleMap to its normal style

Google Maps KML exports use StyleMap (mapping normal/highlight to
style IDs) ubiquitously. The previous parser dropped StyleMap entries
in the catch-all match arm, leaving placemarks unstyled."
```

---

## Task 8: Tree scroll dynamique (vs hardcodé à 20)

**Files:**
- Modify: `src/tui/app.rs:179-200` (handle_action / tree_scroll logic)

`app.rs:190-191` compare `new_pos >= self.tree_scroll + 20` alors que la hauteur réelle du panneau est calculée dynamiquement au render (ligne ~360).

- [ ] **Step 1: Localiser le calcul de hauteur réelle**

```bash
grep -n "tree_scroll\|saturating_sub(4)" src/tui/app.rs
```

Identifier la formule de hauteur (probablement `area.height.saturating_sub(4)` dans `draw`).

- [ ] **Step 2: Ajouter un champ `tree_visible_height: u16` dans `App`**

Dans la définition de `App` :

```rust
pub struct App {
    // ... champs existants ...
    tree_visible_height: u16,
}
```

Initialiser à `20` dans `App::new()` (fallback avant premier render).

- [ ] **Step 3: Mettre à jour le champ au render**

Dans `draw` (autour de la ligne 360), après avoir calculé la hauteur du panneau arbre :

```rust
let tree_height = area.height.saturating_sub(4).max(4);
self.tree_visible_height = tree_height;
```

(Adapter au nom exact de la variable locale.)

- [ ] **Step 4: Utiliser le champ dans `handle_action`**

Remplacer le `20` magique dans la logique de scroll :

```rust
// Avant:
if new_pos >= self.tree_scroll + 20 {
    self.tree_scroll = new_pos - 19;
}
// Après:
let visible = self.tree_visible_height as usize;
if new_pos >= self.tree_scroll + visible {
    self.tree_scroll = new_pos.saturating_sub(visible - 1);
}
```

- [ ] **Step 5: Vérifier compile + tests**

```bash
cargo build && cargo test
```

Expected: `Finished` + tests passent.

- [ ] **Step 6: Commit**

```bash
git add src/tui/app.rs
git commit -m "fix(tui): use dynamic tree panel height for scroll math

Hardcoded 20 lines did not match the panel height computed at render,
causing selection to scroll out of view on short terminals."
```

---

## Task 9: Clamp lat dans `ll2tile` (robustesse pôles)

**Files:**
- Modify: `src/tiles/math.rs`
- Test: `tests/tile_math_test.rs`

`ll2tile` aux pôles produit des coords absurdes (pas de panic, mais wrap-around en u32).

- [ ] **Step 1: Écrire le test de régression**

Dans `tests/tile_math_test.rs`, ajouter :

```rust
#[test]
fn ll2tile_handles_north_pole() {
    let t = kmlcli::tiles::math::ll2tile(89.99, 0.0, 5);
    assert!(t.y < 32, "y={} out of range for z=5", t.y);
}

#[test]
fn ll2tile_handles_south_pole() {
    let t = kmlcli::tiles::math::ll2tile(-89.99, 0.0, 5);
    assert!(t.y < 32, "y={} out of range for z=5", t.y);
}
```

(z=5 → 2^5 = 32 tuiles max par axe.)

- [ ] **Step 2: Lancer les tests, ils doivent échouer**

```bash
cargo test --test tile_math_test ll2tile_handles
```

Expected: FAIL (y >= 32).

- [ ] **Step 3: Clamper lat dans `ll2tile`**

Dans `src/tiles/math.rs`, en début de `ll2tile` :

```rust
pub fn ll2tile(lat: f64, lon: f64, z: u32) -> TileCoord {
    let lat = lat.clamp(-85.05112878, 85.05112878);
    // ... reste inchangé ...
}
```

- [ ] **Step 4: Relancer les tests**

```bash
cargo test --test tile_math_test
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/tiles/math.rs tests/tile_math_test.rs
git commit -m "fix(tiles): clamp latitude in ll2tile to Mercator valid range

Beyond ±85.05° the projection produces y values outside the tile grid,
which silently wrap when cast to u32. Clamp at the function boundary."
```

---

## Task 10: Test de régression Cohen-Sutherland (axis-aligned)

**Files:**
- Test: `tests/clip_test.rs`

Pas de bug réel — l'algo est correct. Test pour ancrer l'invariant et empêcher une régression future si quelqu'un refactor.

- [ ] **Step 1: Ajouter les tests**

Dans `tests/clip_test.rs`, ajouter :

```rust
#[test]
fn clip_horizontal_segment_crossing_box() {
    // Horizontal segment crossing the box left-to-right at y=0.5
    let result = kmlcli::clip::clip_line(-1.0, 0.5, 2.0, 0.5, &[0.0, 1.0], &[0.0, 1.0]);
    assert_eq!(result, Some((0.0, 0.5, 1.0, 0.5)));
}

#[test]
fn clip_vertical_segment_crossing_box() {
    let result = kmlcli::clip::clip_line(0.5, -1.0, 0.5, 2.0, &[0.0, 1.0], &[0.0, 1.0]);
    assert_eq!(result, Some((0.5, 0.0, 0.5, 1.0)));
}

#[test]
fn clip_horizontal_segment_above_box() {
    // Both endpoints above — must return None, no division
    let result = kmlcli::clip::clip_line(-1.0, 2.0, 2.0, 2.0, &[0.0, 1.0], &[0.0, 1.0]);
    assert_eq!(result, None);
}

#[test]
fn clip_vertical_segment_left_of_box() {
    let result = kmlcli::clip::clip_line(-1.0, -1.0, -1.0, 2.0, &[0.0, 1.0], &[0.0, 1.0]);
    assert_eq!(result, None);
}
```

Adapter le path d'import à ce qui est exposé (`crate::clip::clip_line` ou `kmlcli::clip::clip_line`). Vérifier :

```bash
grep -n "pub use\|pub mod clip" src/lib.rs
```

- [ ] **Step 2: Lancer les tests, ils doivent passer immédiatement**

```bash
cargo test --test clip_test
```

Expected: PASS (le code est déjà correct).

- [ ] **Step 3: Commit**

```bash
git add tests/clip_test.rs
git commit -m "test(clip): pin Cohen-Sutherland behavior on axis-aligned segments

Locks in the invariant that segments parallel to a clipping edge
return either None (both outside) or the intersected portion (crossing),
without triggering division by zero."
```

---

## Task 11: Poser le panic hook avant `App::new`

**Files:**
- Modify: `src/main.rs`
- Modify: `src/tui/app.rs` (déplacer le hook hors de `run`)

Si un worker tile panic pendant `App::new`, le hook n'est pas encore installé → terminal cassé.

- [ ] **Step 1: Identifier où le panic hook est posé actuellement**

```bash
grep -n "panic::set_hook\|set_hook" src/
```

Probablement dans `App::run` ou un helper `init_terminal`.

- [ ] **Step 2: Extraire la logique d'init terminal + panic hook dans une fonction publique**

Dans `src/tui/app.rs`, créer (ou déplacer) :

```rust
/// Initialise le terminal (raw mode, alternate screen, mouse capture)
/// et installe un panic hook qui restore le terminal avant de re-panic.
pub fn init_terminal() -> color_eyre::Result<ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>> {
    use crossterm::{execute, terminal::{enable_raw_mode, EnterAlternateScreen}, event::EnableMouseCapture};
    let mut stderr = std::io::stderr();
    enable_raw_mode()?;
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        hook(info);
    }));

    Ok(ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stderr))?)
}

pub fn restore_terminal() -> color_eyre::Result<()> {
    use crossterm::{execute, terminal::{disable_raw_mode, LeaveAlternateScreen}, event::DisableMouseCapture};
    disable_raw_mode()?;
    execute!(std::io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
```

Adapter aux imports et au code existant — ne pas dupliquer la logique si elle existe déjà ailleurs.

- [ ] **Step 3: Appeler `init_terminal` avant `App::new` dans `main.rs`**

```rust
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    // ... dispatch commandes non-TUI ...

    let mut terminal = kmlcli::tui::app::init_terminal()?;
    let result = (|| -> color_eyre::Result<()> {
        let mut app = App::new(/* args */)?;
        app.run(&mut terminal)?;
        Ok(())
    })();
    kmlcli::tui::app::restore_terminal()?;
    result
}
```

Adapter au shape réel de `main.rs` actuel (lire d'abord).

- [ ] **Step 4: Vérifier compile + tests**

```bash
cargo build && cargo test
```

Expected: `Finished` + tests passent.

- [ ] **Step 5: Test manuel (non scripté)**

```bash
cargo run -- <fichier-kml-existant.kml>
```

Vérifier que le TUI s'ouvre et se ferme proprement (Ctrl+C, q). Le terminal doit être propre après quit.

- [ ] **Step 6: Commit**

```bash
git add src/main.rs src/tui/app.rs
git commit -m "fix(tui): install panic hook before constructing App

Tile worker threads are spawned inside App::new(). A panic before run()
left the terminal in raw mode without cleanup. Move terminal init and
the panic hook to main, before any thread spawning."
```

---

## Task 12: Verification finale & PR

- [ ] **Step 1: Run full quality gate**

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

Expected: tout passe.

- [ ] **Step 2: Vérifier la propreté de l'historique**

```bash
git log --oneline main..HEAD
```

Expected: ~11 commits, un par tâche, messages clairs.

- [ ] **Step 3: Push la branche**

```bash
git push -u origin fix/review-v0.2.0
```

- [ ] **Step 4: Ouvrir la PR**

```bash
gh pr create --title "fix: post-v0.2.0 review — debug log, busy-spin, panic handling, parser robustness" --body "$(cat <<'EOF'
## Summary
Corrige les problèmes identifiés dans la review post-release 0.2.0.

### Ship-blockers
- Supprime le log debug `/tmp/kmlcli_perf.log` écrit à chaque frame + chaque tile fetch
- Fix busy-spin 100% CPU dans le thread dispatch tuiles quand l'URL n'est pas encore résolue

### Robustesse
- `TileCache::new()` retourne `Result` au lieu de panic (terminal restore safe)
- Panic hook posé avant `App::new` (workers tuiles spawn dans `new`)
- Cap prefetch à 16 tuiles (évite fanout HTTP unbounded à low zoom)
- Clamp `center_lat` dans pan_up/pan_down (Mercator ±85.05)
- Clamp lat dans `ll2tile` (coords invalides aux pôles)

### Features parser
- Résolution `Kml::StyleMap` → style normal (Google Maps exports)

### Quality
- User-Agent dynamique (`CARGO_PKG_VERSION`)
- Tree scroll utilise la hauteur réelle du panneau
- Test de régression Cohen-Sutherland axis-aligned

## Test plan
- [ ] `cargo test` — tous tests passent (nouveaux : pan clamp, ll2tile pôles, StyleMap, axis-aligned clip)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Smoke test manuel : ouvrir un .kml Google Maps avec StyleMaps, vérifier styles appliqués
- [ ] Smoke test manuel : `ls /tmp/kmlcli_*` doit rester vide après usage
- [ ] Smoke test manuel : Ctrl+C en cours d'usage → terminal propre
EOF
)"
```

---

## Self-Review

**Spec coverage:**
- P0 debug log → Tâche 1
- P0 busy-spin → Tâche 2
- P0 expect on client build → Tâche 5
- P0 clip div-by-zero → Faux positif, Tâche 10 (test régression seulement)
- P1 StyleMap → Tâche 7
- P1 cap prefetch → Tâche 4
- P1 panic hook order → Tâche 11
- P1 pan unbounded → Tâche 6
- P2 UA version → Tâche 3
- P2 tree scroll → Tâche 8
- P2 ll2tile pôles → Tâche 9
- P2 zoom_level aspect ratio → **Pas couvert** : nécessite refactor de `Viewport` (besoin de connaître la taille canvas). Reporté à un futur plan.

**Placeholder scan:** aucun "TBD" / "handle edge cases" / etc. Les "adapter à l'API exacte" en tâches 7 et 11 sont assortis de commandes `grep` concrètes pour découvrir l'API — légitime quand on ne peut pas pré-deviner les noms exacts.

**Type consistency:** `MAX_VISIBLE_TILES` cohérent entre `src/tiles/math.rs` (définition tâche 4) et les utilisations dans `map.rs`/`app.rs`. `tree_visible_height: u16` cohérent dans tâche 8. `init_terminal`/`restore_terminal` cohérent entre `main.rs` et `tui/app.rs` en tâche 11.
