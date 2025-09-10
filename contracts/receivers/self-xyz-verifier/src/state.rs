use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const MAILBOX: Item<Addr> = Item::new("hpl-mailbox");
// Map<evm_address_bytes, cosmos_address>
pub const VERIFICATIONS: Map<&[u8], Addr> = Map::new("verifications");
