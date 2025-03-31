use defmt::{info, panic, unwrap};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
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

pub struct Serial {
    class: CdcAcmClass<'static, Driver<'static, USB>>,
    usb: Option<UsbDevice<'static, Driver<'static, USB>>>,
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
            usb: Some(usb), // Wrap usb in Some
        }
    }

    pub async fn start_server(&mut self, spawner: Spawner) -> ! {
        // Take ownership of usb, replacing it with None
        let usb = self.usb.take().unwrap();
        unwrap!(spawner.spawn(usb_task(usb)));

        loop {
            self.class.wait_connection().await;
            info!("Serial interface has been connected.");
            let _ = echo(&mut self.class).await;
            info!("Serial interface has been disconnected.")
        }
    }
}

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, Driver<'static, USB>>) -> ! {
    device.run().await
}

async fn echo(class: &mut CdcAcmClass<'static, Driver<'static, USB>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data[0] as char);
        class.write_packet(data).await?;
    }
}
