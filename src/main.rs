#![no_std]
#![no_main]

mod motor;
mod usb;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_time::{Duration, Timer};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

use motor::Servo;
use usb::Serial;

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Program start");
    let pac = embassy_rp::init(Default::default());

    let mut serial_if = Serial::new(pac.USB).await;
    // Await the start_server call
    serial_if.start_server(spawner).await;

    Output::new(pac.PIN_26, Level::High);
    loop {
        Timer::after(Duration::from_secs(10)).await;
    }
}

// #[embassy_executor::main]
// async fn main(_spawner: Spawner) -> ! {
//     info!("Program start");
//     let pac = embassy_rp::init(Default::default());

//     // Await the Future returned by Servo::new
//     let mut left_servo = Servo::new(
//         Output::new(pac.PIN_10, Level::Low),
//         Output::new(pac.PIN_11, Level::Low),
//         7,
//     )
//     .await;

//     let mut right_servo = Servo::new(
//         Output::new(pac.PIN_12, Level::Low),
//         Output::new(pac.PIN_13, Level::Low),
//         173,
//     )
//     .await;

//     // Await the async go_to_angle method
//     loop {
//         for angle in 7..60 {
//             right_servo.go_to_angle(180 - angle).await;
//             left_servo.go_to_angle(angle).await;
//         }

//         for angle in (7..60).rev() {
//             right_servo.go_to_angle(180 - angle).await;
//             left_servo.go_to_angle(angle).await;
//         }
//     }
// }
