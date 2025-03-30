use defmt::*;
use embassy_rp::gpio;
use embassy_time::{Duration, Timer};
use gpio::Output;
use {defmt_rtt as _, panic_probe as _};

pub struct Servo<'a> {
    pub step_pin: Output<'a>,
    pub dir_pin: Output<'a>,

    pub angle: i16,
}

impl<'a> Servo<'a> {
    const STEPS_PER_DEGREE: f32 = 640.0 / 9.0;

    pub async fn new(step_pin: Output<'a>, dir_pin: Output<'a>, starting_angle: i16) -> Self {
        info!("Started creating new logical object: Servo");
        let servo = Self {
            step_pin: step_pin,
            dir_pin: dir_pin,

            angle: starting_angle,
        };
        info!("Finished creating new logical object: Servo");

        return servo;
    }

    pub async fn go_to_angle(&mut self, new_angle: i16) {
        let mut swap = self.angle - new_angle;

        if swap >= 0 {
            self.dir_pin.set_high();
        } else {
            swap = -swap;
            self.dir_pin.set_low();
        }

        swap = (swap as f32 * Self::STEPS_PER_DEGREE) as i16;

        for _ in 0..swap {
            self.step_pin.set_high();
            Timer::after(Duration::from_micros(100)).await;
            self.step_pin.set_low();
            Timer::after(Duration::from_micros(100)).await;
        }

        self.angle = new_angle;
    }
}
