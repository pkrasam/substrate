// Copyright 2017 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Canonical hash trie definitions and helper functions.
//!
//! Each CHT is a trie mapping block numbers to canonical hash.
//! One is generated for every `SIZE` blocks, allowing us to discard those blocks in
//! favor of the trie root. When the "ancient" blocks need to be accessed, we simply
//! request an inclusion proof of a specific block number against the trie with the
//! root has. A correct proof implies that the claimed block is identical to the one
//! we discarded.

use triehash;

use primitives::block::{HeaderHash, Number as BlockNumber};

use utils::number_to_db_key;

/// The size of each CHT.
pub const SIZE: BlockNumber = 2048;

/// Returns Some(cht_number) if CHT is need to be built when the block with given number is canonized.
pub fn is_build_required(block_num: BlockNumber) -> Option<BlockNumber> {
	let block_cht_num = block_to_cht_number(block_num)?;
	if block_cht_num < 2 {
		return None;
	}
	let cht_start = start_number(block_cht_num);
	if cht_start != block_num {
		return None;
	}

	Some(block_cht_num - 2)
}

/// Compute a CHT root from an iterator of block hashes. Fails if shorter than
/// SIZE items. The items are assumed to proceed sequentially from `start_number(cht_num)`.
/// Discards the trie's nodes.
pub fn compute_root<I: IntoIterator<Item=Option<HeaderHash>>>(cht_num: BlockNumber, hashes: I) -> Option<HeaderHash> {
	let start_num = start_number(cht_num);
	let pairs = hashes.into_iter()
		.take(SIZE as usize)
		.enumerate()
		.filter_map(|(i, hash)| hash.map(|hash| (number_to_db_key(start_num + i as BlockNumber).to_vec(), hash.to_vec())))
		.collect::<Vec<_>>();
	if pairs.len() != SIZE as usize {
		return None;
	}

	Some(triehash::trie_root(pairs).0.into())
}

/// Get the starting block of a given CHT.
/// CHT 0 includes block 1...SIZE,
/// CHT 1 includes block SIZE + 1 ... 2*SIZE
/// More generally: CHT N includes block (1 + N*SIZE)...((N+1)*SIZE).
/// This is because the genesis hash is assumed to be known
/// and including it would be redundant.
pub fn start_number(cht_num: BlockNumber) -> BlockNumber {
	(cht_num * SIZE) + 1
}

/// Get the ending block of a given CHT.
pub fn end_number(cht_num: BlockNumber) -> BlockNumber {
	start_number(cht_num) + SIZE
}

/// Convert a block number to a CHT number.
/// Returns `None` for `block_num` == 0, `Some` otherwise.
pub fn block_to_cht_number(block_num: BlockNumber) -> Option<BlockNumber> {
	match block_num {
		0 => None,
		n => Some((n - 1) / SIZE),
	}
}

/// Convert CHT node value into block header hash.
pub fn decode_cht_value(value: &[u8]) -> HeaderHash {
	value.into()
}