#![no_std]
use gstd::{msg, prelude::*, ActorId};
use battle_io::{Battle, BattleAction, BattleState};

static mut BATTLE: Option<Battle> = None;

#[no_mangle]
extern fn init() {
    let battle = Battle {
        players: Vec::new(),
        state: BattleState::Registration,
        current_turn: 0,
        tmg_store_id: ActorId::default(),
        winner: ActorId::default(),
        steps: 0,
    };

    unsafe {
        BATTLE = Some(battle);
    }
}

#[gstd::async_main]
async fn main() {
    let action: BattleAction = msg::load().expect("Unable to decode BattleAction");
    let battle: &mut Battle = unsafe { BATTLE.get_or_insert(Default::default()) };
    match action {
        BattleAction::UpdateInfo => battle.update_info().await,
        BattleAction::Move => battle.make_move(),
        BattleAction::Register(tamagotchi_id) => battle.register(&tamagotchi_id).await,
    }
}