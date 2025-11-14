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
mod log_macros;
{% if usb_support == "true" %}
#[cfg(feature = "usb")]
mod usb;
{% endif %}
#[cfg(feature = "defmt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel};
use embassy_time::{Duration, Timer};
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use panic_probe as _;

pub type Channel<T, const N: usize> = channel::Channel<CriticalSectionRawMutex, T, N>;
pub type ChannelReceiver<T, const N: usize> = channel::Receiver<'static, CriticalSectionRawMutex, T, N>;
pub type ChannelSender<T, const N: usize> = channel::Sender<'static, CriticalSectionRawMutex, T, N>;

pub const CONFIG: board::Config = board::Config {};

include!(concat!(env!("OUT_DIR"), "/version.rs"));

struct RuntimeData {
	// Add fields as needed
}
{% if usb_support == "true" %}
#[cfg(feature = "usb")]
async fn handle_message_from_usb(usb_channel: &usb::Channel, runtime_data: &mut RuntimeData) {
	if let Ok(message) = usb_channel.try_receive() {
		info!("USB: got message: {:?}", message);
		use usb::messages::{Incoming as IncomingMessage, Outgoing as OutgoingMessage};
		let response = match message {
			IncomingMessage::Reset => {
				usb::reset_ack()
			},
			IncomingMessage::ExampleMessage => {
				OutgoingMessage::Acknowledgement
			},
		};
		if let Err(e) = usb_channel.try_send(response) {
			error!("USB: failed to send response: {:?}", e);
		}
	}

	if usb::is_requesting_reset() {
		info!("Resetting device");
		Timer::after_millis(100).await; // Give time for the USB driver to finish sending any pending data
		cortex_m::peripheral::SCB::sys_reset();
	}
}
{% endif %}
#[embassy_executor::main]
async fn main(spawner: Spawner) {
{%- if enable_watchdog == "true" %}
	let mut p = board::init(CONFIG, Some(Duration::from_millis({{ watchdog_duration }}))).await;
{%- else %}
	let mut p = board::init(CONFIG, None).await;
{%- endif %}

	let mut runtime_data = RuntimeData {
		// Initialize fields as needed
	};
{% if usb_support == "true" %}
	#[cfg(feature = "usb")]
	let usb_channel = usb::spawn_tasks(&spawner, p.usb, p.cdc_acm);
{% endif %}
	if let Some(wdg) = &mut p.wdg {
		wdg.unleash()
	}

	loop {
		yield_now().await;

		if let Some(wdg) = &mut p.wdg {
			wdg.pet();
		}
{% if usb_support == "true" %}
		#[cfg(feature = "usb")]
		handle_message_from_usb(&usb_channel, &mut runtime_data).await;
{% endif %}
		info!("Hello from main loop");
		Timer::after_millis(500).await;
	}
}
