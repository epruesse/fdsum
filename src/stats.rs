use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct StatsSnapshot {
    pub entries_total: u64,
    pub entries_done: u64,
    pub bytes_total: u64,
    pub bytes_done: u64,
    pub elapsed: Duration,
}

#[derive(Debug)]
pub struct SharedStats {
    pub entries_total: AtomicU64,
    pub entries_done: AtomicU64,
    pub bytes_total: AtomicU64,
    pub bytes_done: AtomicU64,
    pub start_time: Instant,
}

impl SharedStats {
    pub fn new() -> Self {
        Self {
            entries_total: AtomicU64::new(1),
            entries_done: AtomicU64::new(0),
            bytes_total: AtomicU64::new(0),
            bytes_done: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            entries_total: self.entries_total.load(Ordering::Relaxed),
            entries_done: self.entries_done.load(Ordering::Relaxed),
            bytes_total: self.bytes_total.load(Ordering::Relaxed),
            bytes_done: self.bytes_done.load(Ordering::Relaxed),
            elapsed: self.start_time.elapsed(),
        }
    }

    fn _add_entries(&self, count: u64) {
        self.entries_total.fetch_add(count, Ordering::Relaxed);
    }
    fn _complete_entries(&self, count: u64) {
        self.entries_done.fetch_add(count, Ordering::Relaxed);
    }
    fn _add_bytes(&self, count: u64) {
        self.bytes_total.fetch_add(count, Ordering::Relaxed);
    }
    fn _complete_bytes(&self, count: u64) {
        self.bytes_done.fetch_add(count, Ordering::Relaxed);
    }

    pub fn add_entries(&self, count: u64) {
        LOCAL_STATS.with(|cell| {
            let mut stats = cell.borrow_mut();
            stats.entries_discovered += count;
            stats.maybe_flush(self);
        });
    }

    pub fn done_entries(&self, count: u64) {
        LOCAL_STATS.with(|cell| {
            let mut stats = cell.borrow_mut();
            stats.entries_completed += count;
            stats.maybe_flush(self);
        });
    }

    pub fn add_bytes(&self, count: u64) {
        LOCAL_STATS.with(|cell| {
            let mut stats = cell.borrow_mut();
            stats.bytes_discovered += count;
            stats.maybe_flush(self);
        });
    }

    pub fn done_bytes(&self, count: u64) {
        LOCAL_STATS.with(|cell| {
            let mut stats = cell.borrow_mut();
            stats.bytes_completed += count;
            stats.maybe_flush(self);
        });
    }

    pub fn spawn_display_thread(self: Arc<Self>) {
        let m = MultiProgress::new();

        let byte_bar = m.add(ProgressBar::new(0));
        byte_bar.set_style(
            ProgressStyle::with_template(
                "Bytes   [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA {eta})",
            )
            .unwrap(),
        );

        let entry_bar = m.add(ProgressBar::new(0));
        entry_bar.set_style(
            ProgressStyle::with_template(
                "Entries [{bar:40.green/white}] {pos}/{len} ({per_sec} entries/s)",
            )
            .unwrap(),
        );

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(200));
                let snap = self.snapshot();

                byte_bar.set_length(snap.bytes_total);
                byte_bar.set_position(snap.bytes_done);

                entry_bar.set_length(snap.entries_total);
                entry_bar.set_position(snap.entries_done);

                if snap.entries_done >= snap.entries_total {
                    byte_bar.finish_and_clear();
                    entry_bar.finish_and_clear();
                    break;
                }
            }
        });
    }
}

#[derive(Debug)]
pub struct LocalStats {
    pub entries_discovered: u64,
    pub entries_completed: u64,
    pub bytes_discovered: u64,
    pub bytes_completed: u64,
    pub last_flush: Instant,
}

impl LocalStats {
    pub fn new() -> Self {
        Self {
            entries_discovered: 0,
            entries_completed: 0,
            bytes_discovered: 0,
            bytes_completed: 0,
            last_flush: Instant::now(),
        }
    }

    fn maybe_flush(&mut self, stats: &SharedStats) {
        let now = Instant::now();
        if now.duration_since(self.last_flush).as_millis() >= 100 {
            if self.entries_discovered > 0 {
                stats._add_entries(self.entries_discovered);
                self.entries_discovered = 0;
            }
            if self.entries_completed > 0 {
                stats._complete_entries(self.entries_completed);
                self.entries_completed = 0;
            }
            if self.bytes_discovered > 0 {
                stats._add_bytes(self.bytes_discovered);
                self.bytes_discovered = 0;
            }
            if self.bytes_completed > 0 {
                stats._complete_bytes(self.bytes_completed);
                self.bytes_completed = 0
            }
        }
    }
}

thread_local! {
    static LOCAL_STATS: RefCell<LocalStats> = RefCell::new(LocalStats::new());
}
