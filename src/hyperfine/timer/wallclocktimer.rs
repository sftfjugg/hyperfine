use std::process::Child;
use std::time::Instant;

use hyperfine::timer::{TimerStart, TimerStop};
use hyperfine::units::Second;

pub struct WallClockTimer {
    start: Instant,
}

impl TimerStart for WallClockTimer {
    fn start() -> WallClockTimer {
        WallClockTimer {
            start: Instant::now(),
        }
    }

    fn start_for_process(_: &Child) -> Self {
        Self::start()
    }
}

impl TimerStop for WallClockTimer {
    type Result = Second;

    fn stop(&self) -> Second {
        let duration = self.start.elapsed();
        duration.as_secs() as f64 + (duration.subsec_nanos() as f64) * 1e-9
    }
}
