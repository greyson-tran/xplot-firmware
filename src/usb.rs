use defmt::{info, panic, unwrap};
use embassy_executor::Spawner;

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::*;

use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config, UsbDevice};

use static_cell::StaticCell;

#[derive(Debug)]
struct Disconnected;

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected,
        }
    }
}

pub static USBINCHANNEL: Channel<CriticalSectionRawMutex, [u8; 8], 1> = Channel::new();
pub static USBOUTCHANNEL: Channel<CriticalSectionRawMutex, [u8; 8], 1> = Channel::new();

pub struct Serial {
    class: CdcAcmClass<'static, Driver<'static, USB>>,
    usb: Option<UsbDevice<'static, Driver<'static, USB>>>,
    sender: Sender<'static, CriticalSectionRawMutex, [u8; 8], 1>,
    receiver: Receiver<'static, CriticalSectionRawMutex, [u8; 8], 1>,
}

impl Serial {
    pub async fn new(usb: USB) -> Self {
        info!("Started creating new logical object: Serial Interface");

        bind_interrupts!(struct Irqs {
            USBCTRL_IRQ => InterruptHandler<USB>;
        });

        let driver = Driver::new(usb, Irqs);

        let config = {
            let mut config = Config::new(0xc0de, 0xcafe);
            config.manufacturer = Some("Eos Microsystems");
            config.product = Some("eXperimental Ploting System / XPS");
            config.serial_number = Some("XPSF");
            config
        };

        let mut builder = {
            static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
            static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
            static CONTROL_BUF: StaticCell<[u8; 256]> = StaticCell::new();
            Builder::new(
                driver,
                config,
                CONFIG_DESCRIPTOR.init([0; 256]),
                BOS_DESCRIPTOR.init([0; 256]),
                &mut [], // F*** Microsoft
                CONTROL_BUF.init([0; 256]),
            )
        };

        let class = {
            static STATE: StaticCell<State> = StaticCell::new();
            let state = STATE.init(State::new());
            CdcAcmClass::new(&mut builder, state, 64)
        };

        let usb = builder.build();

        info!("Finished creating new logical object: Serial Interface");

        Self {
            class,
            usb: Some(usb),
            sender: USBINCHANNEL.sender(),
            receiver: USBOUTCHANNEL.receiver(),
        }
    }

    pub async fn server(&mut self, spawner: Spawner) -> ! {
        let usb = self.usb.take().unwrap();
        unwrap!(spawner.spawn(usb_task(usb)));

        loop {
            self.class.wait_connection().await;
            info!("Serial interface has been connected.");
            let _ = handler(&mut self.class, &mut self.sender, &mut self.receiver).await;
            info!("Serial interface has been disconnected.")
        }
    }
}

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, Driver<'static, USB>>) -> ! {
    device.run().await
}

async fn handler(
    class: &mut CdcAcmClass<'static, Driver<'static, USB>>,
    sender: &mut Sender<'static, CriticalSectionRawMutex, [u8; 8], 1>,
    receiver: &mut Receiver<'static, CriticalSectionRawMutex, [u8; 8], 1>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];

    loop {
        let _ = class.read_packet(&mut buf).await?;

        let message: [u8; 8] = {
            let mut message = [0u8; 8];
            message[..8].copy_from_slice(&buf[..8]);
            message
        };

        // ECHO TO DEBUG
        info!("{:#?}", &message);

        sender.send(message).await;

        let _ = match receiver.try_receive() {
            Ok(message) => class.write_packet(&message).await,
            Err(_) => continue,
        };
    }
}
