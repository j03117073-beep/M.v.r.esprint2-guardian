// Deterministic core abstractions: DetMap, DetScheduler, DetTime, DetRng
#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::time::{SystemTime as WallClock, UNIX_EPOCH};

/// Deterministic map abstraction. Uses BTreeMap internally to provide
/// deterministic iteration order. API is intentionally small and safe.
pub struct DetMap<K, V>
where
    K: Ord,
{
    inner: BTreeMap<K, V>,
}

impl<K, V> DetMap<K, V>
where
    K: Ord,
{
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.inner.insert(k, v)
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.inner.get(k)
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.inner.remove(k)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.inner.iter()
    }
}

/// Deterministic scheduler and task abstraction. Tasks are executed in
/// deterministic order determined by a numeric order_key. This is the
/// sanctioned scheduler to avoid ad-hoc HashMap/BTreeMap use across the
/// codebase.
pub struct DetScheduler {
    pub tick: u64,
    pub task_queue: BTreeMap<u64, Vec<Box<dyn DetTask>>>,
}

impl DetScheduler {
    pub fn new() -> Self {
        Self {
            tick: 0,
            task_queue: BTreeMap::new(),
        }
    }

    pub fn step(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn has_pending(&self) -> bool {
        !self.task_queue.is_empty()
    }

    pub fn enqueue<T: DetTask + 'static>(&mut self, order_key: u64, task: T) {
        self.task_queue
            .entry(order_key)
            .or_insert_with(Vec::new)
            .push(Box::new(task));
    }

    pub fn dequeue_next(&mut self) -> Option<Box<dyn DetTask>> {
        if let Some((&key, tasks)) = self.task_queue.iter_mut().next() {
            if let Some(task) = tasks.pop() {
                if tasks.is_empty() {
                    self.task_queue.remove(&key);
                }
                return Some(task);
            }
        }
        None
    }
}

/// Deterministic task interface for the scheduler
pub trait DetTask: Debug {
    fn run(&mut self, scheduler: &mut DetScheduler) -> Result<(), String>;
}

/// Deterministic time abstraction. Wraps a user-supplied canonical time
/// value. This keeps code from avoiding direct wall-clock API calls from
/// the deterministic subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DetTime(pub u128);

impl DetTime {
    pub fn from_millis(ms: u128) -> Self {
        DetTime(ms)
    }

    pub fn as_millis(&self) -> u128 {
        self.0
    }

    /// Construct a canonical time marker from the system clock at the boundary.
    /// This is the only sanctioned direct wall clock access for the deterministic
    /// subsystem and should be used only at execution-entry points.
    pub fn canonical_now_ms() -> Self {
        DetTime(
            WallClock::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
        )
    }
}

/// Small deterministic RNG (xorshift64*) for sealed, reproducible randomness.
/// Not cryptographically secure; intended for deterministic internal uses only.
#[derive(Debug, Clone)]
pub struct DetRng {
    state: u64,
}

impl DetRng {
    pub fn seeded(seed: u64) -> Self {
        let mut s = seed.wrapping_add(0x9E3779B97F4A7C15);
        if s == 0 {
            s = 0x6A09E667F3BCC909;
        }
        Self { state: s }
    }

    pub fn next_u64(&mut self) -> u64 {
        // xorshift64* variant
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detmap_basic() {
        let mut m = DetMap::new();
        m.insert(2, "b");
        m.insert(1, "a");
        let keys: Vec<_> = m.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, vec![1, 2]);
    }

    #[test]
    fn rng_repro() {
        let mut a = DetRng::seeded(42);
        let mut b = DetRng::seeded(42);
        assert_eq!(a.next_u64(), b.next_u64());
        assert_eq!(a.next_u32(), b.next_u32());
    }
}
