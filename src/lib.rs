#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod core {
    include!(concat!(env!("OUT_DIR"), "/core.rs"));
}
