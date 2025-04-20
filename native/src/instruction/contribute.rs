use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    sysvars::{rent::Rent, clock::Clock, Sysvar},
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{
    instructions::TransferChecked,
    state::{Mint, TokenAccount},
};
use crate::{
    consts::{
        MAX_CONTRIBUTION_PERCENTAGE,
        PERCENTAGE_SCALER,
        SECONDS_TO_DAYS
    }, error::FundraiserError, 
    state::utils::{
        load_acc_mut, load_acc_mut_unchecked, load_ix_data, DataLen
    },
    state::{Fundraiser, Contribute},
};

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContributeInstructionData {
    pub amount: u64,
    pub fundraiser_bump: u8,
    pub contributor_bump: u8,
}

impl DataLen for ContributeInstructionData {
    const LEN: usize = core::mem::size_of::<ContributeInstructionData>();
}


pub fn process_contribute(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [contributor, mint_to_raise, fundraiser, contributor_acc, contributor_ata, vault, _token_program, _system_program] = accounts
      else {
        return Err(ProgramError::InvalidAccountData);
    };

    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    {
        //? Some checks for authorities
        let vault_acc = TokenAccount::from_account_info(vault)?;
        //? The vault should be intialised on client side to save CUs
        assert_eq!(vault_acc.owner(), fundraiser.key());
        let contributor_ata_acc = TokenAccount::from_account_info(contributor_ata)?;
        assert_eq!(contributor_ata_acc.owner(), contributor.key());
    }

    let ix_data = unsafe {
        load_ix_data::<ContributeInstructionData>(instruction_data)?
    };

    if contributor_acc.data_is_empty() || !contributor_acc.is_owned_by(&crate::ID) {
        let rent = Rent::get()?;
        let bump_seed = [ix_data.contributor_bump];
        let contributor_signer_seeds = [
            Seed::from(Contribute::SEED.as_bytes()),
            Seed::from(fundraiser.key().as_ref()),
            Seed::from(contributor.key().as_ref()),
            Seed::from(&bump_seed[..]),
        ];
        let contributor_signer = Signer::from(&contributor_signer_seeds[..]);
        (CreateAccount {
            from: &contributor.clone(),
            to: contributor_acc,
            lamports: rent.minimum_balance(Contribute::LEN),
            space: Contribute::LEN as u64,
            owner: &crate::ID,
        })
        .invoke_signed(&[contributor_signer])?;
        let contributor_state = (unsafe {
            load_acc_mut_unchecked::<Contribute>(contributor_acc.borrow_mut_data_unchecked())
        })?;
        contributor_state.initialize(ix_data.amount);
    }

    let mint_state = Mint::from_account_info(mint_to_raise)?;
    let decimals = mint_state.decimals();

    let fundraiser_state = unsafe {
        load_acc_mut::<Fundraiser>(fundraiser.borrow_mut_data_unchecked())?
    };
    let contributor_state = unsafe {
        load_acc_mut::<Contribute>(contributor.borrow_mut_data_unchecked())?
    };

    //? check if the contribution amount is < maximum allowed contribution
    if ix_data.amount > (fundraiser_state.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER {
        return Err(FundraiserError::ContributionTooBig.into());
    }

    //? validate if the contribution duration has been reached oir not
    if 
       contributor_state.amount > 
       (fundraiser_state.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER 
       &&
       contributor_state.amount + ix_data.amount >
       (fundraiser_state.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER 
    {
        return Err(FundraiserError::MaximumContributionsReached.into());
    }

    //? check if the contribution duration has been reached
    let current_time = Clock::get()?.unix_timestamp;
    if 
       fundraiser_state.duration < 
       (((current_time - fundraiser_state.time_started) / SECONDS_TO_DAYS) as u8)
    {
        return Err(FundraiserError::FundraiserEnded.into())
    }

    //? check if the maximum contribution per contributor have been reached
    if 
       contributor_state.amount > 
       (fundraiser_state.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER
       &&
       contributor_state.amount + ix_data.amount >
       (fundraiser_state.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER
    { 
        return Err(FundraiserError::MaximumContributionsReached.into());
    }

    (TransferChecked {
        from: contributor_ata,
        to: vault,
        authority: contributor,
        mint: mint_to_raise,
        amount: ix_data.amount,
        decimals,
    }).invoke()?;

    //? update the final states
    contributor_state.amount += ix_data.amount;
    fundraiser_state.current_amount += ix_data.amount;

    Ok(())
}