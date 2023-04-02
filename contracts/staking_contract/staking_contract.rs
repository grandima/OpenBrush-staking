#[openbrush::contract]
mod staking_contract {
    use ink::prelude::vec::Vec;
    use openbrush::{
        contracts::{
            psp22::extensions::metadata::*,
            traits::psp22::PSP22Ref
        },
        traits::{
            Storage
        },
        storage::Mapping
    };

    use ink::storage::traits::StorageLayout;
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    struct Record {
        pub staked_amount: Balance,
        pub staked_at: Timestamp,
        pub last_claimed: Timestamp,
    }
    impl Default for Record {
        fn default() -> Self {
            Self {staked_amount: 0, staked_at: 0, last_claimed: 0}
        }
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        Nothing,
    }

    #[ink(storage)]
    #[derive(Storage)]
    pub struct StakingContract {
        token_addr: AccountId,
        stake_info: Mapping<AccountId, Record>,
        available_amount: Balance 
    }

    impl StakingContract {
        #[ink(constructor)]
        pub fn new(token: AccountId, available_amount: Balance) -> Self {
            Self {token_addr: token, stake_info: Default::default(), available_amount}
        }
        #[ink(message)]
        pub fn stake(&self, amount: Balance) -> Result<(), PSP22Error> {        
            PSP22Ref::transfer_from(&self.token_addr, self.env().caller(), self.env().account_id(), amount, Vec::<u8>::new())
        }
        fn _get_record(&self, account_id: AccountId) -> Record {
            self.stake_info.get(&account_id).unwrap_or(Record::default())
        }
        fn _calculate_share(&self, record: Record) -> Balance {
            record.staked_amount / self.available_amount * record.staked_amount
        }

        #[ink(message)]
        pub fn claim_reward(&mut self) {
            let account_id = self.env().caller();
            let Some(mut record) = self.stake_info.get(&account_id) else {
                return;
            };
            let block_timestamp = self.env().block_timestamp();
            let staked_at = record.staked_at;
            let amount = self._calculate_reward(block_timestamp, staked_at, record.staked_amount) - self._calculate_reward(block_timestamp, record.last_claimed, record.staked_amount);
            if amount <= 0 {
                return;
            }
            let _ = PSP22Ref::transfer_from(&self.token_addr, self.env().account_id(), self.env().caller(), amount, Vec::<u8>::new());
            record.last_claimed = block_timestamp;
            self.stake_info.insert(&account_id, &record);
        }

        fn _calculate_reward(&self, block_timestamp: Timestamp, staked_at: Timestamp, share: Balance) -> Balance {
            if staked_at < block_timestamp {
                return 0;
            }
            let year: u128 = 365;
            let day = 24 * 60 * 60;
            let staking_days = (block_timestamp - staked_at) as u128 / (24 * 60 * 60);
            let mut full_staking_years = staking_days / year;
            let left_staking_days = staking_days - full_staking_years * day;
            let full_share = share;
            let mut total_reward = full_share;
            while full_staking_years != 0 {
                total_reward /= 2;
                full_staking_years -= 1;
            }
            let days_share = (total_reward / 2) * left_staking_days / year;
            total_reward = full_share - total_reward + days_share;
            return total_reward;
        }
    }
}