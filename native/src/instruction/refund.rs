use pinocchio::{
    account_info::AccountInfo, 
    instruction::{Seed, Signer},
    program_error::ProgramError, 
    sysvars::{clock::Clock, Sysvar}, 
    ProgramResult
};
use pinocchio_token::{instructions::TransferChecked, state::{Mint, TokenAccount}};

use crate::{consts::SECONDS_TO_DAYS, error::FundraiserError, state::{fundraiser, load_acc_mut, Contribute, Fundraiser}};

pub fn process_refund(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [contributor, maker, mint_to_raise, fundraiser, contributor_acc, contributor_ata, vault, _token_program, _system_program, _remaining @ ..] = accounts 
      else {
        return Err(ProgramError::MissingRequiredSignature)
    };

    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    //? Some checks for authorities
    let vault_acc = TokenAccount::from_account_info(vault)?;
    //? The vault should be intialised on client side to save CUs
    assert_eq!(vault_acc.owner(), fundraiser.key());
    //? assert_eq!(contributor_ata_acc.owner(), contributor.key());
    //? Some checks for authorities
    //? Check if the fundraiser is initialized
    let fundraiser_state = unsafe {
        load_acc_mut::<Fundraiser>(fundraiser.borrow_mut_data_unchecked())?
    };

    let contributor_state = unsafe {
        load_acc_mut::<Contribute>(contributor_acc.borrow_mut_data_unchecked())?
    };

    //? check if the fundraising duration has been reached
    let current_time = Clock::get()?.unix_timestamp;
    if 
       fundraiser_state.duration >
       (((current_time - fundraiser_state.time_started) / SECONDS_TO_DAYS) as u8)
    {
        return Err(FundraiserError::FundraiserNotEnded.into());
    }

    //? check if the vault account is less than the amount to raise in fundraiser
    if vault_acc.amount() >= fundraiser_state.amount_to_raise {
        return Err(FundraiserError::TargetMet.into());
    }

    //? transfer the funds back to the contributor
    let mint_state = Mint::from_account_info(mint_to_raise)?;
    let bump_seed = [fundraiser_state.bump];
    let fundraiser_seeds = [
        Seed::from(Fundraiser::SEED.as_bytes()),
        Seed::from(maker.key().as_ref()),
        Seed::from(&bump_seed[..])
    ];

    let fundraiser_signer = Signer::from(&fundraiser_seeds[..]);

    (TransferChecked {
        amount: contributor_state.amount,
        from: vault,
        to: contributor_ata,
        authority: fundraiser,
        mint: mint_to_raise,
        decimals: mint_state.decimals(),
    })
    .invoke_signed(&[fundraiser_signer.clone()])?;

    //? close the contributor account
    unsafe {
        *contributor.borrow_mut_lamports_unchecked() +=
            *contributor_acc.borrow_mut_lamports_unchecked();
    }

    contributor_acc.close()?;

    
    Ok(())
}