# Fundraiser Pinocchio

This program demonstrates how to create a fundraising platform users to raise funds in SPL Tokens.

In this program, a user will be able to create a fundraiser account, where he will be specify the mint he wants to collect and the fundraising target.

## Let's walk throught the program architecture:

A fundraising account consists of:

```
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