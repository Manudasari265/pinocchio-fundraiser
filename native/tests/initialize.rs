use crate::connect;
use solana_sdk::{
    account::{AccountSharedData, WritableAccount},
    instruction::{AccountMeta, Instruction},
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
};
use crate::state::{
    Fundraiser,
};

#[test]
fn initialize() {
    let (program_id, mollusk) = connect();

    
}