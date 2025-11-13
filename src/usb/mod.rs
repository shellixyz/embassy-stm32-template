use core::sync::atomic::AtomicBool;

use derive_more::IsVariant;
use embassy_time::{Duration, TimeoutError, with_timeout};
use embassy_usb::driver::EndpointError;
use serde::{Deserialize, Serialize};

use crate::{
	Channel, ChannelReceiver, ChannelSender,
	board::CdcAcmSender,
	usb::messages::{ConnectedDeviceToFirmware, FirmwareToConnectedDevice},
};

pub mod messages;
pub mod tasks;

const CHANNEL_LEN: usize = 5;
pub type TxChannelTx = ChannelSender<FirmwareToConnectedDevice, CHANNEL_LEN>;
pub type RxChannelRx = ChannelReceiver<ConnectedDeviceToFirmware, CHANNEL_LEN>;
type TxChannelRx = ChannelReceiver<FirmwareToConnectedDevice, CHANNEL_LEN>;
type RxChannelTx = ChannelSender<ConnectedDeviceToFirmware, CHANNEL_LEN>;
static mut TX_CHANNEL: Channel<FirmwareToConnectedDevice, CHANNEL_LEN> = Channel::new();
static mut RX_CHANNEL: Channel<ConnectedDeviceToFirmware, CHANNEL_LEN> = Channel::new();

static SEND_RESET_ACK: AtomicBool = AtomicBool::new(false);
static CONNECTED: AtomicBool = AtomicBool::new(false);

pub fn tx_channel_endpoints() -> (TxChannelRx, TxChannelTx) {
	unsafe { (TX_CHANNEL.receiver(), TX_CHANNEL.sender()) }
}

pub fn rx_channel_endpoints() -> (RxChannelTx, RxChannelRx) {
	unsafe { (RX_CHANNEL.sender(), RX_CHANNEL.receiver()) }
}

pub fn send_reset_ack() {
	SEND_RESET_ACK.store(true, core::sync::atomic::Ordering::SeqCst);
}

async fn send_packet(cdc_acm_sender: &mut CdcAcmSender, packet: &[u8]) -> Result<(), EndpointError> {
	for data in packet.chunks(64) {
		match with_timeout(Duration::from_millis(50), cdc_acm_sender.write_packet(data)).await {
			Ok(result) => result?,
			Err(TimeoutError) => break,
		}
	}
	Ok(())
}
