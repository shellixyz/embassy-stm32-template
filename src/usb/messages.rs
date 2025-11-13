use derive_more::IsVariant;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, IsVariant)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FirmwareToConnectedDevice {
	Acknowledgement,
}

#[derive(Serialize, Deserialize)]
pub enum ConnectedDeviceToFirmware {
	Reset,
	ExampleMessage,
}
