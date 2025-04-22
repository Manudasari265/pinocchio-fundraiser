# Fundraiser Pinocchio

This program demonstrates how to create a fundraising platform users to raise funds in SPL Tokens.

In this program, a user will be able to create a fundraiser account, where he will be specify the mint he wants to collect and the fundraising target.

## Let's walk throught the program architecture:

A fundraising account consists of:

```rust
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fundraiser {
    pub is_initialized: bool,
    pub maker: Pubkey,
    pub mint_to_raise: Pubkey,
    pub amount_to_raise: u64,
    pub current_amount: u64,
    pub time_started: i64,
    pub duration: u8,
    pub bump: u8,
}
```

In this state account, we will store:
maker: the person who is starting the fundraising

mint_to_raise: the mint that the maker wants to receive

amount_to_raise: the target amount that the maker is trying to raise

current_amount: the total amount currently donated

time_started: the time when the account was created

duration: the timeframe to collect all the contributions (in days)

bump: since our Fundraiser account will be a PDA (Program Derived Address), we will store the bump of the account


Let´s have a closer look at the accounts that we are passing in this context:

maker: will be the person starting the fundraising. He will be a signer of the transaction, and we mark his account as mutable as we will be deducting lamports from this account

mint_to_raise: The mint that the user wants to receive. This will be a Mint Account, that we will use to store the mint address

fundraiser: will be the state account that we will initialize and the maker will be paying for the initialization of the account. We derive the Fundraiser PDA from the byte representation of the word "fundraiser" and the reference of the maker publick key. Anchor will calculate the canonical bump (the first bump that throes that address out of the ed25519 eliptic curve) and save it for us in a struct

vault: We will initialize a vault (ATA) to receive the contributions. This account will be derived from the mint that the user wants to receive, and the fundraiser account that we are just creating

system_program: Program resposible for the initialization of any new account

token_program and associated_token_program: We are creating new ATAs

A Contribute state account looks something like:

```rust
#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Contribute {
    pub is_initialized: bool,
    pub amount: u64,
}
```
In this state account, we will store:

amount: the target amount that the contributor is trying to contribute

