use std::sync::{Arc, Barrier, Mutex, RwLock,
                RwLockReadGuard, RwLockWriteGuard, MutexGuard, PoisonError};
use rustwlc::WlcOutput;

// 1366x768 is most common screen size.
const DEFAULT_SCREEN_SIZE: usize = 1366 * 768;

lazy_static! {
    static ref SCREEN_SCRAPE: RwLock<bool> = RwLock::new(false);
    pub static ref SCRAPED_PIXELS: Mutex<(Vec<u8>, Option<WlcOutput>)> = {
        Mutex::new((Vec::with_capacity(DEFAULT_SCREEN_SIZE), None))
    };
    static ref SCRAPE_BARRIER: Arc<Barrier> = Arc::new(Barrier::new(2));
}

pub fn read_screen_scrape_lock<'a>() -> RwLockReadGuard<'a, bool> {
    SCREEN_SCRAPE.read().expect("Lock was poisoned!")
}

pub fn write_screen_scrape_lock<'a>() -> RwLockWriteGuard<'a, bool> {
    trace!("Writing screen scrape lock");
    SCREEN_SCRAPE.write().expect("Lock was poisoned!")
}

pub fn scraped_pixels_lock() -> Result<MutexGuard<'static, (Vec<u8>, Option<WlcOutput>)>,
                                       PoisonError<MutexGuard<'static, (Vec<u8>, Option<WlcOutput>)>>> {
    trace!("Locking scraped pixels lock");
    SCRAPED_PIXELS.lock()
}

/// Ensure that the thread that's waiting is synced up with the main thread before
/// continuing either.
pub fn sync_scrape() {
    SCRAPE_BARRIER.wait();
    trace!("Screen scraped barrier synced!")
}
