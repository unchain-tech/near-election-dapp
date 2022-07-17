use crate::*;
use near_sdk::{ext_contract, log, Gas, PromiseResult};

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(10_000_000_000_000);
const GAS_FOR_NFT_ON_TRANSFER: Gas = Gas(25_000_000_000_000);

pub trait NonFungibleTokenCore {
    fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId, memo: Option<String>);

    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool>;

    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken>;
    fn nft_candidate_add_like(&mut self, token_id: TokenId) -> Option<u128>;
    fn nft_check_candidate_like(&mut self, token_id: TokenId) -> Option<u128>;
}

#[ext_contract(ext_non_fungible_token_receiver)]
trait NonFungibleTokenReceiver {
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> Promise;
}

#[ext_contract(ext_self)]
trait NonFungibleTokenResolver {
    fn nft_resolve_transfer(
        &mut self,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
    ) -> bool;
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    #[payable]
    // transfer token
    fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId, memo: Option<String>) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        self.internal_transfer(&sender_id, &receiver_id, &token_id, memo);
    }

    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        let previous_token = self.internal_transfer(&sender_id, &receiver_id, &token_id, memo);

        ext_non_fungible_token_receiver::ext(receiver_id.clone())
            .with_static_gas(GAS_FOR_NFT_ON_TRANSFER)
            .nft_on_transfer(
                sender_id,
                previous_token.owner_id.clone(),
                token_id.clone(),
                msg,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .nft_resolve_transfer(previous_token.owner_id, receiver_id, token_id),
            )
            .into()
    }

    // get specified token info
    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken> {
        if let Some(token) = self.tokens_by_id.get(&token_id) {
            let metadata = self.token_metadata_by_id.get(&token_id).unwrap();
            Some(JsonToken {
                owner_id: token.owner_id,
                metadata,
            })
        } else {
            None
        }
    }

    // add number of likes for specified candidate
    fn nft_candidate_add_like(&mut self, token_id: TokenId) -> Option<u128> {
        if self.tokens_by_id.get(&token_id).is_some() {
            let mut old_num_of_likes = self
                .token_metadata_by_id
                .get(&token_id)
                .unwrap()
                .num_of_likes;
            log!("Former number of likes is:{}", old_num_of_likes.unwrap());
            old_num_of_likes.replace(old_num_of_likes.unwrap() + 1);
            log!("New number of likes is:{}", old_num_of_likes.unwrap());
            log!(
                "New number of likes in contract is:{}",
                self.token_metadata_by_id
                    .get(&token_id)
                    .unwrap()
                    .num_of_likes
                    .unwrap()
            );

            old_num_of_likes
        } else {
            Some(0)
        }
    }

    // get number of likes of specified candidate
    fn nft_check_candidate_like(&mut self, token_id: TokenId) -> Option<u128> {
        if self.tokens_by_id.get(&token_id).is_some() {
            let metadata = self.token_metadata_by_id.get(&token_id).unwrap();
            metadata.num_of_likes
        } else {
            Some(0)
        }
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
    ) -> bool {
        if let PromiseResult::Successful(value) = env::promise_result(0) {
            if let Ok(return_token) = near_sdk::serde_json::from_slice::<bool>(&value) {
                if !return_token {
                    return true;
                }
            }
        }

        let mut token = if let Some(token) = self.tokens_by_id.get(&token_id) {
            if token.owner_id != receiver_id {
                return true;
            }
            token
        } else {
            return true;
        };

        log!("Return {} from @{} to @{}", token_id, receiver_id, owner_id);

        self.internal_remove_token_from_owner(&receiver_id, &token_id);

        self.internal_add_token_to_owner(&owner_id, &token_id);

        token.owner_id = owner_id;

        self.tokens_by_id.insert(&token_id, &token);

        false
    }
}
