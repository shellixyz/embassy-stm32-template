let targets = #{
    STM32G474RETx: #{
        rust_target: "thumbv7m-none-eabi",
        chip_family: "stm32g4x",
        embassy_chip_feature: "stm32g474re",
    },
    STM32F405RGTx: #{
        rust_target: "thumbv7m-none-eabi",
        chip_family: "stm32f4x",
        embassy_chip_feature: "stm32f405rg",
    },
    STM32F415RGTx: #{
        rust_target: "thumbv7m-none-eabi",
        chip_family: "stm32f4x",
        embassy_chip_feature: "stm32f415rg",
    },
};

let target = variable::get("chip");

let target_properties = targets.get(target);
for key in target_properties.keys() {
    variable::set(key, target_properties.get(key));
}
