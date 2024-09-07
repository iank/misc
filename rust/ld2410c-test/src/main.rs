use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::gpio;
use esp_idf_svc::hal::uart;
use hex;

fn ld2410c_decode_radardata(buf: &[u8]) {
    log::info!("Decoded buffer: {}", hex::encode(buf));
}

fn ld2410c_check_header(got: &[u8]) -> bool {
    got == [0xf4,0xf3,0xf2,0xf1]
}

fn ld2410c_check_tail(got: &[u8]) -> bool {
    got == [0xf8,0xf7,0xf6,0xf5]
}

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("ld2410c_test init");

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let config = uart::config::Config::default().baudrate(Hertz(256_000));

    let uart: uart::UartDriver = uart::UartDriver::new(
        peripherals.uart1,
        pins.gpio10,
        pins.gpio9,
        Option::<gpio::AnyIOPin>::None,
        Option::<gpio::AnyIOPin>::None,
        &config
    ).unwrap();

    // TODO: Sync to header?

    loop {
        // Read header
        let mut header = [0u8; 4];
        match uart.read(&mut header, 1000) {
            Ok(4) if ld2410c_check_header(&header) => {},
            Ok(_) => {
                log::error!("Invalid or incomplete header: {}", hex::encode(header));
                continue;
            },
            Err(err) => {
                log::error!("Error reading header: {:?}", err);
                continue;
            }
        }

        // Read length
        let mut length_buf = [0u8; 2];
        match uart.read(&mut length_buf, 1000) {
            Ok(2) => {},
            Ok(_) => {
                log::error!("Incomplete length field");
                continue;
            },
            Err(err) => {
                log::error!("Error reading length: {:?}", err);
                continue;
            }
        }
        let length = u16::from_le_bytes(length_buf) as usize;

        // Read data
        let mut data_buf = vec![0u8; length];
        match uart.read(&mut data_buf, 1000) {
            Ok(n) if n == length => {},
            Ok(n) => {
                log::error!("Incomplete data read: expected {}, got {}", length, n);
                continue;
            },
            Err(err) => {
                log::error!("Error reading data: {:?}", err);
                continue;
            }
        }

        // Read tail
        let mut tail = [0u8; 4];
        match uart.read(&mut tail, 1000) {
            Ok(4) if ld2410c_check_tail(&tail) => {},
            Ok(_) => {
                log::error!("Invalid or incomplete tail: {}", hex::encode(tail));
                continue;
            },
            Err(err) => {
                log::error!("Error reading tail: {:?}", err);
                continue;
            }
        }

        // Decode radar data
        ld2410c_decode_radardata(&data_buf);
    }
}
