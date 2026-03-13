use std::collections::HashMap;

pub struct TimePoint {
    pub timestamp_ms: u64,
    pub values: Vec<u64>,
}

pub struct SeriesAccess {
    series: HashMap<String, Vec<TimePoint>>,
}

impl SeriesAccess {
    pub fn new() -> Self {
        Self {
            series: HashMap::new(),
        }
    }

    pub fn append(&mut self, series_name: &str, point: TimePoint) {
        self.series
            .entry(series_name.to_string())
            .or_default()
            .push(point);
    }

    pub fn range(
        &self,
        series_name: &str,
        start_ms: u64,
        end_ms: u64,
    ) -> Vec<&TimePoint> {
        self.series
            .get(series_name)
            .map(|pts| {
                pts.iter()
                    .filter(|p| p.timestamp_ms >= start_ms && p.timestamp_ms <= end_ms)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn latest(&self, series_name: &str) -> Option<&TimePoint> {
        self.series.get(series_name).and_then(|pts| pts.last())
    }

    pub fn count(&self, series_name: &str) -> usize {
        self.series.get(series_name).map(|pts| pts.len()).unwrap_or(0)
    }
}

impl Default for SeriesAccess {
    fn default() -> Self {
        Self::new()
    }
}
