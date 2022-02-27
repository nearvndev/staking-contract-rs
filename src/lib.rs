use near_sdk::collections::LookupMap;
use near_sdk::{near_bindgen, AccountId, env, PanicOnDefault, Balance, EpochHeight, BlockHeight, BorshStorageKey, Promise, PromiseResult, PromiseOrValue, ext_contract};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::{U128};

use crate::account::*;
use crate::util::*;

mod account;
mod util;
mod internal;
mod core_impl;
mod enumeration;

pub const NO_DEPOSIT: Balance = 0;
pub const NUM_EPOCHS_TO_UNLOCK: EpochHeight = 1;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    // Percent reward per 1 block
    pub reward_numerator: u32,
    pub reward_denumerator: u64,
    pub total_apr: u32
}

impl Default for Config {
    fn default() -> Self {
        // By default APR 15%
        Self { reward_numerator: 715, reward_denumerator: 100000000000, total_apr: 15 }
    }
}

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    AccountKey
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[near_bindgen]
pub struct StakingContract {
    pub owner_id: AccountId, // Owner of contract
    pub ft_contract_id: AccountId,
    pub config: Config, // Config reward and apr for contract
    pub total_stake_balance: Balance, // Total token balance lock in contract
    pub pre_reward: Balance,
    pub last_block_balance_change: BlockHeight,
    pub accounts: LookupMap<AccountId, Account>, // List staking user
    pub paused: bool, // Pause staking pool with limit reward,
    pub paused_in_block: BlockHeight
}

#[near_bindgen]
impl StakingContract {

    #[init]
    pub fn new_default_config(owner_id: AccountId, ft_contract_id: AccountId) -> Self {
        Self::new(owner_id, ft_contract_id, Config::default())
    }

    #[init]
    pub fn new(owner_id: AccountId, ft_contract_id: AccountId, config: Config) -> Self {
        StakingContract {
            owner_id,
            ft_contract_id,
            config,
            total_stake_balance: 0,
            pre_reward: 0,
            last_block_balance_change: env::block_index(),
            accounts: LookupMap::new(StorageKey::AccountKey),
            paused: false,
            paused_in_block: 0
        }
    }

    pub fn get_total_pending_reward(&self) -> U128 {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "ERR_ONLY_OWNER_CONTRACT");
        U128(self.pre_reward + self.internal_calculate_global_reward())
    }

    #[payable]
    pub fn deposit_storage(&mut self, account_id: Option<AccountId>) {
        assert_at_least_one_yocto();
        let account = account_id.unwrap_or_else(|| env::predecessor_account_id());

        let account_stake: Option<Account> = self.accounts.get(&account);
        if account_stake.is_some() {
            refund_deposit(0);
        } else {
            let before_storage_usage = env::storage_usage();

            let new_account = Account {
                stake_balance: 0,
                pre_stake_balance: 0,
                pre_reward: 0,
                last_block_balance_change: env::block_index(),
                unstake_balance: 0,
                unstake_available_epoch_height: 0,
                unstake_start_timestamp: 0
            };

            self.accounts.insert(&account, &new_account);
            let after_storage_usage = env::storage_usage();

            refund_deposit(before_storage_usage - after_storage_usage);
        }

    }

    pub(crate) fn assert_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only owner contract can be access");
    }
}