#![no_std]

#[allow(unused_imports)]
use gstd::{async_main, fmt, debug, exec, msg, prelude::*, ActorId};
use sharded_fungible_token_io::FTokenAction;
use sharded_fungible_token_io::FTokenEvent;
use sharded_fungible_token_io::LogicAction;
use store_io::{StoreAction, StoreEvent};
use tamagotchi_shop_io::{Tamagotchi, TmgEvent, TmgAction};


const HUNGER_PER_BLOCK: u64 = 1;
const BOREDOM_PER_BLOCK: u64 = 2;
const ENERGY_PER_BLOCK: u64 = 2;
const FILL_PER_FEED: u64 = 1000;
const FILL_PER_ENTERTAINMENT: u64 = 1000;
const FILL_PER_SLEEP: u64 = 1000;

static mut TAMAGOTCHI: Option<Tamagotchi> = None;

// TODO: 5️⃣ Add the `approve_tokens` function
async fn approve_tokens(tamagotchi: &mut Tamagotchi, account: &ActorId, amount: u128) {
    // ...
    let _approve_FT = msg::send_for_reply_as::<_, FTokenEvent>(
        tamagotchi.ft_contract_id.unwrap(),
        FTokenAction::Message {
            transaction_id: tamagotchi.transaction_id,
            payload: LogicAction::Approve {
                approved_account: account.clone(),
                amount,
            },
        },
        0,
        0,
    )
    .expect("Error in sending a message `FTokenAction::Message`")
    .await;
    // ...
}

async fn buy_attribute(store: &ActorId, attribute: u32) {
    let _result_buy = msg::send_for_reply_as::<_, StoreEvent>(
        store.clone(),
        StoreAction::BuyAttribute {
            attribute_id: attribute,
        },
        0,
        0,
    )
    .expect("Error in sending a message `StoreAction::BuyAttribute`")
    .await;
}




#[no_mangle]
extern fn init() {
    // TODO: 0️⃣ Copy the `init` function from the previous lesson and push changes to the master branch

    let tamagotchi = Tamagotchi {
        name:  msg::load().expect("Can't decode an init message"),
        date_of_birth: exec::block_timestamp(),
        owner: msg::source(),
        fed: 1000,
        fed_block: exec::block_height().into(),
        entertained: 1000,
        entertained_block: exec::block_height().into(),
        slept: 1000,
        slept_block: exec::block_height().into(),
        approved_account: None,
        ft_contract_id: Default::default(),
        transaction_id: Default::default(),
        approve_transaction: None,
    };
    debug!(
        "The Tamagotchi Program was initialized with name {:?}, birth date {:?}, owner: {:?}",
        tamagotchi.name, tamagotchi.date_of_birth, tamagotchi.owner
    );
    unsafe { TAMAGOTCHI = Some(tamagotchi) };
}


