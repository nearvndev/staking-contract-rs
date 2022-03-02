use crate::*;

impl StakingContract {

    // User deposit FT token and stake
    pub(crate) fn internal_deposit_and_stake(&mut self, account_id: AccountId, amount: Balance) {

        // Check account exists
        let upgradable_account: UpgradableAccount = self.accounts.get(&account_id).unwrap();
        let mut account = Account::from(upgradable_account);

        if account.stake_balance == 0 {
            self.total_staker += 1;
        }

        // if exist account, update balance and update pre data
        let new_reward: Balance = self.internal_calculate_account_reward(&account);

        // update account data
        account.pre_stake_balance = account.stake_balance;
        account.pre_reward += new_reward;
        account.stake_balance += amount;
        account.last_block_balance_change = env::block_index();
        self.accounts.insert(&account_id, &UpgradableAccount::from(account));


        // Update contract data
        let new_contract_reward: Balance = self.internal_calculate_global_reward();
        self.total_stake_balance += amount;
        self.pre_reward += new_contract_reward;
        self.last_block_balance_change = env::block_index();

    }

    pub(crate) fn internal_unstake(&mut self, account_id: AccountId, amount: Balance) {
        let upgradable_account: UpgradableAccount = self.accounts.get(&account_id).unwrap();

        let mut account = Account::from(upgradable_account);

        assert!(amount <= account.stake_balance, "ERR_AMOUNT_MUST_LESS_THAN_BALANCE");

        // if exist account, update balance and update pre data
        let new_reward: Balance = self.internal_calculate_account_reward(&account);

        // update account data
        account.pre_stake_balance = account.stake_balance;
        account.pre_reward += new_reward;
        account.stake_balance -= amount;
        account.last_block_balance_change = env::block_index();
        account.unstake_available_epoch_height = env::epoch_height() + NUM_EPOCHS_TO_UNLOCK;
        account.unstake_balance += amount;
        account.unstake_start_timestamp = env::block_timestamp();
        
        if account.stake_balance == 0 {
            self.total_staker -= 1;
        }

        // update new account data
        self.accounts.insert(&account_id, &UpgradableAccount::from(account));

        // update contract data
        let new_contract_reward: Balance = self.internal_calculate_global_reward();
        self.total_stake_balance -= amount;
        self.pre_reward += new_contract_reward;
        self.last_block_balance_change = env::block_index();
    }

    pub(crate) fn internal_withdraw(&mut self, account_id: AccountId) -> Account {
        let upgradable_account: UpgradableAccount = self.accounts.get(&account_id).unwrap();
        let account: Account = Account::from(upgradable_account);

        assert!(account.unstake_balance > 0, "ERR_UNSTAKE_BALANCE_IS_ZERO");
        assert!(account.unstake_available_epoch_height <= env::epoch_height(), "ERR_DISABLE_WITHDRAW");

        let new_account: Account = Account {
            pre_reward: account.pre_reward,
            stake_balance: account.stake_balance,
            pre_stake_balance: account.pre_stake_balance,
            last_block_balance_change: account.last_block_balance_change,
            unstake_balance: 0,
            unstake_start_timestamp: 0,
            unstake_available_epoch_height: 0
        };

        self.accounts.insert(&account_id, &UpgradableAccount::from(new_account));

        account
    }

    pub(crate) fn internal_calculate_account_reward(&self, account: &Account) -> Balance {
        let lasted_block = if self.paused {
            self.paused_in_block
        } else {
            env::block_index()
        };
        let diff_block = lasted_block - account.last_block_balance_change;
        let reward: Balance = (account.stake_balance * self.config.reward_numerator as u128 * diff_block as u128) / (self.config.reward_denumerator as u128);
        reward
    }

    pub(crate) fn internal_calculate_global_reward(&self) -> Balance {
        let lasted_block = if self.paused {
            self.paused_in_block
        } else {
            env::block_index()
        };
        let diff_block = lasted_block - self.last_block_balance_change;
        let reward: Balance = (self.total_stake_balance * self.config.reward_numerator as u128 * diff_block as u128) / (self.config.reward_denumerator as u128);
        reward
    }
}