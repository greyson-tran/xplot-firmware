#![no_std]
#![no_main]

mod motor;
mod usb;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::*;

use embassy_executor::Spawner;

use embassy_rp::gpio::{Level, Output};

use embassy_time::{Duration, Timer};

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

// use motor::Servo;
use usb::{Serial, USBINCHANNEL, USBOUTCHANNEL};

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Program start");

    let pac = embassy_rp::init(Default::default());

    // let receiver = USBOUTCHANNEL.receiver();
    // let sender = USBINCHANNEL.sender();

    let mut serial_if = Serial::new(pac.USB).await; // Build the Serial Interface

    spawner.spawn(start_server(serial_if, spawner)); // Start the USB Server
    // spawner.spawn(echo(sender, receiver)); // Start the USB Server

    let mut status = Output::new(pac.PIN_25, Level::Low);
    loop {
        status.set_high();
        Timer::after(Duration::from_millis(250)).await;
        status.set_low();
        Timer::after(Duration::from_millis(250)).await;
    }
}

#[embassy_executor::task]
async fn start_server(mut serial_if: Serial, spawner: Spawner) {
    serial_if.server(spawner).await;
}

// #[embassy_executor::task]
// async fn echo(
//     sender: Sender<'static, CriticalSectionRawMutex, [u8; 8], 1>,
//     receiver: Receiver<'static, CriticalSectionRawMutex, [u8; 8], 1>,
// ) {
//     loop {
//         let _ = match receiver.try_receive() {
//             Ok(message) => sender.send(message).await,
//             Err(_) => continue,
//         };
//         Timer::after(Duration::from_millis(250)).await;
//     }
// }

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
