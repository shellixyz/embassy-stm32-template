use core::sync::atomic;

use embassy_futures::yield_now;
use postcard::accumulator::CobsAccumulator;

use super::CONNECTED;
use crate::{
	board::{CdcAcmReceiver, CdcAcmSender, UsbDevice},
	usb::{IncomingChannelTx, OutgoingChannelRx},
};

#[embassy_executor::task]
pub async fn driver(mut usb: UsbDevice) {
	defmt::info!("USB driver task started");
	usb.run().await;
}

#[embassy_executor::task]
pub async fn handle_tx(mut cdc_adm_sender: CdcAcmSender, firmware_to_usb: OutgoingChannelRx) {
	loop {
		yield_now().await;
		if let Ok(message) = firmware_to_usb.try_receive() {
			if !CONNECTED.load(atomic::Ordering::SeqCst) {
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
			if super::send_packet(&mut cdc_adm_sender, serialized).await.is_err() {
				defmt::error!("Failed to send message body");
				continue;
			}
			if message.is_acknowledgement() && super::IS_RESET_ACK.load(atomic::Ordering::SeqCst) {
				// defmt::info!("Setting do reset flag");
				super::REQUESTING_RESET.store(true, atomic::Ordering::SeqCst);
			}
		}
	}
}

#[embassy_executor::task]
pub async fn handle_rx(mut cdc_acm_receiver: CdcAcmReceiver, usb_to_firmware: IncomingChannelTx) {
	const BUF_SIZE: usize = 8192;
	let mut cobs_buf: CobsAccumulator<{ BUF_SIZE }> = CobsAccumulator::new();
	loop {
		cdc_acm_receiver.wait_connection().await;
		defmt::info!("USB connected");
		CONNECTED.store(true, atomic::Ordering::SeqCst);
		loop {
			let mut packet_buf = [0u8; 64];
			let Ok(size) = cdc_acm_receiver.read_packet(&mut packet_buf).await else {
				CONNECTED.store(false, atomic::Ordering::SeqCst);
				defmt::info!("USB disconnected");
				break;
			};

			let buf = &packet_buf[..size];
			let mut window = buf;

			use postcard::accumulator::FeedResult;
			'cobs: while !window.is_empty() {
				window = match cobs_buf.feed::<super::messages::Incoming>(window) {
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
