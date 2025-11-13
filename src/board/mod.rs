use defmt::Format;
use embassy_stm32::{
	bind_interrupts,
	gpio::{self},
	peripherals, set_pins_as_output, usb,
	wdg::IndependentWatchdog,
};
use embassy_time::Duration;
use getset::CopyGetters;
use static_cell::StaticCell;

pub type Switch<'a, P> = switch_hal::Switch<gpio::Output<'a>, P>;
pub type ActiveLowSwitch<'a> = Switch<'a, switch_hal::ActiveLow>;
pub type ActiveHighSwitch<'a> = Switch<'a, switch_hal::ActiveHigh>;
pub type Input<'a, P> = switch_hal::Switch<gpio::Input<'a>, P>;
pub type ActiveHighInput<'a> = Input<'a, switch_hal::ActiveHigh>;

bind_interrupts!(struct Irqs {
	USB_LP => usb::InterruptHandler<peripherals::USB>;
});

pub type Flash<'a> = embassy_stm32::flash::Flash<'a, embassy_stm32::flash::Blocking>;
pub type WDG = IndependentWatchdog<'static, peripherals::IWDG>;

pub type UsbDevice = embassy_usb::UsbDevice<'static, usb::Driver<'static, peripherals::USB>>;
pub type CdcAcmClass = embassy_usb::class::cdc_acm::CdcAcmClass<'static, usb::Driver<'static, peripherals::USB>>;
pub type CdcAcmSender = embassy_usb::class::cdc_acm::Sender<'static, usb::Driver<'static, peripherals::USB>>;
pub type CdcAcmReceiver = embassy_usb::class::cdc_acm::Receiver<'static, usb::Driver<'static, peripherals::USB>>;

static CDC_ACM_STATE: StaticCell<embassy_usb::class::cdc_acm::State> = StaticCell::new();
static USB_CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static USB_BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static USB_CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

#[derive(Debug, Clone, Copy, PartialEq, CopyGetters, Format)]
pub struct Config {}

pub struct Peripherals {
	pub usb: UsbDevice,
	pub cdc_acm: CdcAcmClass,
	pub wdg: Option<WDG>,
}

fn set_clock_config(config: &mut embassy_stm32::Config) {
	use embassy_stm32::rcc::*;
	// config.rcc.hsi = false;
	// config.rcc.hse = Some(Hse {
	// 	freq: Hertz::mhz(16),
	// 	mode: HseMode::Oscillator,
	// });
	// config.rcc.sys = Sysclk::PLL1_R;
	// config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
	// config.rcc.pll = Some(Pll {
	// 	source: PllSource::HSE,
	// 	prediv: PllPreDiv::DIV4,
	// 	mul: PllMul::MUL85,
	// 	divp: Some(PllPDiv::DIV2),
	// 	divq: Some(PllQDiv::DIV2),
	// 	divr: Some(PllRDiv::DIV2),
	// });
	// config.rcc.ahb_pre = AHBPrescaler::DIV1;
	// config.rcc.apb1_pre = APBPrescaler::DIV1;
	// config.rcc.apb2_pre = APBPrescaler::DIV1;
	// config.rcc.ls = LsConfig::off();
	// config.rcc.mux.adc12sel = mux::Adcsel::PLL1_P;
	// config.rcc.mux.adc345sel = mux::Adcsel::DISABLE;
	// config.rcc.mux.clk48sel = mux::Clk48sel::HSI48;
	// config.rcc.mux.fdcansel = mux::Fdcansel::PLL1_Q;
}

pub const DEFAULT_CONFIG: Config = Config {};

pub async fn init_default(watchdog_timeout: Option<Duration>) -> Peripherals {
	init(DEFAULT_CONFIG, watchdog_timeout).await
}

pub async fn init(config: Config, watchdog_timeout: Option<Duration>) -> Peripherals {
	let mut stm32_config = embassy_stm32::Config::default();
	set_clock_config(&mut stm32_config);

	let p = embassy_stm32::init(stm32_config);

	// #[rustfmt::skip]
	// set_pins_as_output!(
	// 	p,
	// 	PA1, PA2, PA3, ...
	// );

	#[cfg(not(feature = "swd"))]
	set_pins_as_output!(p, PA13, PA14);

	let wdg = watchdog_timeout.map(|wdgt| IndependentWatchdog::new(p.IWDG, u32::try_from(wdgt.as_micros()).unwrap()));

	let (usb, cdc_acm) = init_usb(p.USB, p.PA11, p.PA12);

	Peripherals { usb, cdc_acm, wdg }
}

fn init_usb(usb: peripherals::USB, pa11: peripherals::PA11, pa12: peripherals::PA12) -> (UsbDevice, CdcAcmClass) {
	let driver = embassy_stm32::usb::Driver::new(usb, Irqs, pa12, pa11);

	let mut config = embassy_usb::Config::new(0xC0DE, 0xCAFE);
	config.manufacturer = Some("Shellixyz");
	config.product = Some(crate::version::PRODUCT_DESCRIPTION);
	config.serial_number = Some("123456");

	config.device_class = 0xEF;
	config.device_sub_class = 0x02;
	config.device_protocol = 0x01;
	config.composite_with_iads = true;

	let config_descriptor = USB_CONFIG_DESCRIPTOR.init([0; 256]);
	let bos_descriptor = USB_BOS_DESCRIPTOR.init([0; 256]);
	let control_buf = USB_CONTROL_BUF.init([0; 64]);

	let state = CDC_ACM_STATE.init(embassy_usb::class::cdc_acm::State::new());

	let mut builder = embassy_usb::Builder::new(
		driver,
		config,
		config_descriptor,
		bos_descriptor,
		&mut [], // no msos descriptors
		control_buf,
	);

	let class = embassy_usb::class::cdc_acm::CdcAcmClass::new(&mut builder, state, 64);

	let usb = builder.build();
	(usb, class)
}

impl Peripherals {
	pub fn unleash_dog(&mut self) {
		if let Some(wdg) = self.wdg.as_mut() {
			wdg.unleash();
		}
	}

	pub fn pet_dog(&mut self) {
		if let Some(wdg) = self.wdg.as_mut() {
			wdg.pet();
		}
	}
}
