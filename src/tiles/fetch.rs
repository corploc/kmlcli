use std::io::Read;
use std::num::NonZeroUsize;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use lru::LruCache;
use prost::Message;

use super::decode::decode_tile;
use super::math::TileCoord;
use super::proto::Tile;
use super::render::RenderedTile;

const TILEJSON_URL: &str = "https://tiles.openfreemap.org/planet";
const CACHE_SIZE: usize = 64;
const WORKER_COUNT: usize = 4;

pub struct TileCache {
    cache: Arc<Mutex<LruCache<TileCoord, RenderedTile>>>,
    prefetch_tx: mpsc::Sender<Vec<TileCoord>>,
}

impl Default for TileCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TileCache {
    pub fn new() -> Self {
        let cache = Arc::new(Mutex::new(LruCache::new(
            NonZeroUsize::new(CACHE_SIZE).unwrap(),
        )));

        let client = reqwest::blocking::Client::builder()
            .user_agent("kmlcli/0.1")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        let (prefetch_tx, prefetch_rx) = mpsc::channel::<Vec<TileCoord>>();
        let prefetch_rx = Arc::new(Mutex::new(prefetch_rx));

        // Resolve tile URL template synchronously before spawning workers
        // (done in a dedicated thread to not block app startup)
        let url_template: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        {
            let template = url_template.clone();
            let c = client.clone();
            std::thread::spawn(move || {
                if let Some(url) = resolve_tile_url(&c) {
                    *template.lock().unwrap() = Some(url);
                }
            });
        }

        // Dispatch thread: receives batches, fans out individual tiles to workers
        let (tile_tx, tile_rx) = mpsc::channel::<TileCoord>();
        let tile_rx = Arc::new(Mutex::new(tile_rx));

        {
            let cache = cache.clone();
            let url_template = url_template.clone();
            std::thread::spawn(move || {
                let rx = prefetch_rx;
                loop {
                    let coords = {
                        let rx = rx.lock().unwrap();
                        match rx.recv() {
                            Ok(c) => c,
                            Err(_) => return,
                        }
                    };
                    // Drain to latest batch
                    let coords = {
                        let rx = rx.lock().unwrap();
                        let mut latest = coords;
                        while let Ok(newer) = rx.try_recv() {
                            latest = newer;
                        }
                        latest
                    };

                    // Wait for URL template (resolved by separate thread at startup)
                    let has_template = url_template.lock().unwrap().is_some();
                    if !has_template {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        continue;
                    }

                    for coord in coords {
                        let already_cached = {
                            let c = cache.lock().unwrap();
                            c.contains(&coord)
                        };
                        if !already_cached {
                            let _ = tile_tx.send(coord);
                        }
                    }
                }
            });
        }

        // Worker threads: fetch individual tiles in parallel
        for _ in 0..WORKER_COUNT {
            let tile_rx = tile_rx.clone();
            let cache = cache.clone();
            let client = client.clone();
            let url_template = url_template.clone();

            std::thread::spawn(move || {
                loop {
                    let coord = {
                        let rx = tile_rx.lock().unwrap();
                        match rx.recv() {
                            Ok(c) => c,
                            Err(_) => return,
                        }
                    };

                    // Skip if already cached (another worker might have fetched it)
                    {
                        let c = cache.lock().unwrap();
                        if c.contains(&coord) {
                            continue;
                        }
                    }

                    let tmpl = {
                        let t = url_template.lock().unwrap();
                        match t.as_ref() {
                            Some(u) => u.clone(),
                            None => continue,
                        }
                    };

                    let url = tmpl
                        .replace("{z}", &coord.z.to_string())
                        .replace("{x}", &coord.x.to_string())
                        .replace("{y}", &coord.y.to_string());

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
                }
            });
        }

        Self { cache, prefetch_tx }
    }

    /// Access pre-rendered tile data. No cloning — caller borrows via closure.
    pub fn with_cached<F, R>(&self, coord: &TileCoord, f: F) -> Option<R>
    where
        F: FnOnce(&RenderedTile) -> R,
    {
        let mut cache = self.cache.lock().unwrap();
        cache.get(coord).map(f)
    }

    pub fn prefetch(&self, coords: Vec<TileCoord>) {
        let _ = self.prefetch_tx.send(coords);
    }
}

fn resolve_tile_url(client: &reqwest::blocking::Client) -> Option<String> {
    let resp = client.get(TILEJSON_URL).send().ok()?;
    let body = resp.text().ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    let url = json.get("tiles")?.as_array()?.first()?.as_str()?;
    Some(url.to_string())
}

fn decompress_gzip(data: &[u8]) -> Option<Vec<u8>> {
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf).ok()?;
    Some(buf)
}
