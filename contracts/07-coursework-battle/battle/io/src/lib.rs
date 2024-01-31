
#![no_std]
use gmeta::{In, InOut, Metadata, Out};
use gstd::{prelude::*, ActorId, collections::{BTreeMap, BTreeSet}};
use store_io::AttributeId;


pub type TamagotchiId = ActorId;

#[derive(Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum BattleState {
    #[default]
    Registration,
    Moves,
    Waiting,
    GameIsOver,
}

#[derive(Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Battle {
    players: Vec<Player>,
    state: BattleState,
    current_turn: u8,
    tmg_store_id: ActorId,
    winner: ActorId,
    steps: u8,
}

#[derive(Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Player {
   owner: ActorId,
   tmg_id: TamagotchiId,
   energy: u16,
   power: u16,
   attributes: BTreeSet<AttributeId>,
}




#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum BattleAction {
    Register(TamagotchiId),
    Move(TamagotchiId),
    // Attack(TamagotchiId, u8),
    // Defend(TamagotchiId, u8),
    // Surrender(TamagotchiId),
    
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum BattleEvent {
    Registered { tmg_id: TamagotchiId },
    Moved { tmg_id: TamagotchiId, steps: u8 },
    GameOver { winner: ActorId },
    
}

pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
    type Init = In<ActorId>;
    type Handle = InOut<BattleAction, BattleEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = Out<Battle>;
}


