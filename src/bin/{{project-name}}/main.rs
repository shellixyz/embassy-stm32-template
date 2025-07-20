#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![allow(static_mut_refs)]
#![allow(dead_code)]
#![allow(unused_imports)]

use core::sync::atomic::AtomicBool;

use defmt_rtt as _;
use derive_more::IsVariant;
use {{project-name | snake_case}}::board::{self, CdcAcmReceiver, CdcAcmSender, Config, UsbDevice};
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel};
use embassy_time::{Duration, TimeoutError, Timer, with_timeout};
use embassy_usb::driver::EndpointError as UsbEndpointError;
use panic_probe as _;
use postcard::accumulator::CobsAccumulator;
use serde::{Deserialize, Serialize};

pub type Channel<T, const N: usize> = channel::Channel<CriticalSectionRawMutex, T, N>;
pub type ChannelReceiver<T, const N: usize> = channel::Receiver<'static, CriticalSectionRawMutex, T, N>;
pub type ChannelSender<T, const N: usize> = channel::Sender<'static, CriticalSectionRawMutex, T, N>;

pub const CONFIG: Config = Config {};

#[derive(Serialize, Deserialize, IsVariant)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum FirmwareToConnectedUSBDevice {
	Acknowledgement,
}

#[derive(Serialize, Deserialize)]
enum ConnectedUSBDeviceToFirmware {
	ExampleMessage,
}

const USB_CHANNEL_LEN: usize = 5;
type UsbTxChannelTx = ChannelSender<FirmwareToConnectedUSBDevice, USB_CHANNEL_LEN>;
type UsbTxChannelRx = ChannelReceiver<FirmwareToConnectedUSBDevice, USB_CHANNEL_LEN>;
type UsbRxChannelTx = ChannelSender<ConnectedUSBDeviceToFirmware, USB_CHANNEL_LEN>;
type UsbRxChannelRx = ChannelReceiver<ConnectedUSBDeviceToFirmware, USB_CHANNEL_LEN>;
static mut USB_TX_CHANNEL: Channel<FirmwareToConnectedUSBDevice, USB_CHANNEL_LEN> = Channel::new();
static mut USB_RX_CHANNEL: Channel<ConnectedUSBDeviceToFirmware, USB_CHANNEL_LEN> = Channel::new();
static RESET_ACK: AtomicBool = AtomicBool::new(false);
static DO_RESET: AtomicBool = AtomicBool::new(false);
static USB_CONNECTED: AtomicBool = AtomicBool::new(false);

#[embassy_executor::task]
async fn usb_driver_task(mut usb: UsbDevice) {
	defmt::info!("USB driver task started");
	usb.run().await;
}

async fn send_usb_packet(cdc_acm_sender: &mut CdcAcmSender, packet: &[u8]) -> Result<(), UsbEndpointError> {
	for data in packet.chunks(64) {
		match with_timeout(Duration::from_millis(50), cdc_acm_sender.write_packet(data)).await {
			Ok(result) => result?,
			Err(TimeoutError) => break,
		}
	}
	Ok(())
}

#[embassy_executor::task]
async fn handle_usb_tx(mut cdc_adm_sender: CdcAcmSender, firmware_to_usb: UsbTxChannelRx) {
	loop {
		if let Ok(message) = firmware_to_usb.try_receive() {
			if !USB_CONNECTED.load(core::sync::atomic::Ordering::SeqCst) {
				continue;
			}
			// defmt::info!("USB TX: {:?}", message);
			let mut buf = [0u8; 4096];
			let serialized = match postcard::to_slice_cobs(&message, &mut buf) {
				Ok(serialized) => serialized,
				Err(_error) => {
					// defmt::error!("Failed to serialize USB message: {:?}", error);
					continue;
				},
			};
			if send_usb_packet(&mut cdc_adm_sender, serialized).await.is_err() {
				defmt::error!("Failed to send message body");
				continue;
			}
			if message.is_acknowledgement() && RESET_ACK.load(core::sync::atomic::Ordering::SeqCst) {
				// defmt::info!("Setting do reset flag");
				DO_RESET.store(true, core::sync::atomic::Ordering::SeqCst);
			}
		} else {
			yield_now().await;
		}
	}
}

#[embassy_executor::task]
async fn handle_usb_rx(mut cdc_acm_receiver: CdcAcmReceiver, usb_to_firmware: UsbRxChannelTx) {
	const BUF_SIZE: usize = 8192;
	let mut cobs_buf: CobsAccumulator<{ BUF_SIZE }> = CobsAccumulator::new();
	loop {
		cdc_acm_receiver.wait_connection().await;
		defmt::info!("USB connected");
		USB_CONNECTED.store(true, core::sync::atomic::Ordering::SeqCst);
		loop {
			let mut packet_buf = [0u8; 64];
			let Ok(size) = cdc_acm_receiver.read_packet(&mut packet_buf).await else {
				USB_CONNECTED.store(true, core::sync::atomic::Ordering::SeqCst);
				defmt::info!("USB disconnected");
				break;
			};

			let buf = &packet_buf[..size];
			let mut window = buf;

			use postcard::accumulator::FeedResult;
			'cobs: while !window.is_empty() {
				window = match cobs_buf.feed::<ConnectedUSBDeviceToFirmware>(window) {
					FeedResult::Consumed => break 'cobs,
					FeedResult::OverFull(new_wind) => new_wind,
					FeedResult::DeserError(new_wind) => {
						// defmt::error!("Failed to deserialize USB message");
						new_wind
					},
					FeedResult::Success {
						data: message,
						remaining,
					} => {
						// defmt::info!("USB message received: {:?}", message);
						usb_to_firmware.try_send(message).unwrap_or_else(|_| {
							defmt::error!("Failed to send USB message to firmware");
						});
						remaining
					},
				};
			}
		}
	}
}

fn handle_messages_from_usb(
	tuner_to_firmware_rx: &UsbRxChannelRx,
	firmware_main_to_tuner_tx: &UsbTxChannelTx,
	// runtime_data: &mut RuntimeData,
) {
	if let Ok(message) = tuner_to_firmware_rx.try_receive() {
		// defmt::info!("main got message: {:?}", message);
		let response = match message {
			ConnectedUSBDeviceToFirmware::ExampleMessage => {
				defmt::info!("Received ExampleMessage from USB device");
				FirmwareToConnectedUSBDevice::Acknowledgement
			},
		};
		if let Err(e) = firmware_main_to_tuner_tx.try_send(response) {
			defmt::error!("Failed to send response: {:?}", e);
		}
	}
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let mut p = board::init(CONFIG, Some(Duration::from_secs(1))).await;

	let (firmware_to_connected_device_rx, firmware_to_connected_device_tx) =
		unsafe { (USB_TX_CHANNEL.receiver(), USB_TX_CHANNEL.sender()) };
	let (connected_device_to_firmware_tx, connected_device_to_firmware_rx) =
		unsafe { (USB_RX_CHANNEL.sender(), USB_RX_CHANNEL.receiver()) };

	spawner.must_spawn(usb_driver_task(p.usb));

	let (cdc_acm_sender, cdc_acm_receiver) = p.cdc_acm.split();
	spawner.must_spawn(handle_usb_rx(cdc_acm_receiver, connected_device_to_firmware_tx));
	spawner.must_spawn(handle_usb_tx(cdc_acm_sender, firmware_to_connected_device_rx));

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

		handle_messages_from_usb(
			&connected_device_to_firmware_rx,
			&firmware_to_connected_device_tx,
			// &mut runtime_data,
		);

		// do stuff
	}
}
