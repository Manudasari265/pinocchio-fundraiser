use crate::state::{Contribute, Fundraiser};
use crate::tests::connect;
use fundraiser_pinocchio::state::fundraiser;
use mollusk_svm::{program, result::Check};
use mollusk_token::token;
use solana_sdk::program_option::COption;
use solana_sdk::{
    account::{AccountSharedData, ReadableAccount, WritableAccount},
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    program_pack::Pack,
    pubkey::Pubkey,
};

#[test]
fn contribute_test() {
    let (program_id, mollusk) = connect();

    let (system_program, system_account) = mollusk_svm::program::keyed_account_for_system_program();

    mollusk.add_program(
        &spl_token::ID,
        "tests/elfs/spl_token-3.5.0",
        &mollusk_svm::program::loader_keys::LOADER_V3,
    );

    let (token_program, token_account) = (
        spl_token::ID,
        program::create_program_account_loader_v3(&spl_token::ID),
    );

    let contributor = Pubkey::new_from_array([0x01; 32]);
    let contributor_account = AccountSharedData::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

    let mint_to_raise = Pubkey::new_from_array([0x02; 32]);
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
    )
    .unwrap();

    let fundraiser = Pubkey::new_from_array([0x03; 32]);
    let mut fundraiser_acc = AccountSharedData::new(
        mollusk.sysvars.rent.minimum_balance(Fundraiser::LEN),
        Fundraiser::LEN,
        &program_id,
    );
    solana_sdk::program_pack::Pack::pack(
        Fundraiser {
            is_initialized: true,
            maker: contributor,
            mint_to_raise: mint_to_raise,
            amount_to_raise: 100_000_000,
            current_amount: 0,
            time_started: Clock::get().unwrap().unix_timestamp,
            duration: 7,
            bump: 255,
        },
        fundraiser_acc.data_as_mut_slice(),
    )
    .unwrap();

    let contributer_acc = Pubkey::new_unique();
    let contributor_ata_acc = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_sdk::program_pack::Pack::pack(
        spl_token::state::Account {
            mint,
            owner: contributor,
            amount: 100_000_000,
            delegate: COption::None,
            state: AccounState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        contributor_ata_acc.data_as_mut_slice(),
    )
    .unwrap();

    let vault = Pubkey::new_unique();
    let mut vault_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_sdk::program_pack::Pack::pack(
        spl_token::state::Account {
            mint,
            owner: fundraiser,
            amount: 0,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        vault_account.data_as_mut_slice(),
    )
    .unwrap();

    // prepare the instructon data
    let data: [Vec<u8>; 4] = [
        vec![1],
        1_000_000u64.to_le_bytes().to_vec(),
        vec![255],
        vec![255],
    ]
    .concat();

    let ix_accounts = vec![
        AccountMeta::new(contributor, true),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new(fundraiser, false),
        AccountMeta::new(contributor_acc, false),
        AccountMeta::new(contributor_ata, false),
        AccountMeta::new(vault, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let instruction = Instruction::new_with_bytes(program_id, &data, accounts);

    let tx_accounts = vec![
        (contributor, contributor_account),
        (mint, mint_account),
        (fundraiser, fundraiser_account),
        (contributor_acc, contributor_acc_account),
        (contributor_ata, contributor_ata_account),
        (vault, vault_account),
        (token_program, token_account),
        (system_program, system_account),
    ];

    let result =
        mollusk.process_and_validate_instruction(&instruction, &tx_accounts, &[Check::success()]);

}
