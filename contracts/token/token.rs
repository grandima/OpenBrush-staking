
#[openbrush::contract]
mod token {
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
            String as SString,
        }
    };
    #[ink(storage)]
    #[derive(Storage, Default)]
    pub struct Token {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
        #[storage_field]
        ownable: ownable::Data,
    }

    impl PSP22 for Token {}
    
    impl PSP22Metadata for Token {}

    impl Ownable for Token {}
    
    const INITIAL_SUPPLY: u128 = 1_000_000_000 * 10u128.pow(18);

    const STAKING_PERCENTAGE: u128 = 70;

    impl Token {
        #[ink(constructor)]
        pub fn new(name: Option<SString>, symbol: Option<SString>) -> Self {
            let mut contract = Self::default();
            contract.metadata.name = name;
            contract.metadata.symbol = symbol;
            let decimals: u8 = 18;
            contract.metadata.decimals = decimals;
            let tokens_to_owner = INITIAL_SUPPLY - INITIAL_SUPPLY * STAKING_PERCENTAGE / 100;
            contract._init_with_owner(Self::env().caller());
            assert!(contract
                ._mint_to(Self::env().caller(), tokens_to_owner).is_ok(),
                "Failed to mint tokens to admin"
            );
            contract
        }
        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn mint_to_staking_acc(&mut self, account: AccountId) -> Result<(), PSP22Error> {
            let supply = 1_000_000_000_u128 * 70 / 100;
            self.mint(account, supply as Balance)
        }
    }

    impl PSP22Mintable for Token {
        /// override the `mint` function to add the `only_owner` modifier
        #[ink(message)]
        #[modifiers(only_owner)]
        fn mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._mint_to(account, amount)
        }
    }


    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;
        use openbrush::contracts::psp22::{
            extensions::metadata::psp22metadata_external::PSP22Metadata, psp22_external::PSP22,
        };

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn instantiation_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = TokenRef::new(
                Some(SString::from("OpenBrushToken")),
                Some(SString::from("OBT"))
            );

            // When
            let contract_account_id = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Check Token Name
            let token_name = build_message::<TokenRef>(contract_account_id.clone())
                .call(|token| token.token_name());
            assert_eq!(
                client
                    .call_dry_run(&ink_e2e::alice(), &token_name, 0, None)
                    .await
                    .return_value(),
                Some(SString::from("OpenBrushToken"))
            );

            // // Check Token Symbol
            // let token_symbol = build_message::<TokenRef>(contract_account_id.clone())
            //     .call(|token| token.token_symbol());
            // assert_eq!(
            //     client
            //         .call_dry_run(&ink_e2e::alice(), &token_symbol, 0, None)
            //         .await
            //         .return_value(),
            //     Some(OBString::from("OBT"))
            // );

            // // Check Token Decimals
            // let token_decimals = build_message::<TokenRef>(contract_account_id.clone())
            //     .call(|token| token.token_decimals());
            // assert_eq!(
            //     client
            //         .call_dry_run(&ink_e2e::alice(), &token_decimals, 0, None)
            //         .await
            //         .return_value(),
            //     18
            // );

            // // Check Total Supply
            // let total_supply = build_message::<TokenRef>(contract_account_id.clone())
            //     .call(|token| token.total_supply());
            // assert_eq!(
            //     client
            //         .call_dry_run(&ink_e2e::alice(), &total_supply, 0, None)
            //         .await
            //         .return_value(),
            //         INITIAL_SUPPLY - INITIAL_SUPPLY * STAKING_PERCENTAGE / 100
            // );

            // // Check Balance of Contract Owner (Alice)
            // let alice_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Alice);
            // let alice_balance = build_message::<TokenRef>(contract_account_id.clone())
            //     .call(|token| token.balance_of(alice_account));
            // assert_eq!(
            //     client
            //         .call_dry_run(&ink_e2e::bob(), &alice_balance, 0, None)
            //         .await
            //         .return_value(),
            //     INITIAL_SUPPLY - (INITIAL_SUPPLY * STAKING_ALLOCATION / 100)
            // );
            Ok(())
        }
    }
}