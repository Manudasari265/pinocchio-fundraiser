#![no_std]
#![allow(unexpected_cfgs)]

#[cfg(feature = "std")]
extern crate std;
#[cfg(not(feature = "no-entrypoint"))]

mod entrypoint;
pub mod error;
pub mod instruction;
pub mod state;
pub mod consts;

pinocchio_pubkey::declare_id!("7KuDrDJsLa2iKcUovWs7DFNYRdYJ12MyKyaJwnqmhSxy");