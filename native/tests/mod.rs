#[cfg(test)]
mod initialize;

#[cfg(test)]
mod contribute;

#[cfg(test)]
mod check;

#[cfg(test)]
mod refund;

use mollusk_svm::{program, Mollusk};
use solana_sdk::{
    account::{
        AccountSharedData,
        WritableAccount,
    },
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    state::AccountState,
};

pub fn connect() -> (Pubkey, Mollusk) {
    let program_id = Pubkey::new_from_array(
        five8_const::decode_32_const(
            "CFWNF69EF5YznkoL1AwzLQ8XyMVqVYyeC9aPdiyRHmSR"
        )
    );

    let mut mollusk = Mollusk::new(&program_id, "target/deploy/fundraiser-pinocchio");
    mollusk_token::token::add_program(&mut mollusk);

    (program_id, mollusk)
}