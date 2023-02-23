/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{
    assert_one_yocto, env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue,
};
use std::convert::TryInto;
use std::u128;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    owner_id: AccountId,
    metadata: LazyOption<FungibleTokenMetadata>,
    max_supply: Balance
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, max_supply: Balance) -> Self {
        Self::new(
            owner_id,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
            max_supply
        )
    }

    pub fn set_owner(&mut self, owner_id: AccountId) -> AccountId {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "ERR_NOT_ALLOWED"
        );
        self.owner_id = owner_id.into();
        self.owner_id.clone().try_into().unwrap()
    }

    pub fn get_owner(&mut self) -> AccountId {
        let owner: AccountId = self.owner_id.clone().try_into().unwrap();
        return owner;
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(owner_id: AccountId, metadata: FungibleTokenMetadata, max_supply: Balance) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            owner_id: owner_id,
            max_supply: max_supply
        };
        this
    }

    pub fn mint(&mut self, account_id: ValidAccountId, amount: U128) -> U128 {
        // assert_one_yocto();
        // assert_eq!(false, true, "Revert");
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "ERR_NOT_ALLOWED"
        );
        let next_total_supply:Balance = self.token.total_supply.checked_add(amount.into()).unwrap();
        assert!(next_total_supply<=self.max_supply, "Overflow");
        let account = self.token.accounts.get(account_id.as_ref());
        if account == None {
            self.token.internal_register_account(account_id.as_ref());
        }
        self.token
            .internal_deposit(account_id.as_ref(), amount.into());
        amount
    }

    pub fn burn(&mut self, account_id: ValidAccountId, amount: U128) {
        assert_one_yocto();
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "ERR_NOT_ALLOWED"
        );
        self.token
            .internal_withdraw(account_id.as_ref(), amount.into());
    }

    pub fn change_max_supply(&mut self, max_supply: Balance) {
        assert_one_yocto();
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "ERR_NOT_ALLOWED"
        );
        self.max_supply = max_supply;
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token);
near_contract_standards::impl_fungible_token_storage!(Contract, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        let metadata = self.metadata.get().unwrap();
        metadata
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{env, testing_env, MockedBlockchain};

    use super::*;

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.build());
        let max_supply:Balance = 210000;
        let mut contract = Contract::new(accounts(0).to_string(), {
            FungibleTokenMetadata {
                spec: "ft-1.0.0".to_string(),
                name: "ZEUS".to_string(),
                symbol: "zeus".to_string(),
                decimals: 8,
                icon: None,
                reference: None,
                reference_hash: None,
            }
        },max_supply);
        // testing_env!(context
        //     .predecessor_account_id(farmer)
        //     .is_view(false)
        //     .block_timestamp(to_nano(time_stamp))
        //     .attached_deposit(1)
        //     .build());

        testing_env!(context
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        // contract.mint(accounts(0), 1_000_000.into());
        // assert_eq!(contract.ft_balance_of(accounts(0)), 1_000_000.into());
        contract.change_max_supply(1_000_000);
        contract.mint(accounts(0), 1_000_000.into());
        println!("MintedValue: {:?}", contract.ft_balance_of(accounts(0)));
        // assert_eq!(contract.ft_balance_of(accounts(0)), 2_000_000.into());
        // contract.burn(accounts(0), 1_000_000.into());

        // testing_env!(context
        //     .attached_deposit(125 * env::storage_byte_cost())
        //     .build());
        // contract.storage_deposit(Some(accounts(1)), None);
        // testing_env!(context
        //     .attached_deposit(1)
        //     .predecessor_account_id(accounts(0))
        //     .build());
        // contract.ft_transfer(accounts(1), 1_000.into(), None);
        // assert_eq!(contract.ft_balance_of(accounts(1)), 1_000.into());

        // contract.burn(accounts(1), 500.into());
        // assert_eq!(contract.ft_balance_of(accounts(1)), 500.into());
    }
}
