#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(never_type)]
#![warn(clippy::pedantic)]
#![allow(static_mut_refs)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

mod board;

#[cfg(feature = "usb")]
mod usb;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

use core::sync::atomic::AtomicBool;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel};
use embassy_time::Timer;
use panic_probe as _;

pub type Channel<T, const N: usize> = channel::Channel<CriticalSectionRawMutex, T, N>;
pub type ChannelReceiver<T, const N: usize> = channel::Receiver<'static, CriticalSectionRawMutex, T, N>;
pub type ChannelSender<T, const N: usize> = channel::Sender<'static, CriticalSectionRawMutex, T, N>;

pub const CONFIG: board::Config = board::Config {};

static DO_RESET: AtomicBool = AtomicBool::new(false);

struct RuntimeData {
	// Add fields as needed
}

#[cfg(feature = "usb")]
fn handle_messages_from_usb(
	tuner_to_firmware_rx: &usb::RxChannelRx,
	firmware_main_to_tuner_tx: &usb::TxChannelTx,
	runtime_data: &mut RuntimeData,
) {
	if let Ok(message) = tuner_to_firmware_rx.try_receive() {
		// defmt::info!("main got message: {:?}", message);
		use usb::messages::{
			ConnectedDeviceToFirmware as IncomingMessage, FirmwareToConnectedDevice as OutgoingMessage,
		};
		let response = match message {
			IncomingMessage::Reset => {
				defmt::info!("Received Reset command from USB device");
				usb::send_reset_ack();
				OutgoingMessage::Acknowledgement
			},
			IncomingMessage::ExampleMessage => {
				defmt::info!("Received ExampleMessage from USB device");
				usb::send_reset_ack();
				OutgoingMessage::Acknowledgement
			},
		};
		if let Err(e) = firmware_main_to_tuner_tx.try_send(response) {
			defmt::error!("Failed to send response: {:?}", e);
		}
	}
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let mut p = board::init(CONFIG, None).await;

	let mut runtime_data = RuntimeData {
		// Initialize fields as needed
	};

	#[cfg(feature = "usb")]
	let (firmware_to_connected_device_rx, firmware_to_connected_device_tx) = usb::tx_channel_endpoints();
	#[cfg(feature = "usb")]
	let (connected_device_to_firmware_tx, connected_device_to_firmware_rx) = usb::rx_channel_endpoints();

	#[cfg(feature = "usb")]
	{
		spawner.must_spawn(usb::tasks::driver(p.usb));

		let (cdc_acm_sender, cdc_acm_receiver) = p.cdc_acm.split();
		spawner.must_spawn(usb::tasks::handle_rx(cdc_acm_receiver, connected_device_to_firmware_tx));
		spawner.must_spawn(usb::tasks::handle_tx(cdc_acm_sender, firmware_to_connected_device_rx));
	}

	Timer::after_secs(2).await;

	if let Some(wdg) = &mut p.wdg {
		wdg.unleash()
	}

	loop {
		yield_now().await;

		if let Some(wdg) = &mut p.wdg {
			wdg.pet();
		}

		if DO_RESET.load(core::sync::atomic::Ordering::SeqCst) {
			defmt::info!("Resetting device");
			Timer::after_millis(100).await; // Give time for the USB driver to finish sending any pending data
			cortex_m::peripheral::SCB::sys_reset();
		}

		#[cfg(feature = "usb")]
		handle_messages_from_usb(
			&connected_device_to_firmware_rx,
			&firmware_to_connected_device_tx,
			&mut runtime_data,
		);

		defmt::info!("Hello from main loop");
		Timer::after_millis(500).await;
	}
}
