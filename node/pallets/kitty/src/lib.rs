#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, dispatch,ensure, sp_runtime, Parameter,
	traits::{Get, Randomness, Currency, ReservableCurrency, ExistenceRequirement},
};
use frame_system::ensure_signed;
use sp_runtime::{
	DispatchError,
	traits::{AtLeast32Bit, Bounded, Member},
};
use codec::{Encode, Decode};
use sp_io::hashing::blake2_128;
use sp_std::vec;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Randomness: Randomness<Self::Hash>;
	type KittyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;

	type NewKittyReserve: Get<BalanceOf<Self>>;
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Trait> as KittiesModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		
		pub KittyDB get(fn kitty_db): map hasher(blake2_128_concat) <T as Trait>::KittyIndex => Option<Kitty>;
		pub KittiesCount get(fn kitties_count): <T as Trait>::KittyIndex;
		pub KittyOwner get(fn kitties_owner):  map hasher(blake2_128_concat)  <T as Trait>::KittyIndex => Option<T::AccountId>;
		
		pub KittyTotal get(fn kitty_total) : map hasher(blake2_128_concat) T::AccountId => vec::Vec<T::KittyIndex>;

        pub KittiesParents get(fn kitty_parents) : map hasher(blake2_128_concat) T::KittyIndex => (T::KittyIndex, T::KittyIndex);

        pub KittiesChildren get(fn kitty_children): double_map hasher(blake2_128_concat) T::KittyIndex,  hasher(blake2_128_concat) T::KittyIndex => vec::Vec<T::KittyIndex>;

        pub KittiesSiblings get(fn kitty_siblings): map hasher(blake2_128_concat) T::KittyIndex => vec::Vec<T::KittyIndex>;

        pub KittiesSpouse get(fn kitty_spouse) : map hasher(blake2_128_concat) T::KittyIndex => vec::Vec<T::KittyIndex>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where  AccountId = <T as frame_system::Trait>::AccountId, <T as Trait>::KittyIndex{
		Created(AccountId, KittyIndex),
		Transferred(AccountId, AccountId, KittyIndex),
		Breed(AccountId, KittyIndex, KittyIndex, KittyIndex),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		KittiesCountOverflow,
		KittyIdInvalid,
		SameParentNotAllowed,
		KittyNotExists,
        NotKittyOwner,
        TransferToSelf,
		NoEnoughBalance,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;
		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		#[weight = 0]
		pub fn create(origin) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id()?;
			let dna = Self::random_value(&sender);
			
			T::Currency::reserve(&sender, T::NewKittyReserve::get()).map_err(|_| Error::<T>::NoEnoughBalance)?;
			
			Self::insert_kitty(&sender, kitty_id, Kitty(dna));

			Self::deposit_event(RawEvent::Created(sender, kitty_id));
			Ok(())
		}

		#[weight = 0]
		pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(to != sender, Error::<T>::TransferToSelf);

			let owner = Self::kitties_owner(kitty_id).ok_or(Error::<T>::KittyNotExists)?;
			ensure!(owner == sender, Error::<T>::NotKittyOwner);

			KittyOwner::<T>::insert(kitty_id, to.clone());

			KittyTotal::<T>::mutate(&sender, |val| val.retain(|&temp| temp == kitty_id));
			KittyTotal::<T>::mutate(&to, |val| val.push(kitty_id));
			
			Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
			Ok(())
		}

		#[weight = 0]
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
			Self::deposit_event(RawEvent::Breed(sender, kitty_id_1, kitty_id_2, new_kitty_id));
			Ok(())
		}

	}
}


impl <T: Trait>Module<T> {
	fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
		let kitty_id = Self::kitties_count(); // jusr get value
		if kitty_id == T::KittyIndex::max_value(){
			return Err(Error::<T>::KittiesCountOverflow.into())
		}

