use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    sysvars::{
        clock::Clock,
        rent::Rent,
        Sysvar
    },
    ProgramResult,
    pubkey::Pubkey
};
use pinocchio_token::state::TokenAccount;
use pinocchio_system::instructions::CreateAccount;
use crate::{
    state::Fundraiser,
    utils::{
        DataLen,
        load_ix_data,
        load_acc_mut_unchecked,
    }
};

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InitializeInstructionData {
    pub amount: u64, // 
    pub duration: u8,
    pub bump: u8,
}

impl DataLen for InitializeInstructionData {
    const LEN: usize = core::mem::size_of::<InitializeInstructionData>();
}

pub fn process_initialize(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [maker, mint_to_raise, fundraiser, vault, _system_program, _token_program, _remaining @ ..] = accounts 
      else {
        return Err(ProgramError::InvalidAccountData);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !fundraiser.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    //? creating the vault token account
    let vault_acc = TokenAccount::from_account_info(vault)?;

    //? design choice: the vault can be created on client to save CUs
    assert_eq!(
        vault_acc.owner(),
        fundraiser.key()
    );

    //? import the rent for account space allocation
    let rent = Rent::get()?;
    //? load the instruction data
    let ix_data = unsafe {
        load_ix_data::<InitializeInstructionData>(instruction_data)?
    };

    //? extract the bump seed and construct fundraiser seeds for the pda derivatiuon 
    let bump_seed = [ix_data.bump];
    let fundraiser_seeds = [
        Seed::from(Fundraiser::SEED.as_bytes()),
        Seed::from(maker.key().as_ref()),
        Seed::from(&bump_seed[..]),
    ];

    let fundraiser_signer = Signer::from(
        &fundraiser_seeds[..],
    );

    //? now create the fundraiser account
    (CreateAccount {
        from: maker,
        to: fundraiser,
        lamports: rent.minimum_balance(Fundraiser::LEN),
        space: Fundraiser::LEN as u64,
        owner: &crate::ID,
    })
    .invoke_signed(
        &[fundraiser_signer]
    )?;

    //? load the fundraiser state account
    let fundraiser_state = (unsafe {
       load_acc_mut_unchecked::<Fundraiser>(fundraiser)
    })?;

    //? initialize the fundraiser state account
    fundraiser_state.initialize(
        *maker.key(),
        *mint_to_raise.key(),
        ix_data.amount,
        ix_data.duration,
        ix_data.bump,
        Clock::get()?.unix_timestamp
    );

    Ok(())
}