use mollusk_svm::{program, Mollusk};
use mollusk_svm::result::{Check, ProgramResult};
use solana_sdk::account_info::AccountInfo;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::program_option::COption;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
extern crate alloc;
use alloc::vec;


use fundraiser_pinocchio::instruction::{InitializeInstructionData, ContributeInstructionData};
use fundraiser_pinocchio::state::{Fundraiser, Contribute, to_bytes};
use solana_sdk::rent::Rent;
use solana_sdk::sysvar::Sysvar;

pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array(fundraiser_pinocchio::ID);
pub const RENT: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");

pub fn mollusk() -> Mollusk {
    let mut mollusk = Mollsuk::new(&PROGRAM_ID, "target/deploy/fundraiser_pinocchio");

    mollusk.add_program(
        &spl_token::ID,
        "tests/elfs/spl_token-3.5.0",
        &mollusk_svm::program::loader_keys::LOADER_V3
    );
    mollusk
}

pub trait AccountExt {
    fn refresh(
        &mut self,
        account_pubkey: &Pubkey,
        result: mollusk_svm::result::InstructionResult
    ) -> &mut Self;
}

impl AccountExt for Account {
    fn refresh(
        &mut self,
        account_pubkey: &Pubkey,
        result: mollusk_svm::result::InstructionResult
    ) -> &mut Self {
        *self = result.get_account(account_pubkey).unwrap().clone();
        self
    }
}

pub fn get_spl_token_program() -> (Pubkey, Account) {
    (spl_token::ID, program::create_program_account_loader_v3(&spl_token::ID))
}

pub fn get_rent_data() -> Vec<u8> {
    let rent = Rent::default();
    unsafe {
        core::slice::from_raw_parts(
            &rent as *const Rent as *const u8,
            Rent::size_of()
        ).tovec()
    }
}

//? Setup common fundraiser accounts
pub fn setup_fundraiser(mollusk: &Mollusk) -> (
    // Pubkeys
    Pubkey,
    Pubkey,
    Pubkey,
    u8,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    // Accounts
    Account,
    Account,
    Account,
    Account,
    Account,
    Account,
    Account,
) {
    //? Setup system and token programs
    let (system_program, system_account) = program::keyed_account_for_system_program();
    let (token_program, token_account) = get_spl_token_program();

    //? Setup maker and contributor accounts
    let maker = Pubkey::new_from_array([0x01; 32]);
    let maker_account = Account::new(2 * LAMPORTS_PER_SOL, 0, &system_program);
    let contributor = Pubkey::new_unique();
    let contributor_account = Account::new(2 * LAMPORTS_PER_SOL, 0, &system_program);

    //? Derive required PDAs
    let (fundraiser, fundraiser_bump) = Pubkey::find_program_address(
        &[
            Fundraiser::SEED.as_bytes(), 
            &maker.to_bytes(),
            &PROGRAM_ID,
        ]
    );

    //? create empty fundraiser account (will be init later)
    let fundraiser_account = Account::new(0, 0, &system_program);;

    //? create the mint account
    let mint_to_raise = Pubkey::new_from_array([0x03; 32]);
    let mut mint_to_raise_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::Len,
        &token_program,
    );
    solana_sdk::program_pack::Pack::pack(
        spl_token::state::Mint {
            decimals: 6,
            supply: 100_000,
            is_initialized: true,
            freeze_authority: COption::None,
            mint_authority: COption::None,
        }
    )
    .unwrap();

    //? create vault account
    let vault = Pubkey::new_from_array([0x04; 32]);
    let vault_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::Len,
        &token_program,
    );
    solana_sdk::program_pack::Pack::pack(
        spl_token::state::Account {
            amount: 0,
            mint: mint_to_raise,
            owner: fundraiser,
            delegate: COption::None,
            state: spl_token::state::AccounState::Initialized,
            close_authority: COption::None,
            is_native: COption::None,
            delegated_amount: 0,
        },
        vault_account.data_as_mut_slice()
    )
    .unwrap();

    (
        //? Return Pubkeys
        maker,
        contributor,
        fundraiser,
        fundraiser_bump,
        mint_to_raise,
        vault,
        system_program,
        token_program,

        //? Return Accounts
        maker_account,
        contributor_account,
        fundraiser_account,
        mint_to_raise_account,
        vault_account,
        system_account,
        token_account,
    )
}

pub fn execute_initialize(
    mollusk: &Mollusk,
    maker: Pubkey,
    maker_account: Account,
    mint_to_raise: Pubkey,
    mint_to_raise_account: Account,
    fundraiser: Pubkey,
    fundraiser_account: Account,
    vault: Pubkey,
    vault_account: Account,
    fundraiser_bump: u8,
    system_program: Pubkey,
    system_account: Account,
    token_program: Pubkey,
    token_account: Account,
    amount: u64,
    duration: u8,
) -> mollusk_svm::result::InstructionResult {
    //? create the instruction accounts
    let ix_accounts = vec![
        AccountMeta::new(maker, true),
        AccountMeta::new(mint_to_raise, false),
        AccountMeta::new(fundraiser, true),
        AccountMeta::new(vault, false),
        AccountMeta::new_readonly(system_program, false),
        AccountMeta::new_readonly(token_program, false),
    ];

    //? create the instruction data
    let ix_data = InitializeInstructionData {
        amount,
        duration,
        bump: fundraiser_bump,
    };

    //? serialize the instruction data
    let mut ser_ix_data = vec![0]; //* set the Ix discriminator = 0
    ser_ix_data.extend_from_slice(
        unsafe {
            to_bytes(&ix_data)
        }
    );

    //? create the instruction
    let instruction = Instruction::new_with_bytes(
        PROGRAM_ID,
        &ser_ix_data.
        ix_accounts,
    );

    //? create transaction accounts
    let tx_accounts = &vec![
        (maker, maker_account),
        (mint_to_raise, mint_to_raise_account),
        (fundraiser, fundraiser_account),
        (vault, vault_account),
        (system_program, system_account),
        (token_program, token_account),
    ];

    //? process the instruction
    let result = mollusk.process_and_validate_instruction(
        &instruction,
        tx_accounts,
        mollusk.sysvars.clone(),
    );
    result
}

