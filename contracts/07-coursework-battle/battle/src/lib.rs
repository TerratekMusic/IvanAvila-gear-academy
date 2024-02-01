
#![no_std]
use battle_io::{BattleAction, BattleEvent, BattleState};
use gstd::{debug, exec, msg, prelude::*, ActorId, panic, collections::{BTreeMap, BTreeSet}, async_main };
use coursework_battle_io::{ TmgAction, TmgEvent};
use store_io::{StoreAction, StoreEvent, AttributeId, TamagotchiId};


const MAX_POWER: u16 = 10_000;
const MIN_POWER: u16 = 3_000;

const SWORD_POWER: u16 = 2;
const SWORD_ID: AttributeId = Default::default();

//CONST SHIELD_POWER: u16 = 2;
//CONST SHIELD_ID: AttributeId = Default::default();

const TIME_FOR_UPDATE: u32 = 500;
const MAX_STEPS_FOR_ROUND: u8 = 3;
const GAS_AMOUNT: u64 = 100_000_000;

static mut BATTLE : Option<Battle> = None;
static mut PLAYER : Option<Player> = None;

#[derive(Default)]
struct Battle {
    players: Vec<Player>,
    state: BattleState,
    current_turn: u8,
    tmg_store_id: ActorId,
    winner: ActorId,
    steps: u8,
}

#[derive(Default, Clone)]
pub struct Player {
    owner: ActorId,
    tmg_id: TamagotchiId,
    energy: u16,
    power: u16,
    attributes: BTreeSet<AttributeId>,
}

impl Battle {
    async fn register(&mut self, tmg_id: &TamagotchiId) {
        assert_eq!(
            self.state,
            BattleState::Registration,
            "The game has already started"
        );

        let owner = get_owner(tmg_id).await;
        let attributes = get_attributes(&self.tmg_store_id, tmg_id).await;

        let power = generate_power();
        let power = MAX_POWER - power;
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
        let (random, _) = exec::random(random_input)
            .expect("Error in getting random number");
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
            msg::reply(BattleEvent::GameOver, 0)
                .expect("Error in sending a reply `BattleEvent::GameIsOver`");
            return;
        }
        if self.steps <= MAX_STEPS_FOR_ROUND {
            self.steps += 1;
            self.current_turn = next_turn as u8;
            msg::reply(BattleEvent::Moved, 0)
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
    async fn update_info(&mut self) {
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


#[no_mangle]
extern fn init() {
    let init_message: BattleAction = msg::load().expect("Can't decode regitration message");
    let battle = Battle{
        players: Vec::new(),
        state: BattleState::Registration,
        current_turn: 0,
        tmg_store_id: ActorId::default(),
        winner: ActorId::default(),
        steps: 0,
    };

  unsafe{
    BATTLE = Some(battle);
  }
    
}

#[gstd::async_main]
async fn main() {


    let action: BattleAction = msg::load().expect("Unable to decode `BattleAction`");
    let battle: &mut Battle = unsafe { BATTLE.get_or_insert(Default::default()) };

    // match action {
    //     BattleAction::UpdateInfo => battle.update_info().await,
    //     BattleAction::Move => battle.make_move().await,
    //     BattleAction::Register (TamagotchiId) => battle.register(&TamagotchiId).await,
    // }

}

pub async fn get_owner(tmg_id: &ActorId) -> ActorId {
    let reply: TmgEvent = msg::send_for_reply_as(*tmg_id, TmgAction::TmgInfo, 0, 0)
        .expect("Error in sending a message `TmgAction::TmgInfo`")
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
    let (random, _) = exec::random(random_input)
        .expect("Error in getting random number");
    let bytes: [u8; 2] = [random[0], random[1]];
    let random_power: u16 = u16::from_be_bytes(bytes) % MAX_POWER;
    if random_power < MIN_POWER {
        return MAX_POWER / 2;
    }
    random_power
}



async fn get_attributes(
    tmg_store_id: &ActorId,
    tmg_id: &TamagotchiId,
) -> BTreeSet<AttributeId> {
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
    let (random, _) = exec::random(random_input)
        .expect("Error in getting random number");
    random[0] % 2
}





