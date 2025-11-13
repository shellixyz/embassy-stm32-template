let stm32g474_clock_config = file::read_text("clock_config/stm32g474.txt");
let stm32f4x5_clock_config = file::read_text("clock_config/stm32f4x5.txt");

let targets = #{
    "STM32G474RETx (16MHz HSE)": #{
        rust_target: "thumbv7m-none-eabi",
        chip_family: "stm32g4x",
        embassy_chip_feature: "stm32g474re",
        clock_config: stm32g474_clock_config,
    },
    "STM32F405RGTx (8MHz HSE)": #{
        rust_target: "thumbv7m-none-eabi",
        chip_family: "stm32f4x",
        embassy_chip_feature: "stm32f405rg",
        clock_config: stm32f4x5_clock_config,
    },
    "STM32F415RGTx (8MHz HSE)": #{
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
