#[macro_export]
macro_rules! set_pins_as_output {
	($peripherals:ident, $pin:ident $(, $other_pins:ident)+$(,)?) => {
		set_pins_as_output!($peripherals, $pin);
		set_pins_as_output!($peripherals, $($other_pins),+);
	};
	($peripherals:ident, $pin:ident) => {
		let output = gpio::Output::new($peripherals.$pin, gpio::Level::Low, gpio::Speed::Low);
		core::mem::forget(output);
	};
}
