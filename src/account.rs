use near_sdk::Timestamp;

use crate::*;


#[derive(BorshDeserialize, BorshSerialize)]
pub enum UpgradableAccount {
    Default(Account),
    Current(Account)
}

impl From<UpgradableAccount> for Account {
    fn from(account: UpgradableAccount) -> Self {
        match account {
            UpgradableAccount::Default(account) => account,
            UpgradableAccount::Current(account) => account
        }
    }
}

impl From<Account> for UpgradableAccount {
    fn from(account: Account) -> Self {
        UpgradableAccount::Current(account)
    }
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Account {
    pub stake_balance: Balance,
    pub pre_stake_balance: Balance,
    pub pre_reward: Balance,
    pub last_block_balance_change: BlockHeight,
    pub unstake_balance: Balance,
    pub unstake_start_timestamp: Timestamp,
    pub unstake_available_epoch_height: EpochHeight
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountJson {
    pub account_id: AccountId,
    pub stake_balance: U128,
    pub unstake_balance: U128,
    pub reward: U128,
    pub can_withdraw: bool,
    pub start_unstake_timestamp: Timestamp,
    pub unstake_available_epoch: EpochHeight,
    pub current_epoch: EpochHeight
}