		Ok(kitty_id)
	}

	fn random_value(sender: &T::AccountId) -> [u8;16]{
		let payload = (
			T::Randomness::random_seed(),
			&sender,
			frame_system::Module::<T>::extrinsic_index(),
		);
		payload.using_encoded(blake2_128)
	}

	fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty){
		
		KittyDB::<T>::insert(kitty_id, kitty);
		KittiesCount::<T>::put(kitty_id + 1.into());
		KittyOwner::<T>::insert(kitty_id, owner);

		if KittyTotal::<T>::contains_key(&owner){
			KittyTotal::<T>::mutate(owner, |val| val.push(kitty_id));
		}else{
			KittyTotal::<T>::insert(owner, vec![kitty_id]);
		}
	}

	fn update_kitties_parents(
        children: T::KittyIndex,
        parent1: T::KittyIndex,
        parent2: T::KittyIndex,
    ) {
        <KittiesParents<T>>::insert(children, (parent1, parent2));
    }

    fn update_kitties_children(
        children: T::KittyIndex,
        parent1: T::KittyIndex,
        parent2: T::KittyIndex,
    ) {
        if <KittiesChildren<T>>::contains_key(parent1, parent2) {
            let _ = <KittiesChildren<T>>::mutate(parent1, parent2, |val| val.push(children));
        } else {
            <KittiesChildren<T>>::insert(parent1, parent2, vec![children]);
        }
	}
	
    fn update_kitties_siblings(kitty_id: T::KittyIndex) {
		let (parent1, parent2) = KittiesParents::<T>::get(kitty_id);
		
        if <KittiesChildren<T>>::contains_key(parent1, parent2) {
            let val: vec::Vec<T::KittyIndex> = KittiesChildren::<T>::get(parent1, parent2);
            let reserve_val: vec::Vec<T::KittyIndex> =
                val.into_iter().filter(|&val| val != kitty_id).collect();
            <KittiesSiblings<T>>::insert(kitty_id, reserve_val);
        } else {
            <KittiesSiblings<T>>::insert(kitty_id, vec::Vec::<T::KittyIndex>::new());
        }
    }

    fn update_kitties_spouse(partner1: T::KittyIndex, partner2: T::KittyIndex) {
		if KittiesSpouse::<T>::contains_key(&partner1){
			let val: vec::Vec<T::KittyIndex> = KittiesSpouse::<T>::get(partner1);
			let reserve_val: vec::Vec<T::KittyIndex> =
				val.into_iter().filter(|&val| val == partner2).collect();
			if reserve_val.len() == 0 {
				KittiesSpouse::<T>::mutate(&partner1, |val| val.push(partner2));
			}
		}else{
			KittiesSpouse::<T>::insert(partner1, vec![partner2]);
		};
    }


	fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8{
		(selector & dna1) | (!selector & dna2)
	}

	fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex, DispatchError> {
		ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentNotAllowed);

		T::Currency::reserve(&sender, T::NewKittyReserve::get()).map_err(|_| Error::<T>::NoEnoughBalance)?;

		let kitty1 = Self::kitty_db(kitty_id_1).ok_or(Error::<T>::KittyIdInvalid)?;
		let kitty2 = Self::kitty_db(kitty_id_2).ok_or(Error::<T>::KittyIdInvalid)?;

		let owner1 = Self::kitties_owner(kitty_id_1).ok_or(Error::<T>::KittyNotExists)?;
		let owner2 = Self::kitties_owner(kitty_id_2).ok_or(Error::<T>::KittyNotExists)?;
		
		ensure!(owner1 == *sender, Error::<T>::NotKittyOwner);
		ensure!(owner2 == *sender, Error::<T>::NotKittyOwner);
		
		let kitty_id = Self::next_kitty_id()?;

		let kitty1_dna = kitty1.0;
		let kitty2_dna = kitty2.0;

		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8; 16];

		for i in 0..kitty1_dna.len() {
			new_dna[i] = Self::combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
		}

		Self::insert_kitty(sender, kitty_id, Kitty(new_dna));
		
        Self::update_kitties_spouse(kitty_id_1, kitty_id_2);
        Self::update_kitties_spouse(kitty_id_2,kitty_id_1);
        Self::update_kitties_parents(kitty_id, kitty_id_1, kitty_id_2);
        Self::update_kitties_children(kitty_id, kitty_id_1, kitty_id_2);
        Self::update_kitties_siblings(kitty_id);

		Ok(kitty_id)
	}

}