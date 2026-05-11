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

pub struct TileCache {
    cache: Arc<Mutex<LruCache<TileCoord, RenderedTile>>>,
    prefetch_tx: mpsc::Sender<Vec<TileCoord>>,
}

impl TileCache {
    pub fn new() -> Self {
        let cache = Arc::new(Mutex::new(LruCache::new(
            NonZeroUsize::new(CACHE_SIZE).unwrap(),
        )));

        let client = reqwest::blocking::Client::builder()
            .user_agent("kmlcli/0.1")
            .build()
            .expect("Failed to build HTTP client");

        let (prefetch_tx, prefetch_rx) = mpsc::channel::<Vec<TileCoord>>();

        // Single persistent worker thread
        {
            let cache = cache.clone();
            let client = client.clone();
            std::thread::spawn(move || {
                // Resolve tile URL template first
                let url_template = loop {
                    if let Some(url) = resolve_tile_url(&client) {
                        break url;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                };

                // Process prefetch requests
                while let Ok(coords) = prefetch_rx.recv() {
                    // Drain queued requests — only process the latest batch
                    let coords = {
                        let mut latest = coords;
                        while let Ok(newer) = prefetch_rx.try_recv() {
                            latest = newer;
                        }
                        latest
                    };

                    for coord in coords {
                        {
                            let cache_lock = cache.lock().unwrap();
                            if cache_lock.contains(&coord) {
                                continue;
                            }
                        }

                        let url = url_template
                            .replace("{z}", &coord.z.to_string())
                            .replace("{x}", &coord.x.to_string())
                            .replace("{y}", &coord.y.to_string());

                        if let Ok(response) = client.get(&url).send() {
                            if response.status().is_success() {
                                if let Ok(bytes) = response.bytes() {
                                    if bytes.is_empty() {
                                        let mut cache_lock = cache.lock().unwrap();
                                        cache_lock.put(
                                            coord,
                                            RenderedTile {
                                                segments: Vec::new(),
                                                labels: Vec::new(),
                                            },
                                        );
                                        continue;
                                    }
                                    let decompressed =
                                        decompress_gzip(&bytes).unwrap_or_else(|| bytes.to_vec());
                                    if let Ok(tile) = Tile::decode(decompressed.as_slice()) {
                                        let features = decode_tile(&tile, &coord);
                                        let rendered = super::render::prerender_tile(&features);
                                        let mut cache_lock = cache.lock().unwrap();
                                        cache_lock.put(coord, rendered);
                                    }
                                }
                            }
                        }
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
        cache.get(coord).map(|tile| f(tile))
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
