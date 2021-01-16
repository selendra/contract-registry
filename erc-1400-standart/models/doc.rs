use ink_env::Hash;
use ink_storage::traits::{PackedLayout, SpreadLayout};

#[derive(Debug, scale::Encode, PackedLayout, scale::Decode, SpreadLayout)]
#[cfg_attr( feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub struct Document { 
    doc_uri: String,
    doc_hash: Hash,
}
    