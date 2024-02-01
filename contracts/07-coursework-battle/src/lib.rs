
#![no_std]
use gmeta::{In, InOut, Metadata, Out};
use gstd::{collections::BTreeSet, exec, msg, panic, prelude::*, ActorId};
use store_io::{AttributeId, StoreAction, StoreEvent};
use coursework_battle_io::{TmgAction, TmgEvent};

const MAX_POWER: u16 = 10_000;
const MIN_POWER: u16 = 3_000;

const MAX_ENERGY: u16 = 20_000;
const MIN_ENERGY: u16 = 6_000;

const SWORD_POWER: u16 = 2;

//CONST SHIELD_POWER: u16 = 2;
//CONST SHIELD_ID: AttributeId = Default::default();

const TIME_FOR_UPDATE: u32 = 500;
const MAX_STEPS_FOR_ROUND: u8 = 3;
const GAS_AMOUNT: u64 = 100_000_000;

pub type TamagotchiId = ActorId;

#[derive(Default, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
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
    pub players: Vec<Player>,
    pub state: BattleState,
    pub current_turn: u8,
    pub tmg_store_id: ActorId,
    pub winner: ActorId,
    pub steps: u8,
}

impl Battle {
    pub async fn register(&mut self, tmg_id: &TamagotchiId) {
        assert_eq!(
            self.state,
            BattleState::Registration,
            "The game has already started"
        );

        let owner = get_owner(tmg_id).await;
        let attributes = get_attributes(&self.tmg_store_id, tmg_id).await;

        let power = generate_power();
        let power = MAX_POWER - power;
        let energy = generate_energy();
        let player = Player {
            owner,
            tmg_id: *tmg_id,
            energy,
            power,
            attributes,
        };
        self.players.push(player);
        if self.players.len() == 2 {
            self.current_turn = get_turn();
            self.state = BattleState::Moves;
        }
        msg::reply(BattleEvent::Registered { tmg_id: *tmg_id }, 0)
            .expect("Error during a reply `BattleEvent::Registered");
    }

    pub fn get_turn() -> u8 {
        let random_input: [u8; 32] = array::from_fn(|i| i as u8 + 1);
        let (random, _) = exec::random(random_input).expect("Error in getting random number");
        random[0] % 2
    }

    pub fn make_move(&mut self) {
        assert_eq!(
            self.state,
            BattleState::Moves,
            "The game is not in `Moves` state"
        );
        let turn = self.current_turn as usize;

        let next_turn = ((turn + 1) % 2) as usize;
        let player = self.players[turn].clone();
        assert_eq!(
            player.owner,
            msg::source(),
            "You are not in the game or it is not your turn"
        );
        let mut opponent = self.players[next_turn].clone();
        // let sword_power = if player.attributes.contains(&SWORD_ID) {
        //     SWORD_POWER
        // } else {
        //     1
        // };
        let sword_power = SWORD_POWER;

        opponent.energy = opponent.energy.saturating_sub(sword_power * player.power);

        self.players[next_turn] = opponent.clone();
        // Check if the opponent lost
        if opponent.energy == 0 {
            self.players = Vec::new();
            self.state = BattleState::GameIsOver;
            self.winner = player.tmg_id;
            msg::reply(BattleEvent::GameIsOver, 0)
                .expect("Error in sending a reply `BattleEvent::GameIsOver`");
            return;
        }
        if self.steps <= MAX_STEPS_FOR_ROUND {
            self.steps += 1;
            self.current_turn = next_turn as u8;
            msg::reply(BattleEvent::MoveMade, 0)
                .expect("Error in sending a reply `BattleEvent::MoveMade`");
        } else {
            self.state = BattleState::Waiting;
            self.steps = 0;
            msg::send_with_gas_delayed(
                exec::program_id(),
                BattleAction::UpdateInfo,
                GAS_AMOUNT,
                0,
                TIME_FOR_UPDATE,
            )
            .expect("Error in sending a delayed message `BattleAction::UpdateInfo`");
            msg::reply(BattleEvent::GoToWaitingState, 0)
                .expect("Error in sending a reply `BattleEvent::MoveMade`");
        }
    }
    pub async fn update_info(&mut self) {
        assert_eq!(
            msg::source(),
            exec::program_id(),
            "Only the contract itself can call that action"
        );
        assert_eq!(
            self.state,
            BattleState::Waiting,
            "The contract must be in `Waiting` state"
        );

        for i in 0..2 {
            let player = &mut self.players[i];
            let attributes = get_attributes(&self.tmg_store_id, &player.tmg_id).await;
            player.attributes = attributes;
        }
        self.state = BattleState::Moves;
        self.current_turn = get_turn();
        msg::reply(BattleEvent::InfoUpdated, 0)
            .expect("Error during a reply BattleEvent::InfoUpdated");
    }
}

#[derive(Default, Encode, Decode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Player {
    pub owner: ActorId,
    pub tmg_id: TamagotchiId,
    pub energy: u16,
    pub power: u16,
    pub attributes: BTreeSet<AttributeId>,
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum BattleAction {
    Register(TamagotchiId),
    Move,
    UpdateInfo,
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
    GoToWaitingState,
    InfoUpdated,
    GameIsOver,
    MoveMade,
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

pub async fn get_owner(tmg_id: &ActorId) -> ActorId {
    let reply: TmgEvent = msg::send_for_reply_as(*tmg_id, TmgAction::TmgInfo, 0, 0)
        .expect("Error in sending a message `TmgAction::Owner")
        .await
        .expect("Unable to decode TmgEvent");
    if let TmgEvent::Owner(owner) = reply {
        owner
    } else {
        panic!("Wrong received message");
    }
}

pub fn generate_power() -> u16 {
    let random_input: [u8; 32] = array::from_fn(|i| i as u8 + 1);
    let (random, _) = exec::random(random_input).expect("Error in getting random number");
    let bytes: [u8; 2] = [random[0], random[1]];
    let random_power: u16 = u16::from_be_bytes(bytes) % MAX_POWER;
    if random_power < MIN_POWER {
        return MAX_POWER / 2;
    }
    random_power
}

pub fn generate_energy() -> u16 {
    let random_input: [u8; 32] = array::from_fn(|i| i as u8 + 1);
    let (random, _) = exec::random(random_input).expect("Error in getting random number");
    let bytes: [u8; 2] = [random[0], random[1]];
    let random_energy: u16 = u16::from_be_bytes(bytes) % MAX_ENERGY;
    if random_energy < MIN_ENERGY {
        return MAX_ENERGY / 2;
    }
    random_energy
}

async fn get_attributes(tmg_store_id: &ActorId, tmg_id: &TamagotchiId) -> BTreeSet<AttributeId> {
    let reply: StoreEvent = msg::send_for_reply_as(
        *tmg_store_id,
        StoreAction::GetAttributes {
            tamagotchi_id: *tmg_id,
        },
        0,
        0,
    )
    .expect("Error in sending a message `StoreAction::GetAttributes")
    .await
    .expect("Unable to decode `StoreEvent`");
    if let StoreEvent::Attributes { attributes } = reply {
        attributes
    } else {
        panic!("Wrong received message");
    }
}

pub fn get_turn() -> u8 {
    let random_input: [u8; 32] = array::from_fn(|i| i as u8 + 1);
    let (random, _) = exec::random(random_input).expect("Error in getting random number");
    random[0] % 2
}
