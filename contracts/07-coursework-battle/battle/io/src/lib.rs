


#[derive(Default)]
enum BattleState {
    #[default]
    Registration,
    Moves,
    Waiting,
    GameIsOver,
}


#[derive(Default)]
pub struct Battle {
    players: Vec<Player>,
    state: BattleState,
    current_turn: u8,
    tmg_store_id: ActorId,
    winner: ActorId,
    steps: u8,
}

#[derive(Default)]
pub struct Player {
   owner: ActorId,
   tmg_id: TamagotchiId,
   energy: u16,
   power: u16,
   attributes: BTreeSet<AttributeId>,
}


