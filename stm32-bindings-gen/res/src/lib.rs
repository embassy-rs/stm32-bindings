#![no_std]
#![allow(non_snake_case)]
#![allow(unused)]
#![allow(non_camel_case_types)]
#![doc(html_no_source)]

mod generated {

    mod std {
        pub mod os {
            pub mod raw {
                pub type c_uint = u32;
            }
        }
    }

    // core::include!(core::concat!(core::env!("OUT_DIR"), "/bindings.rs"));
}
