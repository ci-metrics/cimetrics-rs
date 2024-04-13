use std::collections::BTreeMap;
use std::io::Write;
use std::sync::Mutex;
use std::sync::OnceLock;

static METRICS: Metrics = Metrics::new();

struct Metrics(OnceLock<MetricsInner>);

/// Use a `BTreeMap` over a `HashMap` so metrics are output in a consistent order and give a useful
/// diff when commited.
struct MetricsInner(Mutex<(u64, BTreeMap<&'static str, u64>)>);

pub struct MetricsHandle<'a>(&'a MetricsInner);

pub fn handle() -> MetricsHandle<'static> {
    let metrics = METRICS
        .0
        .get_or_init(|| MetricsInner(Mutex::new((0, BTreeMap::new()))));
    let mut guard = metrics.0.lock().unwrap();
    let (count, _map) = &mut *guard;
    *count += 1;

    MetricsHandle(metrics)
}

impl Metrics {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }
}

impl MetricsHandle<'_> {
    pub fn add(&self, name: &'static str, value: u64) {
        let mut guard = self.0 .0.lock().unwrap();
        let (_count, map) = &mut *guard;
        map.insert(name, value);
    }
}

impl Drop for MetricsHandle<'_> {
    fn drop(&mut self) {
        use std::fmt::Write;

        let mut guard = self.0 .0.lock().unwrap();
        let (count, map) = &mut *guard;
        *count -= 1;
        if *count == 0 {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open("metrics.csv")
                .unwrap();
            let csv = map.iter_mut().fold(String::new(), |mut acc, (k, v)| {
                writeln!(acc, "{k},{v}").unwrap();
                acc
            });
            file.write_all(csv.as_bytes()).unwrap();
        }
    }
}
