

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
    #[derive(Storage, Default)]
    pub struct StakingContract {
        staking_token: AccountId,
        mapping: Mapping<AccountId, ()>
    }

    impl StakingContract {
        #[ink(constructor)]
        pub fn new(staking_token: AccountId) -> Self {
            Self {staking_token: staking_token}
        }
        #[ink(message)]
        pub fn stake(&self, amount: Balance) {

        }
    }
}