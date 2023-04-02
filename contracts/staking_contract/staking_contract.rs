#[openbrush::contract]
mod staking_contract {
    use ink::prelude::vec::Vec;
    use openbrush::{
        contracts::{
            psp22::extensions::metadata::*,
            traits::psp22::PSP22Ref,
            psp37::extensions::{burnable::*, mintable::*},
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
    }
    impl Default for Record {
        fn default() -> Self {
            Self {staked_amount: 0, staked_at: 0}
        }
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NoRecord(AccountId),
        AmountToStakeIsZero,
        AllowanceLowerThanAmount,
        StakeOverflow,
        PSP22Error(PSP22Error)
    }

    impl PSP37 for StakingContract {}

    #[ink(storage)]
    #[derive(Storage)]
    pub struct StakingContract {
        token_addr: AccountId,
        stake_info: Mapping<AccountId, Record>,
        total_staked_amount: Balance,
        //for simplicity it's 700_000_000
        available_reward_amount: Balance,
        initial_timestamp: Timestamp,
        #[storage_field]
        psp37: psp37::Data,
    }

    impl StakingContract {
        #[ink(constructor)]
        pub fn new(token: AccountId, available_reward_amount: Balance) -> Self {
            Self {token_addr: token, stake_info: Default::default(), total_staked_amount: 0, available_reward_amount: available_reward_amount, initial_timestamp: Self::env().block_timestamp(), psp37: Default::default()}
        }
        #[ink(message)]
        pub fn stake(&mut self, amount: Balance) -> Result<(), Error> {
            if amount == 0 {
                return Err(Error::AmountToStakeIsZero);
            }
            if self._can_stake(amount) {

            }
            let token = self.token_addr;
            let from = self.env().caller();
            let to = self.env().account_id();
            let allowance = PSP22Ref::allowance(&token, from, to);
            if allowance < amount {
                return Err(Error::AllowanceLowerThanAmount);
            }
            let before_contract_balance = PSP22Ref::balance_of(&token, to);
            //ERRR
            let _ = PSP22Ref::transfer_from(&self.token_addr, self.env().caller(), self.env().account_id(), amount, Vec::<u8>::new());
            let after_contract_balance = PSP22Ref::balance_of(&token, to);
            let mut final_staked_amount: Balance = 0;
            match after_contract_balance.checked_sub(before_contract_balance) {
                Some(result) => { final_staked_amount = result; }
                None => { return Err(Error::StakeOverflow);}
            };
            self.total_staked_amount += final_staked_amount;
            let block_timestamp = self.env().block_timestamp();
            let record = Record {staked_amount: final_staked_amount, staked_at:  block_timestamp};
            self.stake_info.insert(&from, &record);
            return Ok(());
        }

        #[ink(message)]
        pub fn unstake(&mut self) -> Result<(), Error> {
            let _ = self.claim_reward()?;
            
            let from = self.env().caller();
            let Some(record) = self.stake_info.get(&from) else {
                return Err(Error::NoRecord(from))
            };
            let staked_amount = record.staked_amount;
            //ERRR
            let _ = PSP22Ref::transfer_from(&self.token_addr, self.env().account_id(), self.env().caller(), staked_amount, Vec::<u8>::new());
            self.total_staked_amount -= staked_amount;
            self.stake_info.remove(&from);
            Ok(())
        }
        
        #[ink(message)]
        pub fn claim_reward(&mut self) -> Result<(), Error> {
            let account_id = self.env().caller();
            let Some(mut record) = self.stake_info.get(&account_id) else {
                return Err(Error::NoRecord(account_id))
            };
            let block_timestamp = self.env().block_timestamp();
            let start_staking_timestamp = record.staked_at;
            let initial_timestamp = self.initial_timestamp;
            let reward_amount = self._calculate_reward(block_timestamp, initial_timestamp, start_staking_timestamp, self.available_reward_amount / self.total_staked_amount * record.staked_amount);
            if reward_amount == 0 {
                return Ok(());
            }
            //ERRR: check transfer error
            let _ = PSP22Ref::transfer_from(&self.token_addr, self.env().account_id(), self.env().caller(), reward_amount, Vec::<u8>::new());
            record.staked_at = block_timestamp;
            self.stake_info.insert(&account_id, &record);
            Ok(())
        }

        fn _calculate_reward(&self, block_timestamp: Timestamp, initial_timestamp: Timestamp, start_staking_timestamp: Timestamp, share: Balance) -> Balance {
            if initial_timestamp < block_timestamp {
                return 0;
            }
            let year = 365;
            let day = 24 * 60 * 60;
            let era = (block_timestamp - initial_timestamp) / day / year;
            let era_start_timestamp = era * year * day;
            let mut start_staking_timestamp = start_staking_timestamp;
            if era_start_timestamp > start_staking_timestamp {
                start_staking_timestamp = era_start_timestamp;
            }
            let staking_days = (block_timestamp - era_start_timestamp) / day;
            let mut full_staking_years = era;
            let left_staking_days = staking_days - full_staking_years * day;
            let mut total_reward = share;
            while full_staking_years != 0 {
                total_reward /= 2;
                full_staking_years -= 1;
            }
            total_reward = (total_reward / 2) * (((block_timestamp - start_staking_timestamp) / day) / year) as u128;
            return total_reward;
        }

        fn _get_record(&self, account_id: AccountId) -> Record {
            self.stake_info.get(&account_id).unwrap_or(Record::default())
        }
        fn _can_stake(&self, amount: Balance) -> bool {
            self.available_reward_amount >= amount
        }
    }

}