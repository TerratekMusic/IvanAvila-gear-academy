#![no_std]

use gstd::prelude::*;
use scale_info::TypeInfo;
use gmeta::{In, InOut, Metadata, Out};

#[derive(Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Tamagotchi {
    pub name: String,
    pub date_of_birth: u64,
    pub owner: ActorId,
    pub fed: u64,
    pub fed_block: u64,
    pub entertained: u64,
    pub entertained_block: u64,
    pub slept: u64,
    pub slept_block: u64,
    pub approved_account: Option<ActorId>,
    // TODO: 2️⃣ Add new fields
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum TmgAction {
    Name,
    Age,
    Feed,
    Entertain,
    Sleep,
    Transfer(ActorId),
    Approve(ActorId),
    RevokeApproval,
    // TODO: 3️⃣ Add new actions
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum TmgEvent {
    Name(String),
    Age(u64),
    Fed(u64),
    Entertained(u64),
    Slept(u64),
    Transfer(ActorId),
    Approve(ActorId),
    RevokeApproval,
    // TODO: 4️⃣ Add new events
}

pub struct ProgramMetadata;

// TODO: 0️⃣ Copy `Metadata` from the first lesson and push changes to the master branch
impl Metadata for ProgramMetadata {
    type Init = In<String>;
    type Handle = InOut<TmgAction, TmgEvent>;
    type State = Out<Tamagotchi>;
    type Reply = ();
    type Others = ();
    type Signal = ();
}
