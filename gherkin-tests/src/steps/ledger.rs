use std::cmp::Ordering;

use cucumber::{given, then, when};
use many_client::client::ledger::{SendArgs, TokenAmount};
use many_identity::Identity;
use num_bigint::BigUint;

use crate::params::Identifier;
use crate::world::World;

#[given(expr = "an identity {identifier}")]
fn setup_identity(world: &mut World, id: Identifier) {
    world.insert_identity(id);
}

#[given(expr = "a symbol {word}")]
fn setup_symbol(world: &mut World, symbol: String) {
    assert!(world.symbols().contains_key(&symbol));
}

#[given(expr = "{identifier} has {int} {word}")]
async fn id_has_x_symbols(world: &mut World, id: Identifier, amount: BigUint, symbol: String) {
    let amount: TokenAmount = amount.into();
    let faucet = world.spec_config().faucet_identity.clone();
    let identity = world.identity(&id).unwrap().clone();
    let symbol = *world.symbol(&symbol).unwrap();
    let current_balance = world.balance(identity.address(), symbol).await;
    let faucet_balance = world.balance(faucet.address(), symbol).await;

    assert_ne!(faucet_balance, TokenAmount::zero());

    match amount.cmp(&current_balance) {
        Ordering::Greater => {
            world
                .faucet_ledger_client()
                .send(SendArgs {
                    from: Some(faucet.address()),
                    to: identity.address(),
                    amount: amount.clone() - current_balance,
                    symbol,
                })
                .await
                .expect("Should have sent");
        }
        Ordering::Less => {
            world
                .ledger_client(identity.address())
                .send(SendArgs {
                    from: Some(identity.address()),
                    to: faucet.address(),
                    amount: current_balance - amount.clone(),
                    symbol,
                })
                .await
                .expect("Should have sent");
        }
        _ => {}
    }

    let new_balance = world.balance(identity.address(), symbol).await;
    assert_eq!(new_balance, amount);
}

#[when(expr = "{identifier} sends {int} {word} to {identifier}")]
async fn send_symbol(
    world: &mut World,
    sender_id: Identifier,
    amount: u32,
    symbol: String,
    receiver_id: Identifier,
) {
    let symbol = *world.symbol(&symbol).unwrap();
    let sender = world.identity(&sender_id).unwrap().address();
    let receiver = world.identity(&receiver_id).unwrap().address();
    world
        .ledger_client(sender)
        .send(SendArgs {
            from: Some(sender),
            to: receiver,
            amount: amount.into(),
            symbol,
        })
        .await
        .unwrap();
}

#[then(expr = "the balance of {identifier} should be {int} {word}")]
async fn balance_should_be(world: &mut World, id: Identifier, amount: BigUint, symbol: String) {
    let identity = world.identity(&id).unwrap().address();
    let symbol = *world.symbol(&symbol).unwrap();
    let balance = world.balance(identity, symbol).await;
    assert_eq!(balance, TokenAmount::from(amount));
}
