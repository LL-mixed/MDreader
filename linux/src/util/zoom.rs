// Zoom stepping/clamping — extracted pure logic mirroring macOS ReaderModel zoom caps
// (zoomIn = min(zoom*1.1, 3.0), zoomOut = max(zoom/1.1, 0.3), resetZoom = 1.0). Kept out of
// the GTK wiring so the bounds are unit-testable (parity with ReaderModelTests).

/// Zoom range enforced on both platforms (30%–300%).
pub const MIN_ZOOM: f64 = 0.3;
pub const MAX_ZOOM: f64 = 3.0;
/// Per-step factor — one notch of zoom in/out.
const ZOOM_FACTOR: f64 = 1.1;

/// One zoom step toward (`up`) or away from the content. Clamped to [MIN_ZOOM, MAX_ZOOM].
pub fn step(zoom: f64, up: bool) -> f64 {
    if up {
        (zoom * ZOOM_FACTOR).min(MAX_ZOOM)
    } else {
        (zoom / ZOOM_FACTOR).max(MIN_ZOOM)
    }
}

/// Clamp an arbitrary zoom into the allowed range (e.g. when restoring a persisted value).
pub fn clamp(zoom: f64) -> f64 {
    zoom.clamp(MIN_ZOOM, MAX_ZOOM)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_up_grows_then_caps() {
        assert!((step(1.0, true) - 1.1).abs() < 1e-9);
        assert_eq!(step(2.9, true), MAX_ZOOM);
        assert_eq!(step(MAX_ZOOM, true), MAX_ZOOM);
    }

    #[test]
    fn step_down_shrinks_then_floors() {
        assert!((step(1.1, false) - 1.0).abs() < 1e-9);
        assert_eq!(step(0.32, false), MIN_ZOOM);
        assert_eq!(step(MIN_ZOOM, false), MIN_ZOOM);
    }

    #[test]
    fn clamp_pulls_into_range() {
        assert_eq!(clamp(5.0), MAX_ZOOM);
        assert_eq!(clamp(0.1), MIN_ZOOM);
        assert!((clamp(1.5) - 1.5).abs() < 1e-9);
        assert_eq!(clamp(MAX_ZOOM), MAX_ZOOM);
        assert_eq!(clamp(MIN_ZOOM), MIN_ZOOM);
    }
}
