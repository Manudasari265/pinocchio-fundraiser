use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError, 
    ProgramResult};

use pinocchio_token::{
    instructions::{CloseAccount,TransferChecked},
    state::{Mint, TokenAccount},
};
use crate::{
    error::FundraiserError, state::{utils::{
        load_acc, load_ix_data, DataLen
    }, Fundraiser}
};

impl DataLen for Mint {
    const LEN: usize = core::mem::size_of::<Mint>();
}

impl DataLen for TokenAccount {
    const LEN: usize = core::mem::size_of::<TokenAccount>();
}

pub fn process_check(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [maker, mint_to_raise, fundraiser, vault, maker_ata, _token_program, _system_program, _remaining @ ..] = accounts
      else {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    };

    //? check if the maker is the signer here
    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let fundraiser_state = unsafe {
        load_acc::<Fundraiser>(fundraiser.borrow_data_unchecked())?
    };
    if fundraiser_state.current_amount < fundraiser_state.amount_to_raise {
        return Err(FundraiserError::TargetNotMet.into());
    }

    //? transfer funds to the maker
    let mint_state = Mint::from_account_info(mint_to_raise)?;

    let bump_seed = [fundraiser_state.bump];
    let fundraiser_seeds = [
        Seed::from(Fundraiser::SEED.as_bytes()),
        Seed::from(maker.key().as_ref()),
        Seed::from(&bump_seed[..]),
    ];

    let fundraiser_signer = Signer::from(&fundraiser_seeds[..]);

    (TransferChecked {
        from: vault,
        to: maker_ata,
        amount: fundraiser_state.current_amount,
        authority: fundraiser,
        mint: mint_to_raise,
        decimals: mint_state.decimals(),
    })
    .invoke_signed(&[fundraiser_signer.clone()])?;

    //? close the vault account
    (CloseAccount {
        account: vault,
        destination: maker,
        authority: fundraiser,
    })
    .invoke_signed(&[fundraiser_signer.clone()])?;

    //? finally close the fundraiser account
    unsafe {
        *maker.borrow_mut_lamports_unchecked() += *fundraiser.borrow_lamports_unchecked();
    }
    fundraiser.close()?;

    Ok(())
}