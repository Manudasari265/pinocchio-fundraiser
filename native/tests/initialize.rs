use crate::tests::connect;
use mollusk_svm::{
    program,
    result::Check,
};
use solana_sdk::{
    account::{AccountSharedData, WritableAccount}, instruction::{AccountMeta, Instruction}, native_token::LAMPORTS_PER_SOL, program_option::COption, program_pack::Pack, pubkey::Pubkey, system_program
};
use spl_token::state::AccountState;


#[test]
fn initialize() {
    let (program_id, mollusk) = connect();

    let (sytem_program, system_account) = 
        mollusk_svm::program::keyed_account_for_system_program();

    mollusk.add_program(
        &spl_token::ID,
        "tests/elfs/spl_token-3.5.0",
        &mollusk_svm::program::loader_keys::LOADER_V3,
    );

    let (token_program, token_account) = (
        spl_token::ID,
        program::create_program_account_loader_v3(&spl_token::ID),
    );

    // Accounts
    let maker = Pubkey::new_from_array([0x01; 32]);
    let maker_account = AccountSharedData::new(
        1 * LAMPORTS_PER_SOL,
        0,
        &system_program,
    );

    let (fundraiser, _bump) = Pubkey::find_program_address(
        &[b"fundraiser", 
        &maker.to_bytes()], 
        &program_id
    );
    
    let fundraiser_acc = AccountSharedData::new(
        0,
        0,
        &system_program,
    );

    let mint_to_raise = Pubkey::new_from_array([0x2; 32]);
    let mut mint_to_raise_acc = AccountSharedData::new(
        mollusk
                    .sysvars
                    .rent
                    .minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN,
                &token_program,
    );
    solana_sdk::program_pack::Pack::pack(
        spl_token::state::Mint {
            mint_authority: COption::None,
            supply: 100_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_to_raise_acc.data_as_mut_slice(),
    ).unwrap();

    let vault = Pubkey::new_from_array([0x3; 32]);

    let mut vault_acc = AccountSharedData::new(
        mollusk
                    .sysvars
                    .rent
                    .minimum_balance(spl_token::state::Account::LEN),
                spl_token::state::Account::LEN,
                &token_program,
    );
    solana_sdk::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_to_raise,
            owner: fundraiser,
            amount: 0,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        vault_acc.data_as_mut_slice(),
    )
    .unwrap();

    let ix_data = [
        vec![0],
        100_000_000u64.to_le_bytes().to_vec(),
        vec![30],
        vec![1],
    ]
    .concat();

    let ix_accounts = vec![
        AccountMeta::new(maker, true),
        AccountMeta::new(mint_to_raise, false),
        AccountMeta::new(fundraiser, false),
        AccountMeta::new(vault, false),
        AccountMeta::new_readonly(system_program, false),
        AccountMeta::new_readonly(token_program, false),
    ];

    let instruction = Instruction::new_with_bytes(
        program_id, 
        &ix_data, 
        ix_accounts
    );

    let tx_accounts = &vec![
        (maker, maker_account.clone()),
        (mint_to_raise, mint_to_raise_acc.clone()),
        (fundraiser, fundraiser_acc.clone()),
        (vault, vault_acc.clone()),
        (system_program, system_account.clone()),
        (token_program, token_account.clone()),
    ];

    let init_res = mollusk.process_and_validate_instruction(
        &instruction,
        tx_accounts,
        &[
            Check::success()
        ]
    );
}