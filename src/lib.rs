#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;
use support::{decl_module, decl_storage, decl_event};
use system::{ensure_root, ensure_none};
use system::offchain::SubmitUnsignedTransaction;
use sr_primitives::{
	traits::SaturatedConversion,
	transaction_validity::{
		TransactionValidity, ValidTransaction, InvalidTransaction,
		TransactionPriority, TransactionLongevity,
	},
};

/// The module's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;

	/// A dispatchable call type.
	type Call: From<Call<Self>>;

	/// A transaction submitter.
	type SubmitTransaction: SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;
}

decl_event!(
	pub enum Event {
		NewPrice(u64),
	}
);

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		Price get(price): u64;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		fn set_prices(origin, price: u64){
			ensure_root(origin)?;
			Price::put(price);
		}

		fn submit_price(origin, price: u64) {
			ensure_none(origin)?;
			Price::put(price);
		}

		fn offchain_worker(n: T::BlockNumber) {
			Self::get_price(n);
		}
	}
}

impl<T: Trait> Module<T> {
	pub fn get_price(n: T::BlockNumber) {
		let price = n.saturated_into::<u64>();
		let call = Call::submit_price(price);
		let _ = T::SubmitTransaction::submit_unsigned(call);
	}
}


impl<T: Trait> support::unsigned::ValidateUnsigned for Module<T> {
	type Call = Call<T>;

	fn validate_unsigned(call: &Self::Call) -> TransactionValidity {
		match call {
			Call::submit_price(_) => {
				Ok(ValidTransaction {
					priority: TransactionPriority::max_value(),
					requires: vec![],
					provides: vec![],
					longevity: TransactionLongevity::max_value(),
					propagate: true,
				})
			}
			_ => Err(InvalidTransaction::Call.into()),
		}
	}
}

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use primitives::H256;
	use support::{impl_outer_origin, impl_outer_dispatch, assert_ok, parameter_types};
	use sr_primitives::{
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Header, TestXt},
		weights::Weight,
		Perbill,
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	impl_outer_dispatch! {
		pub enum Call for Test where origin: Origin {
			price_oracle::PriceOracle,
		}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = Call;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}

	type Extrinsic = TestXt<Call, ()>;
	type SubmitTransaction = system::offchain::TransactionSubmitter<(), Call, Extrinsic>;

	impl Trait for Test {
		type Event = ();
		type Call = Call;
		type SubmitTransaction = SubmitTransaction;
	}
	type PriceOracle = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn it_works_for_default_value() {
		new_test_ext().execute_with(|| {
			assert_ok!(PriceOracle::submit_price(Origin::system(system::RawOrigin::None), 100));
			assert_eq!(PriceOracle::price(), 100);
		});
	}
}
