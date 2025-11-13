let stm32g474_clock_config = `config.rcc.hsi = false;
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

let stm32f4x5_clock_config = `config.rcc.hsi = false;
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

let targets = #{
    STM32G474RETx (16MHz HSE): #{
        rust_target: "thumbv7m-none-eabi",
        chip_family: "stm32g4x",
        embassy_chip_feature: "stm32g474re",
        clock_config: stm32g474_clock_config,
    },
    STM32F405RGTx (8MHz HSE): #{
        rust_target: "thumbv7m-none-eabi",
        chip_family: "stm32f4x",
        embassy_chip_feature: "stm32f405rg",
        clock_config: stm32f4x5_clock_config,
    },
    STM32F415RGTx (8MHz HSE): #{
        rust_target: "thumbv7m-none-eabi",
        chip_family: "stm32f4x",
        embassy_chip_feature: "stm32f415rg",
        clock_config: stm32f4x5_clock_config,
    },
};

let target = variable::get("chip");

let target_properties = targets.get(target);
for key in target_properties.keys() {
    variable::set(key, target_properties.get(key));
}
