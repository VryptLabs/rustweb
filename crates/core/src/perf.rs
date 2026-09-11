use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RenderSample {

    pub component: &'static str,

    pub view_time: Duration,

    pub patch_time: Duration,

    pub unnecessary: bool,
}

pub trait RenderProfiler: Send + Sync {

    fn on_sample(&self, sample: &RenderSample);

    fn on_error(&self, component: &'static str, message: &str) {
        let _ = (component, message);
    }
}

#[derive(Debug, Default)]
pub struct NoopProfiler;
impl RenderProfiler for NoopProfiler {
    fn on_sample(&self, _sample: &RenderSample) {}
}

#[derive(Debug, Default)]
pub struct UnnecessaryRenderDetector {
    inner: Mutex<DetectorState>,
}

#[derive(Debug, Default)]
struct DetectorState {
    total: u64,
    unnecessary: u64,
    total_view: Duration,
}

impl UnnecessaryRenderDetector {

    pub fn record(
        &self,
        _component: &'static str,
        view_time: Duration,
        _patch_time: Duration,
        unnecessary: bool,
    ) {
        let mut s = self.inner.lock().unwrap();
        s.total += 1;
        s.total_view += view_time;
        if unnecessary {
            s.unnecessary += 1;
        }
    }

    pub fn total(&self) -> u64 {
        self.inner.lock().unwrap().total
    }

    pub fn unnecessary(&self) -> u64 {
        self.inner.lock().unwrap().unnecessary
    }

    pub fn mean_view(&self) -> Duration {
        let s = self.inner.lock().unwrap();
        if s.total == 0 {
            Duration::ZERO
        } else {
            s.total_view / (s.total as u32)
        }
    }
}

impl RenderProfiler for UnnecessaryRenderDetector {
    fn on_sample(&self, sample: &RenderSample) {
        self.record(sample.component, sample.view_time, sample.patch_time, sample.unnecessary);
    }
}

static GLOBAL: OnceLock<Mutex<Box<dyn RenderProfiler>>> = OnceLock::new();

fn global_cell() -> &'static Mutex<Box<dyn RenderProfiler>> {
    GLOBAL.get_or_init(|| Mutex::new(Box::new(NoopProfiler)))
}

pub fn set_global_profiler(p: impl RenderProfiler + 'static) -> bool {
    if GLOBAL.get().is_some() {
        return false;
    }
    *global_cell().lock().unwrap() = Box::new(p);
    true
}

pub fn global_profiler<F: FnOnce(&dyn RenderProfiler)>(f: F) {
    if let Some(cell) = GLOBAL.get() {
        let guard = cell.lock().unwrap();
        f(&**guard);
    }
}

pub fn time_it<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let t0 = Instant::now();
    let r = f();
    (r, t0.elapsed())
}
