#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

#[cfg(feature = "core")]
pub mod core {
    include!(concat!(env!("OUT_DIR"), "/core.rs"));
}

#[cfg(feature = "idset")]
pub mod idset {
    include!(concat!(env!("OUT_DIR"), "/idset.rs"));
}

#[cfg(feature = "hostlist")]
pub mod hostlist {
    include!(concat!(env!("OUT_DIR"), "/hostlist.rs"));
}
