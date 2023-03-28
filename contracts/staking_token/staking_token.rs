

#[openbrush::contract]
mod staking_token {
    use openbrush::{
        contracts::psp22::extensions::metadata::*,
        traits::{
            Storage,
            String
        }
    };

    impl PSP22 for StakingToken {}
    
    impl PSP22Metadata for StakingToken {}

    #[ink(storage)]
    #[derive(Storage, Default)]
    pub struct StakingToken {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data
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
            assert!(contract._mint_to(staking_account, initial_supply * 70 / 100).is_ok());
            assert!(contract._mint_to(Self::env().caller(), initial_supply * 30 / 100).is_ok());
            contract
        }
    }
}