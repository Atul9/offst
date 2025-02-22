#![feature(async_await, await_macro, arbitrary_self_types)]
#![feature(nll)]
#![feature(generators)]
#![feature(never_type)]
#![deny(trivial_numeric_casts, warnings)]
#![allow(intra_doc_link_resolution_failure)]
#![allow(
    clippy::too_many_arguments,
    clippy::implicit_hasher,
    clippy::module_inception,
    clippy::new_without_default
)]

#[macro_use]
extern crate prettytable;

// #[macro_use]
// extern crate log;

#[macro_use]
extern crate serde_derive;

mod multi_route_util;

pub mod buyer;
pub mod config;
pub mod file;
pub mod info;
pub mod seller;
pub mod utils;

pub mod stctrllib;
pub mod stverifylib;