#[async_main]
async fn main() {
     // TODO: 0️⃣ Copy the `handle` function from the previous lesson and push changes to the master branch
     let _tamagotchi = unsafe {
        TAMAGOTCHI
            .as_mut()
            .expect("The contract is not initialized")
    };
    
    let name = &_tamagotchi.name;
    let current_time = exec::block_timestamp();
    let age = current_time.saturating_sub(_tamagotchi.date_of_birth);
    let action: TmgAction = msg::load().expect("Can't decode an action message");
    
    // 

  match action {
        TmgAction::Name => {
            msg::reply(TmgEvent::Name(name.to_string()), 0).expect("Error in sending name");
        }
        TmgAction::Age =>{
            msg::reply(TmgEvent::Age(age), 0).expect("Error in sending age");
           
        } 
        TmgAction::Feed => {
            let fed: u64 = _tamagotchi.fed;
            let fed_block: u64 = _tamagotchi.fed_block;
            let current_block: u64 = exec::block_height().into();
            let time_passed: u64 = current_block - fed_block;
            let hunger: u64 = time_passed * HUNGER_PER_BLOCK;
            let current_fed:u64  = fed - hunger;
            let new_fed:u64 = current_fed + FILL_PER_FEED;
           
            let new_fed_block: u64 = current_block;
            _tamagotchi.fed = new_fed;
            _tamagotchi.fed_block = new_fed_block;
            msg::reply(TmgEvent::Fed(new_fed), 0).expect("Error in sending fed");
        }
        TmgAction::Entertain => {
            let entertained: u64 = _tamagotchi.entertained;
           let entertained_block: u64 = _tamagotchi.entertained_block;
            let current_block: u64 = exec::block_height().into();
            let time_passed: u64 = current_block - entertained_block;
            let boredom: u64 = time_passed * BOREDOM_PER_BLOCK;
            let current_entertained: u64 = entertained - boredom;
            let new_entertained: u64 = current_entertained + FILL_PER_ENTERTAINMENT;

            let new_entertained_block: u64 = current_block;
            _tamagotchi.entertained = new_entertained;
            _tamagotchi.entertained_block = new_entertained_block;
            msg::reply(TmgEvent::Entertained(new_entertained), 0).expect("Error in sending entertained");
            
        }
        TmgAction::Sleep => {
            let slept: u64 = _tamagotchi.slept;
            let slept_block: u64 = _tamagotchi.slept_block;
            let current_block: u64 = exec::block_height().into();
            let time_passed: u64 = current_block - slept_block;
            let energy: u64 = time_passed * ENERGY_PER_BLOCK;
            let current_slept: u64 = slept - energy;
            let new_slept: u64 = current_slept + FILL_PER_SLEEP;

            let new_slept_block: u64 = current_block;
            _tamagotchi.slept = new_slept;
            _tamagotchi.slept_block = new_slept_block;
            msg::reply(TmgEvent::Slept(new_slept), 0).expect("Error in sending slept");
        
        }
        TmgAction::Transfer(new_owner) => {
            if _tamagotchi.owner == msg::source() {
                _tamagotchi.owner = new_owner;
                msg::reply(TmgEvent::Transfer(new_owner), 0).expect("Error in sending transfer");
                // debug!("Tamagotchi Transfered to account: {:?}", new_owner)
            }else{
                panic!("You are not the owner of this Tamagotchi");}
         
        }
        TmgAction::Approve(account) => {
            if _tamagotchi.owner == msg::source() {
            _tamagotchi.approved_account = Some(account);
            msg::reply(TmgEvent::Approve(account), 0).expect("Error in sending approve");
            // debug!("Approved account: {:?}", account)
            }else{
                panic!("You are not the allowed to approve accounts for this Tamagotchi");}
        }
        TmgAction::RevokeApproval => {
            if _tamagotchi.owner == msg::source() {
            _tamagotchi.approved_account = None;
            msg::reply(TmgEvent::RevokeApproval, 0).expect("Error in sending revoke approval");
            // debug!("Approved account: {:?} has been revoked", _tamagotchi.approved_account)
            }else{
                panic!("You are not the allowed to revoke approval for this Tamagotchi");}
            }
            // TODO; 6️⃣ Add handling new actions
        TmgAction::SetFTokenContract(contract) => {
            _tamagotchi.ft_contract_id = Some(contract);
            msg::reply(TmgEvent::FTokenContractSet, 0)
                .expect("Error in a reply `TmgEvent::FTokenContractSet`");
        }
        TmgAction::ApproveTokens { account, amount } => {
            approve_tokens(_tamagotchi, &account, amount).await;
            msg::reply(TmgEvent::TokensApproved { account, amount }, 0)
                .expect("Error in a reply `TmgEvent::TokensApproved`");
        }
        TmgAction::BuyAttribute { store_id, attribute_id } => {
            buy_attribute(&store_id, attribute_id).await;
            msg::reply(TmgEvent::AttributeBought(attribute_id), 0)
                .expect("Error in a reply `TmgEvent::AttributeBought`");
        }
    };

    

}

#[no_mangle]
extern fn state() {
    // TODO: 0️⃣ Copy the `handle` function from the previous lesson and push changes to the master branch
    let tamagotchi = unsafe {
        TAMAGOTCHI
            .as_ref()
            .expect("The contract is not initialized")
    };
    msg::reply(tamagotchi, 0).expect("Failed to share state");
}
