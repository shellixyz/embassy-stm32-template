let chip = variable::get("chip");

let stm32g474_config = `config.rcc.hsi = false;
	config.rcc.hse = Some(Hse {
		freq: Hertz::mhz(16),
		mode: HseMode::Oscillator,
	});
	config.rcc.sys = Sysclk::PLL1_R;
	config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
	config.rcc.pll = Some(Pll {
		source: PllSource::HSE,
		prediv: PllPreDiv::DIV4,
		mul: PllMul::MUL85,
		divp: Some(PllPDiv::DIV2),
		divq: Some(PllQDiv::DIV2),
		divr: Some(PllRDiv::DIV2),
	});
	config.rcc.ahb_pre = AHBPrescaler::DIV1;
	config.rcc.apb1_pre = APBPrescaler::DIV1;
	config.rcc.apb2_pre = APBPrescaler::DIV1;
	config.rcc.ls = LsConfig::off();
	config.rcc.mux.adc12sel = mux::Adcsel::PLL1_P;
	config.rcc.mux.adc345sel = mux::Adcsel::DISABLE;
	config.rcc.mux.clk48sel = mux::Clk48sel::HSI48;
	config.rcc.mux.fdcansel = mux::Fdcansel::PLL1_Q;`;

let stm32f4_config = `config.rcc.hsi = false;
	config.rcc.hse = Some(Hse {
		freq: Hertz::mhz(8),
		mode: HseMode::Oscillator,
	});
	config.rcc.sys = Sysclk::PLL1_P;
	config.rcc.pll = Some(Pll {
		source: PllSource::HSE,
		prediv: PllPreDiv::DIV4,
		mul: PllMul::MUL168,
		divp: Some(PllPDiv::DIV2),
		divq: Some(PllQDiv::DIV7),
		divr: None,
	});
	config.rcc.ahb_pre = AHBPrescaler::DIV1;
	config.rcc.apb1_pre = APBPrescaler::DIV4;
	config.rcc.apb2_pre = APBPrescaler::DIV2;
	config.rcc.ls = LsConfig::off();`;

let clock_config = if chip.starts_with("STM32G4") {
	stm32g474_config
} else if chip.starts_with("STM32F4") {
	stm32f4_config
} else {
	print(`Unknown chip target: ${chip}`);
	""
};

let file_path = "src/board/mod.rs";
let file_content = file::read_text(file_path);
let updated_content = file_content.replace("// CLOCK_CONFIG_PLACEHOLDER", clock_config);
file::write_text(file_path, updated_content);

print(`Configured clock settings for ${chip}`);
