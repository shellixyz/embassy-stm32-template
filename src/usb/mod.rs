use core::sync::atomic::{self, AtomicBool};

use derive_more::IsVariant;
use embassy_executor::Spawner;
use embassy_sync::channel;
use embassy_time::{Duration, TimeoutError, with_timeout};
use embassy_usb::driver::EndpointError;
use serde::{Deserialize, Serialize};

use crate::{
	ChannelReceiver, ChannelSender,
	board::{CdcAcmClass, CdcAcmSender, UsbDevice},
};

pub mod messages;
pub mod tasks;

const CHANNEL_LEN: usize = 5;
const SEND_PACKET_TIMEOUT: Duration = Duration::from_millis(50);

type OutgoingChannelTx = ChannelSender<messages::Outgoing, CHANNEL_LEN>;
type OutgoingChannelRx = ChannelReceiver<messages::Outgoing, CHANNEL_LEN>;
type IncomingChannelRx = ChannelReceiver<messages::Incoming, CHANNEL_LEN>;
type IncomingChannelTx = ChannelSender<messages::Incoming, CHANNEL_LEN>;

static mut OUTGOING_CHANNEL: crate::Channel<messages::Outgoing, CHANNEL_LEN> = crate::Channel::new();
static mut INCOMING_CHANNEL: crate::Channel<messages::Incoming, CHANNEL_LEN> = crate::Channel::new();

static IS_RESET_ACK: AtomicBool = AtomicBool::new(false);
static REQUESTING_RESET: AtomicBool = AtomicBool::new(false);
static CONNECTED: AtomicBool = AtomicBool::new(false);

pub struct Channel {
	outgoing_tx: OutgoingChannelTx,
	incoming_rx: IncomingChannelRx,
}

impl Channel {
	pub fn try_send(&self, message: messages::Outgoing) -> Result<(), channel::TrySendError<messages::Outgoing>> {
		self.outgoing_tx.try_send(message)
	}

	pub fn try_receive(&self) -> Result<messages::Incoming, channel::TryReceiveError> {
		self.incoming_rx.try_receive()
	}
}

fn outgoing_channel_endpoints() -> (OutgoingChannelRx, OutgoingChannelTx) {
	unsafe { (OUTGOING_CHANNEL.receiver(), OUTGOING_CHANNEL.sender()) }
}

fn incoming_channel_endpoints() -> (IncomingChannelTx, IncomingChannelRx) {
	unsafe { (INCOMING_CHANNEL.sender(), INCOMING_CHANNEL.receiver()) }
}

pub fn spawn_tasks(spawner: &Spawner, usb_device: UsbDevice, cdc_acm: CdcAcmClass) -> Channel {
	let (outgoing_rx, outgoing_tx) = outgoing_channel_endpoints();
	let (incoming_tx, incoming_rx) = incoming_channel_endpoints();

	spawner.must_spawn(tasks::driver(usb_device));

	let (cdc_acm_sender, cdc_acm_receiver) = cdc_acm.split();
	spawner.must_spawn(tasks::handle_rx(cdc_acm_receiver, incoming_tx));
	spawner.must_spawn(tasks::handle_tx(cdc_acm_sender, outgoing_rx));

	Channel {
		outgoing_tx,
		incoming_rx,
	}
}

pub fn reset_ack() -> messages::Outgoing {
	IS_RESET_ACK.store(true, atomic::Ordering::SeqCst);
	messages::Outgoing::Acknowledgement
}

pub fn is_requesting_reset() -> bool {
	REQUESTING_RESET.load(atomic::Ordering::SeqCst)
}

async fn send_packet(cdc_acm_sender: &mut CdcAcmSender, packet: &[u8]) -> Result<(), EndpointError> {
	for data in packet.chunks(64) {
		match with_timeout(SEND_PACKET_TIMEOUT, cdc_acm_sender.write_packet(data)).await {
			Ok(result) => result?,
			Err(TimeoutError) => break,
		}
	}
	Ok(())
}
