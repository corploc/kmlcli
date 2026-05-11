use std::io::Read;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use lru::LruCache;
use prost::Message;

use super::decode::{decode_tile, DecodedFeature};
use super::math::TileCoord;
use super::proto::Tile;

const TILE_URL_TEMPLATE: &str = "https://tiles.openfreemap.org/planet/{z}/{x}/{y}.pbf";
const CACHE_SIZE: usize = 64;

pub struct TileCache {
    cache: Arc<Mutex<LruCache<TileCoord, Vec<DecodedFeature>>>>,
    client: reqwest::blocking::Client,
}

impl TileCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(CACHE_SIZE).unwrap(),
            ))),
            client: reqwest::blocking::Client::builder()
                .user_agent("kmlcli/0.1")
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    pub fn get_cached(&self, coord: &TileCoord) -> Option<Vec<DecodedFeature>> {
        let mut cache = self.cache.lock().unwrap();
        cache.get(coord).cloned()
    }

    pub fn prefetch(&self, coords: Vec<TileCoord>) {
        let cache = self.cache.clone();
        let client = self.client.clone();

        std::thread::spawn(move || {
            for coord in coords {
                {
                    let cache_lock = cache.lock().unwrap();
                    if cache_lock.contains(&coord) {
                        continue;
                    }
                }

                let url = TILE_URL_TEMPLATE
                    .replace("{z}", &coord.z.to_string())
                    .replace("{x}", &coord.x.to_string())
                    .replace("{y}", &coord.y.to_string());

                if let Ok(response) = client.get(&url).send() {
                    if response.status().is_success() {
                        if let Ok(bytes) = response.bytes() {
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

fn decompress_gzip(data: &[u8]) -> Option<Vec<u8>> {
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf).ok()?;
    Some(buf)
}
