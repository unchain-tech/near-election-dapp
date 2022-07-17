use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]

    //mint token
    pub fn nft_mint(&mut self, mut metadata: TokenMetadata, receiver_id: AccountId) {
        metadata.token_id = Some(self.token_id_counter);
        metadata.num_of_likes = Some(0);
        let initial_storage_usage = env::storage_usage();
        let token = TokenOwner {
            owner_id: receiver_id,
        };
        let token_id = self.token_id_counter;
        let token_kind = &metadata.token_kind;

        assert!(
            self.tokens_by_id
                .insert(&self.token_id_counter, &token)
                .is_none(),
            "Token already exists"
        );

        self.token_metadata_by_id
            .insert(&self.token_id_counter, &metadata);
        self.internal_add_token_to_owner(&token.owner_id, &token_id);
        self.internal_add_token_to_kind_map(&token.owner_id, &token_id, &token_kind);

        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        self.token_id_count();

        refund_deposit(required_storage_in_bytes);
    }

    // count token id
    pub fn token_id_count(&mut self) {
        self.token_id_counter = self.token_id_counter + 1;
    }

    // get next token id
    pub fn show_token_id_counter(&self) -> u128 {
        self.token_id_counter
    }
}
