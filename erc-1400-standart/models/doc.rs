use ink_env::Hash;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use ink_prelude::string::String;

#[derive(Debug, scale::Encode, PackedLayout, scale::Decode, SpreadLayout, Clone)]
#[cfg_attr( feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub struct Document { 
    pub doc_uri: String,
    pub doc_hash: Hash,
}
    