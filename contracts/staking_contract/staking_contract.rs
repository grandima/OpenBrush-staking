

#[openbrush::contract]
mod staking_token {
    use openbrush::{
        contracts::psp22::extensions::metadata::*,
        traits::{
            Storage,
            String
        }
    };

    const DAY_SECONDS: u128 = 60 * 60 * 24;

    #[ink(storage)]
    #[derive(Storage)]
    pub struct StakingContract {
        token_addr: AccountId
    }

    impl StakingContract {
        #[ink(constructor)]
        pub fn new(staking_token: AccountId) -> Self {
            Self {token_addr: staking_token}
        }
        #[ink(message)]
        pub fn stake(&self, amount: AccountId) {

        }
    }
}