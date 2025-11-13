default:
    @echo No receipe specified

size:
    DEFMT_LOG=info cargo size --release --bin {{project-name}}

build:
    DEFMT_LOG=info cargo build --release --bin {{project-name}}
    mkdir -p bin && ln -sf ../target/{{rust_target}}/release/{{project-name}} bin/{{project-name}}

build-swd:
    DEFMT_LOG=info cargo build --release --features swd --bin {{project-name}}
    mkdir -p bin && ln -sf ../target/{{rust_target}}/release/{{project-name}} bin/{{project-name}}

run: build-swd
    probe-rs run --chip {{chip}} bin/{{project-name}}

build-bin: build
    arm-none-eabi-objcopy -O binary bin/{{project-name}}{,.bin}

flash:
    DEFMT_LOG=info cargo flash --release --chip {{chip}} --features swd --bin {{project-name}}

dfu-flash: build-bin
	dfu-util -a 0 -s 0x08000000:leave -D bin/{{project-name}}.bin

st-flash: build-bin
	st-flash write bin/{{project-name}}.bin 0x08000000

stm32pcli-display-protection:
	STM32_Programmer -c port=usb1 -ob displ

stm32pcli-lock:
	STM32_Programmer -c port=usb1 -ob rdp=0xBB

stm32pcli-unlock:
	STM32_Programmer -c port=usb1 -rdu

unlock:
	openocd -f interface/stlink.cfg -f target/{{chip_family}}.cfg -c 'init; reset halt; {{chip_family}} unlock 0; reset halt; exit'

lock:
	openocd -f interface/stlink.cfg -f target/{{chip_family}}.cfg -c 'init; reset halt; {{chip_family}} lock 0; reset halt; exit'

dfu-unlock:
	dfu-util -a 0 -s :unprotect:force
	@for i in $(seq 20 -1 1); do echo -n "$i "; sleep 1; done
	@echo done

dfu-erase:
        dfu-util -a 0 -s :mass-erase:force

clean-build:
    cargo clean

clean-bin:
    rm -rf bin

clean: clean-build clean-bin
