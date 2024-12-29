use std::time::Instant;

pub struct EventLoop {
    period: f32,
}

impl EventLoop {
    pub fn new(frequency: u16) -> Self {
        EventLoop {
            period: 1.0 / f32::from(frequency)
        }
    }

    pub fn begin(&mut self) {
        let mut previous_time = Instant::now();
        let mut accumulator: f32 = 0.0;

        loop {
            let current_time = Instant::now();
            let delta_time = previous_time.elapsed();
            previous_time = current_time;
    
            accumulator += delta_time.as_secs_f32();
            while accumulator >= self.period {
                // Fixed update goes here
                accumulator -= self.period;
            }

            let _alpha = accumulator / self.period;
            // render with alpha as interpolation factor
        }        
    }
}
