#[macro_export]
macro_rules! error {
    ($fmt:expr $(, $($arg:tt)*)?) => (
        #[cfg(feature = "defmt")]
        defmt::error!($fmt $(, $($arg)*)?);
        #[cfg(not(feature = "defmt"))]
        { $($( let _ = $arg; )*)? }
    )
}

#[macro_export]
macro_rules! info {
    ($fmt:expr $(, $($arg:tt)*)?) => (
        #[cfg(feature = "defmt")]
        defmt::info!($fmt $(, $($arg)*)?);
        #[cfg(not(feature = "defmt"))]
        { $($( let _ = $arg; )*)? }
    )
}

#[macro_export]
macro_rules! warn {
    ($fmt:expr $(, $($arg:tt)*)?) => (
        #[cfg(feature = "defmt")]
        defmt::warn!($fmt $(, $($arg)*)?);
        #[cfg(not(feature = "defmt"))]
        { $($( let _ = $arg; )*)? }
    )
}

#[macro_export]
macro_rules! debug {
    ($fmt:expr $(, $($arg:tt)*)?) => (
        #[cfg(feature = "defmt")]
        defmt::debug!($fmt $(, $($arg)*)?);
        #[cfg(not(feature = "defmt"))]
        { $($( let _ = $arg; )*)? }
    )
}

#[macro_export]
macro_rules! trace {
    ($fmt:expr $(, $($arg:tt)*)?) => (
        #[cfg(feature = "defmt")]
        defmt::trace!($fmt $(, $($arg)*)?);
        #[cfg(not(feature = "defmt"))]
        { $($( let _ = $arg; )*)? }
    )
}

#[macro_export]
macro_rules! println {
    ($fmt:expr $(, $($arg:tt)*)?) => (
        #[cfg(feature = "defmt")]
        defmt::println!($fmt $(, $($arg)*)?);
        #[cfg(not(feature = "defmt"))]
        { $($( let _ = $arg; )*)? }
    )
}
