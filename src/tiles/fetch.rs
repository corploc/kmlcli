use std::io::Read;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use lru::LruCache;
use prost::Message;

use super::decode::{decode_tile, DecodedFeature};
use super::math::TileCoord;
use super::proto::Tile;

const TILEJSON_URL: &str = "https://tiles.openfreemap.org/planet";
const CACHE_SIZE: usize = 64;

pub struct TileCache {
    cache: Arc<Mutex<LruCache<TileCoord, Vec<DecodedFeature>>>>,
    client: reqwest::blocking::Client,
    tile_url_template: Arc<Mutex<Option<String>>>,
}

impl TileCache {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .user_agent("kmlcli/0.1")
            .build()
            .expect("Failed to build HTTP client");

        let tile_url_template = Arc::new(Mutex::new(None));

        // Resolve tile URL template from TileJSON in background
        {
            let template = tile_url_template.clone();
            let c = client.clone();
            std::thread::spawn(move || {
                if let Some(url) = resolve_tile_url(&c) {
                    *template.lock().unwrap() = Some(url);
                }
            });
        }

        Self {
            cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(CACHE_SIZE).unwrap(),
            ))),
            client,
            tile_url_template,
        }
    }

    pub fn get_cached(&self, coord: &TileCoord) -> Option<Vec<DecodedFeature>> {
        let mut cache = self.cache.lock().unwrap();
        cache.get(coord).cloned()
    }

    pub fn prefetch(&self, coords: Vec<TileCoord>) {
        let cache = self.cache.clone();
        let client = self.client.clone();
        let template = self.tile_url_template.clone();

        std::thread::spawn(move || {
            // Wait for template to be resolved
            let url_template = {
                let t = template.lock().unwrap();
                match t.as_ref() {
                    Some(url) => url.clone(),
                    None => return, // Not resolved yet, skip this batch
                }
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
                                // Empty tile (ocean/void) — cache empty result
                                let mut cache_lock = cache.lock().unwrap();
                                cache_lock.put(coord, Vec::new());
                                continue;
                            }
                            let decompressed =
                                decompress_gzip(&bytes).unwrap_or_else(|| bytes.to_vec());
                            if let Ok(tile) = Tile::decode(decompressed.as_slice()) {
                                let features = decode_tile(&tile, &coord);
                                let mut cache_lock = cache.lock().unwrap();
                                cache_lock.put(coord, features);
                            }
                        }
                    }
                }
            }
        });
    }
}

/// Fetch TileJSON and extract the tile URL template.
fn resolve_tile_url(client: &reqwest::blocking::Client) -> Option<String> {
    let resp = client.get(TILEJSON_URL).send().ok()?;
    let body = resp.text().ok()?;
    // Parse TileJSON to extract tiles[0]
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
