use crate::*;

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolInfo {
    pub total_stake_balance: U128,
    pub total_reward: U128,
    pub total_stakers: U128,
    pub is_paused: bool
}

#[near_bindgen]
impl StakingContract {
    /**
     * Get current reward by account_id
     */
    pub fn get_account_reward(&self, account_id: AccountId) -> Balance {
        let upgradable_account: UpgradableAccount = self.accounts.get(&account_id).unwrap();
        let account: Account = Account::from(upgradable_account);
        let new_reward = self.internal_calculate_account_reward(&account);

        account.pre_reward + new_reward
    }

    pub fn get_account_info(&self, account_id: AccountId) -> AccountJson {
        let upgradable_account: UpgradableAccount = self.accounts.get(&account_id).unwrap();
        let account: Account = Account::from(upgradable_account);
        let new_reward = self.internal_calculate_account_reward(&account);

        AccountJson { 
            account_id: account_id, 
            stake_balance: U128(account.stake_balance), 
            unstake_balance: U128(account.unstake_balance), 
            reward: U128(account.pre_reward + new_reward), 
            can_withdraw: account.unstake_available_epoch_height <= env::epoch_height(),
            start_unstake_timestamp: account.unstake_start_timestamp,
            unstake_available_epoch: account.unstake_available_epoch_height,
            current_epoch: env::epoch_height()
        }
    }

    pub fn get_pool_info(&self) -> PoolInfo {
        PoolInfo { 
            total_stake_balance: U128(self.total_stake_balance), 
            total_reward: U128(self.pre_reward + self.internal_calculate_global_reward()), 
            total_stakers: U128(self.total_staker), 
            is_paused: self.paused
        }
    }
}