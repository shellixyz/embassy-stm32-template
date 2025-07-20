#![no_std]
#![feature(never_type)]
#![warn(clippy::pedantic)]
// #![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod board;

include!(concat!(env!("OUT_DIR"), "/version.rs"));
