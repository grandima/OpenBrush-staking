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
        pub share_amount: Balance,
        pub staked_at: Timestamp,
        pub last_claimed: Timestamp,
    }
    impl Default for Record {
        fn default() -> Self {
            Self {staked_amount: 0, share_amount: 0, staked_at: 0, last_claimed: 0}
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

    #[ink(storage)]
    #[derive(Storage)]
    pub struct StakingContract {
        token_addr: AccountId,
        stake_info: Mapping<AccountId, Record>,
        total_staked_amount: Balance,
        available_reward_amount: Balance
    }

    impl StakingContract {
        #[ink(constructor)]
        pub fn new(token: AccountId, available_reward_amount: Balance) -> Self {
            Self {token_addr: token, stake_info: Default::default(), total_staked_amount: 0, available_reward_amount: available_reward_amount}
        }
        #[ink(message)]
        pub fn stake(&mut self, amount: Balance) -> Result<(), Error> {
            if amount == 0 {
                return Err(Error::AmountToStakeIsZero);
            }
            if self._can_stake() {

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
            let record = Record {staked_amount: final_staked_amount, share_amount: 0, staked_at:  block_timestamp, last_claimed: 0};
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
            let staked_at = record.staked_at;
            let reward_amount = self._calculate_reward(block_timestamp, staked_at, record.staked_amount) - self._calculate_reward(block_timestamp, record.last_claimed, record.staked_amount);
            if reward_amount == 0 {
                return Ok(());
            }

            //ERRR: check transfer error
            let _ = PSP22Ref::transfer_from(&self.token_addr, self.env().account_id(), self.env().caller(), reward_amount, Vec::<u8>::new());
            self.available_reward_amount -= reward_amount;
            record.last_claimed = block_timestamp;
            self.stake_info.insert(&account_id, &record);
            Ok(())
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

        fn _get_record(&self, account_id: AccountId) -> Record {
            self.stake_info.get(&account_id).unwrap_or(Record::default())
        }
        fn _can_stake(&self, amount: Balance) -> bool {
            self.available_reward_amount >= amount
        }
    }
    // impl Into<Error> for PSP22Error {
    //     fn into(self) -> Error {
    //         Error::PSP22Error(self)
    //     }
    // }
}