use near_sdk::{serde_json::json, json_types::U128};
use near_sdk_sim::{init_simulator, UserAccount, DEFAULT_GAS, STORAGE_AMOUNT, to_yocto};
use near_sdk_sim::transaction::ExecutionStatus;
use staking_contract::AccountJson;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FT_CONTRACT_WASM_FILE => "token-test/vbi-ft.wasm",
    STAKING_CONTRACT_WASM_FILE => "out/staking-contract.wasm"
}

const FT_CONTRACT_ID: &str = "ft_contract";
const FT_TOTAL_SUPPY: &str = "100000000000000000000000000000";
const FT_STAKING_CONTRACT_BALANCE: &str = "50000000000000000000000000000";
const STAKING_CONTRACT_ID: &str = "staking_contract";
const ALICE_DEPOSIT_BALANCE: &str = "10000000000000000000000000000";

pub fn init() -> (UserAccount, UserAccount, UserAccount, UserAccount) {
    let root = init_simulator(None);

    let alice = root.create_user("alice".to_string(), to_yocto("100"));

    // Deploy and init 1M Token
    let ft_contract = root.deploy_and_init(
        &FT_CONTRACT_WASM_FILE,
        FT_CONTRACT_ID.to_string(), 
        "new_default_meta",
        &json!({
            "owner_id": alice.account_id(),
            "total_supply": FT_TOTAL_SUPPY
        }).to_string().as_bytes(),
        STORAGE_AMOUNT,
        DEFAULT_GAS
    );

    // Deploy and init staking contract
    let staking_contract = root.deploy_and_init(
        &STAKING_CONTRACT_WASM_FILE, 
        STAKING_CONTRACT_ID.to_string(), 
        "new_default_config", 
        &json!({
            "owner_id": alice.account_id(),
            "ft_contract_id": ft_contract.account_id()
        }).to_string().as_bytes(), 
        STORAGE_AMOUNT, 
        DEFAULT_GAS
    );

    // storage deposit
    root.call(
        ft_contract.account_id(), 
        "storage_deposit", 
        &json!({
            "account_id": staking_contract.account_id()
        }).to_string().as_bytes(), 
        DEFAULT_GAS, 
        to_yocto("0.01")
    );

    // Transfer 50% total supply to staking contract
    alice.call(
        ft_contract.account_id(), 
        "ft_transfer", 
        &json!({
            "receiver_id": staking_contract.account_id(),
            "amount": FT_STAKING_CONTRACT_BALANCE
        }).to_string().as_bytes(), 
        DEFAULT_GAS, 
        1
    );

    (root, ft_contract, staking_contract, alice)
}


#[test]
fn init_contract_test() {
    let (root, ft_contract, staking_contract, alice) = init();

    // test deploy ft_contract
    let total_suppy: String = root.view(
        ft_contract.account_id(), 
        "ft_total_supply",
        &json!({}).to_string().as_bytes()
    ).unwrap_json();

    println!("Total supply: {}", total_suppy);
    assert_eq!(FT_TOTAL_SUPPY, total_suppy, "Total supply must equal {}", FT_TOTAL_SUPPY);

    // test alice balance
    let alice_balance: String = root.view(
        ft_contract.account_id(), 
        "ft_balance_of", 
        &json!({
            "account_id": alice.account_id()
        }).to_string().as_bytes()
    ).unwrap_json();

    println!("Alice balance: {}", alice_balance);
    assert_eq!(FT_STAKING_CONTRACT_BALANCE, alice_balance, "Alice balance must equal {}", FT_STAKING_CONTRACT_BALANCE);

    // test staking contract balance
    let staking_balance: String = root.view(
        ft_contract.account_id(), 
        "ft_balance_of", 
        &json!({
            "account_id": staking_contract.account_id()
        }).to_string().as_bytes()
    ).unwrap_json();

    println!("Staking contract balance: {}", staking_balance);
    assert_eq!(FT_STAKING_CONTRACT_BALANCE, staking_balance, "Staking contract balance must equal {}", FT_STAKING_CONTRACT_BALANCE);
}

#[test]
fn deposit_and_stake_test() {
    let (root, ft_contract, staking_contract, alice) = init();

    // staking contract storage deposit
    alice.call(
        staking_contract.account_id(), 
        "storage_deposit", 
        &json!({}).to_string().as_bytes(),
        DEFAULT_GAS, 
        to_yocto("0.01")
    );

    alice.call(
        ft_contract.account_id(), 
        "ft_transfer_call", 
        &json!({
            "receiver_id": staking_contract.account_id(),
            "amount": ALICE_DEPOSIT_BALANCE,
            "msg": ""
        }).to_string().as_bytes(),
         DEFAULT_GAS, 
        1
    );

    let account_json: AccountJson = root.view(
        staking_contract.account_id(), 
        "get_account_info", 
        &json!({
            "account_id": alice.account_id()
        }).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(account_json.account_id, alice.account_id());
    assert_eq!(account_json.stake_balance, U128(10000000000000000000000000000));
    assert!(account_json.reward.0 > 0);
    assert_eq!(account_json.unstake_balance.0, 0);
}

#[test]
fn deposit_and_stake_error_storage_test() {
    let (root, ft_contract, staking_contract, alice) = init();


    // Deposit without storage deposit
    let outcome = alice.call(
        ft_contract.account_id(), 
        "ft_transfer_call", 
        &json!({
            "receiver_id": staking_contract.account_id(),
            "amount": ALICE_DEPOSIT_BALANCE,
            "msg": ""
        }).to_string().as_bytes(),
         DEFAULT_GAS, 
        1
    );

    // Have one error
    assert_eq!(outcome.promise_errors().len(), 1);

    // assert error type
    if let ExecutionStatus::Failure(error) = &outcome.promise_errors().remove(0).unwrap().outcome().status {
        println!("Error: {}", error.to_string());
        assert!(error.to_string().contains("ERR_NOT_FOUND_ACCOUNT"));
    } else {
        unreachable!();
    }

}