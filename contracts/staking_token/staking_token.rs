
#[openbrush::contract]
mod staking_token {
    use openbrush::{
        contracts::{
            ownable::*,
            psp22::extensions::{
                metadata::*, 
                mintable::*
            }
        },
        modifiers,
        traits::{
            Storage,
            String
        }
    };

    impl PSP22 for StakingToken {}
    
    impl PSP22Metadata for StakingToken {}

    impl Ownable for StakingToken {}

    #[ink(storage)]
    #[derive(Storage, Default)]
    pub struct StakingToken {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
        #[storage_field]
        ownable: ownable::Data,
    }

    impl StakingToken {
        #[ink(constructor)]
        pub fn new(name: Option<String>, symbol: Option<String>, staking_account: AccountId) -> Self {
            let mut contract = Self::default();
            contract.metadata.name = name;
            contract.metadata.symbol = symbol;
            let decimals: u8 = 18;
            contract.metadata.decimals = decimals;
            let initial_supply = 1_000_000_000 * (10 as u128).pow(decimals as u32);
            contract.psp22.supply = initial_supply;
            contract._init_with_owner(Self::env().caller());
            assert!(contract.mint(staking_account, initial_supply * 70 / 100).is_ok());
            assert!(contract.mint(Self::env().caller(), initial_supply * 30 / 100).is_ok());
            contract
        }
        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn mint_to_staking_acc(&mut self, account: AccountId) -> Result<(), PSP22Error> {
            let supply = (self.psp22.supply as Balance) * 70 / 100;
            self.mint(account, supply as Balance)
        }
    }

    impl PSP22Mintable for StakingToken {
        /// override the `mint` function to add the `only_owner` modifier
        #[ink(message)]
        #[modifiers(only_owner)]
        fn mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._mint_to(account, amount)
        }
    }
}