use rustc_hash::FxHashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// How often to refresh caches (in seconds)
pub const CACHE_REFRESH_INTERVAL_SECS: u64 = 30 * 60; // 30 minutes

/// Buffer values for special markets
const ATP_BUFFER: f64 = 0.01;
const LIGUE1_BUFFER: f64 = 0.01;

// ============================================================================
// Cache Data Structures
// ============================================================================

/// Holds all cached market data
pub struct MarketCaches {
    /// Token ID -> neg_risk boolean
    pub neg_risk: RwLock<FxHashMap<String, bool>>,
    /// Token ID -> market slug
    pub slugs: RwLock<FxHashMap<String, String>>,
    /// ATP token IDs (for buffer calculation)
    pub atp_tokens: RwLock<FxHashMap<String, String>>,
    /// Ligue 1 token IDs (for buffer calculation)
    pub ligue1_tokens: RwLock<FxHashMap<String, ()>>,
    /// Token ID -> live status (for GTD expiry calculation)
    pub live_status: RwLock<FxHashMap<String, bool>>,
    /// Last refresh timestamp (Unix seconds)
    pub last_refresh: AtomicU64,
    /// Cache statistics
    pub stats: CacheStats,
}

#[derive(Default)]
pub struct CacheStats {
    pub neg_risk_count: AtomicU64,
    pub slug_count: AtomicU64,
    pub atp_count: AtomicU64,
    pub ligue1_count: AtomicU64,
    pub live_count: AtomicU64,
    pub refresh_count: AtomicU64,
    pub last_refresh_duration_ms: AtomicU64,
}

impl MarketCaches {
    pub fn new() -> Self {
        Self {
            neg_risk: RwLock::new(FxHashMap::default()),
            slugs: RwLock::new(FxHashMap::default()),
            atp_tokens: RwLock::new(FxHashMap::default()),
            ligue1_tokens: RwLock::new(FxHashMap::default()),
            live_status: RwLock::new(FxHashMap::default()),
            last_refresh: AtomicU64::new(0),
            stats: CacheStats::default(),
        }
    }

    /// Check if token is neg_risk
    #[inline]
    pub fn is_neg_risk(&self, token_id: &str) -> Option<bool> {
        self.neg_risk.read().ok()?.get(token_id).copied()
    }

    /// Get slug for token
    #[inline]
    pub fn get_slug(&self, token_id: &str) -> Option<String> {
        self.slugs.read().ok()?.get(token_id).cloned()
    }

    /// Check if token is ATP market
    #[inline]
    pub fn is_atp_token(&self, token_id: &str) -> bool {
        self.atp_tokens
            .read()
            .map(|c| c.contains_key(token_id))
            .unwrap_or(false)
    }

    /// Check if token is Ligue 1 market
    #[inline]
    pub fn is_ligue1_token(&self, token_id: &str) -> bool {
        self.ligue1_tokens
            .read()
            .map(|c| c.contains_key(token_id))
            .unwrap_or(false)
    }

    /// Get ATP buffer for token (0.01 if ATP, 0.0 otherwise)
    #[inline]
    pub fn get_atp_buffer(&self, token_id: &str) -> f64 {
        if self.is_atp_token(token_id) {
            ATP_BUFFER
        } else {
            0.0
        }
    }

    /// Get Ligue 1 buffer for token (0.01 if Ligue1, 0.0 otherwise)
    #[inline]
    pub fn get_ligue1_buffer(&self, token_id: &str) -> f64 {
        if self.is_ligue1_token(token_id) {
            LIGUE1_BUFFER
        } else {
            0.0
        }
    }

    /// Get live status for token (for GTD expiry calculation)
    #[inline]
    pub fn get_is_live(&self, token_id: &str) -> Option<bool> {
        self.live_status.read().ok()?.get(token_id).copied()
    }

    /// Insert neg_risk value for a token (for dynamic updates)
    pub fn set_neg_risk(&self, token_id: String, neg_risk: bool) {
        if let Ok(mut cache) = self.neg_risk.write() {
            cache.insert(token_id, neg_risk);
        }
    }

    /// Insert slug for a token (for dynamic updates)
    pub fn set_slug(&self, token_id: String, slug: String) {
        if let Ok(mut cache) = self.slugs.write() {
            cache.insert(token_id, slug);
        }
    }

    /// Get cache statistics summary
    pub fn get_stats_summary(&self) -> String {
        format!(
            "Caches: neg_risk={}, slugs={}, atp={}, ligue1={}, refreshes={}",
            self.stats.neg_risk_count.load(Ordering::Relaxed),
            self.stats.slug_count.load(Ordering::Relaxed),
            self.stats.atp_count.load(Ordering::Relaxed),
            self.stats.ligue1_count.load(Ordering::Relaxed),
            self.stats.refresh_count.load(Ordering::Relaxed),
        )
    }

    /// Check if cache refresh is needed
    pub fn needs_refresh(&self) -> bool {
        let last = self.last_refresh.load(Ordering::Relaxed);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - last >= CACHE_REFRESH_INTERVAL_SECS
    }
}

impl Default for MarketCaches {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct CacheLoadResult {
    pub neg_risk_loaded: usize,
    pub slugs_loaded: usize,
    pub atp_loaded: usize,
    pub ligue1_loaded: usize,
    pub live_loaded: usize,
    pub load_time_ms: u64,
}

impl std::fmt::Display for CacheLoadResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Loaded caches in {}ms: neg_risk={}, slugs={}, atp={}, ligue1={}, live={}",
            self.load_time_ms,
            self.neg_risk_loaded,
            self.slugs_loaded,
            self.atp_loaded,
            self.ligue1_loaded,
            self.live_loaded
        )
    }
}

// ============================================================================
// Global Cache Instance
// ============================================================================

use std::sync::OnceLock;

static GLOBAL_CACHES: OnceLock<MarketCaches> = OnceLock::new();

/// Get the global cache instance
pub fn global_caches() -> &'static MarketCaches {
    GLOBAL_CACHES.get_or_init(MarketCaches::new)
}

// ============================================================================
// Convenience Functions (for backwards compatibility)
// ============================================================================

/// Get ATP buffer for a token (convenience function)
#[inline]
pub fn get_atp_token_buffer(token_id: &str) -> f64 {
    global_caches().get_atp_buffer(token_id)
}

/// Get Ligue 1 buffer for a token (convenience function)
#[inline]
pub fn get_ligue1_token_buffer(token_id: &str) -> f64 {
    global_caches().get_ligue1_buffer(token_id)
}

/// Get slug for a token (convenience function)
#[inline]
pub fn get_slug(token_id: &str) -> Option<String> {
    global_caches().get_slug(token_id)
}

/// Check if token is neg_risk (convenience function)
#[inline]
pub fn is_neg_risk(token_id: &str) -> Option<bool> {
    global_caches().is_neg_risk(token_id)
}

/// Get is_live for a token (convenience function)
#[inline]
pub fn get_is_live(token_id: &str) -> Option<bool> {
    global_caches().get_is_live(token_id)
}