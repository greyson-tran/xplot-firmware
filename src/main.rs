#![no_std]
#![no_main]

mod motor;
mod usb;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

use motor::Servo;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Program start");
    let pac = embassy_rp::init(Default::default());

    // Await the Future returned by Servo::new
    let mut left_servo = Servo::new(
        Output::new(pac.PIN_10, Level::Low),
        Output::new(pac.PIN_11, Level::Low),
        7,
    )
    .await;

    let mut right_servo = Servo::new(
        Output::new(pac.PIN_12, Level::Low),
        Output::new(pac.PIN_13, Level::Low),
        173,
    )
    .await;

    // Await the async go_to_angle method
    loop {
        for angle in 7..60 {
            right_servo.go_to_angle(180 - angle).await;
            left_servo.go_to_angle(angle).await;
        }

        for angle in (7..60).rev() {
            right_servo.go_to_angle(180 - angle).await;
            left_servo.go_to_angle(angle).await;
        }
    }
}
