#![no_std]

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "std")]
extern crate std;

pub mod error;
pub mod instruction;
pub mod state;
pub mod consts;

pinocchio_pubkey::declare_id!("7KuDrDJsLa2iKcUovWs7DFNYRdYJ12MyKyaJwnqmhSxy");