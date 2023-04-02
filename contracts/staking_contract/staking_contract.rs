#[openbrush::contract]
mod staking_contract {
    use ink::prelude::vec::Vec;
    use openbrush::{
        contracts::{
            psp22::extensions::metadata::*,
            traits::psp22::PSP22Ref,
            psp37::Internal,
            psp37::extensions::{mintable::*},
        },
        traits::{
            Storage
        },
        storage::Mapping
    };
    use ink::{
        storage::traits::StorageLayout,
    };
    const DAY_SECONDS: u64 = 60 * 60 * 24;
    const YEAR: u64 = 365;
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
    struct Record {
        pub staked_amount: Balance,
        pub initial_timestamp: Timestamp,
        pub staked_at: Timestamp,
        pub last_reputation_update: Timestamp
    }
    impl Default for Record {
        fn default() -> Self {
            Self {staked_amount: 0, initial_timestamp: 0, staked_at: 0, last_reputation_update: 0}
        }
    }
    impl Record {
        pub fn calculate_reputation(&self, block_timestamp: Timestamp) -> u128 {
            ((block_timestamp - self.last_reputation_update) / DAY_SECONDS) as u128 * self.staked_amount
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
        reputation_map: Mapping<AccountId, u128>,
    }

    impl StakingContract {
        #[ink(constructor)]
        pub fn new(token: AccountId, available_reward_amount: Balance) -> Self {
            Self {token_addr: token, stake_info: Default::default(), total_staked_amount: 0, available_reward_amount: available_reward_amount, initial_timestamp: Self::env().block_timestamp(), psp37: Default::default(), reputation_map: Default::default()}
        }
        #[ink(message)]
        pub fn stake(&mut self, amount: Balance) -> Result<(), Error> {
            if amount == 0 {
                return Err(Error::AmountToStakeIsZero);
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
            let record = Record {staked_amount: final_staked_amount, initial_timestamp: block_timestamp, staked_at:  block_timestamp, last_reputation_update: block_timestamp};
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
        //The computation is very strange here. But it may be adjusted.
        fn _calculate_reward(&self, block_timestamp: Timestamp, initial_timestamp: Timestamp, start_staking_timestamp: Timestamp, share: Balance) -> Balance {
            if initial_timestamp < block_timestamp {
                return 0;
            }
            let year = YEAR;
            let day = DAY_SECONDS;
            let era = (block_timestamp - initial_timestamp) / day / year;
            let era_start_timestamp = era * year * day;
            let mut start_staking_timestamp = start_staking_timestamp;
            if era_start_timestamp > start_staking_timestamp {
                start_staking_timestamp = era_start_timestamp;
            }
            let mut full_staking_years = era;
            let mut total_reward = share;
            while full_staking_years != 0 {
                total_reward /= 2;
                full_staking_years -= 1;
            }
            total_reward = (total_reward / 2) * (((block_timestamp - start_staking_timestamp) / day) / year) as u128;
            return total_reward;
        }


        pub fn claim_reputation(&mut self, account_id: AccountId) {
            let pending_rep = match self.stake_info.get(&account_id) {
                Some(mut record) => {
                    let block_timestamp = self.env().block_timestamp();
                    let new_rep = record.calculate_reputation(block_timestamp);
                    record.last_reputation_update = block_timestamp;
                    self.stake_info.insert(&account_id, &record);
                    new_rep
                },
                None => 0
            };
            let old_rep = self.reputation_map.get(&account_id).unwrap_or(0);
            let new_rep = old_rep + pending_rep;
            //ERRR
            if old_rep == new_rep  {
                return;
            }
            self.reputation_map.insert(&account_id, &new_rep);
            let mut rep_billions = new_rep / 1_000_000_000_u128;
            let mut level = 0;
            while rep_billions != 0 {
                rep_billions /= 10_u128;
                level += 1;
            }
            if self.balance_of(account_id, Some(Id::U128(level))) == 0 {
                let mut out_vec = Vec::new();
                out_vec.push((Id::U128(level), 1));
                //ERRR
                let _ = self._mint_to(account_id, out_vec);
            }
        }
    }

}