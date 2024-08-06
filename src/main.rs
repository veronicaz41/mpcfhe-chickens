use itertools::Itertools;
use phantom_zone::FheUint8;
use phantom_zone::*;

use rand::{thread_rng, RngCore};

fn play(
    encrypted_constants: &[FheUint8],
    board: &[(FheUint8, FheUint8)],
    player0_action: &FheUint8,
    player1_action: &FheUint8,
    player2_action: &FheUint8,
    player3_action: &FheUint8,
) -> FheUint8 {
    // To compile from c
    return player0_action.clone();
}

// fn u64_to_binary<const N: usize>(v: u64) -> [bool; N] {
//     assert!((v as u128) < 2u128.pow(N as u32));
//     let mut result = [false; N];
//     for i in 0..N {
//         if (v >> i) & 1 == 1 {
//             result[i] = true;
//         }
//     }
//     result
// }

#[derive(Copy, Clone)]
enum Action {
    MoveUp = 0,
    MoveDown,
    MoveLeft,
    MoveRight,
    LayEgg,
    PickupEgg,
}

const BOARD_DIMS: u8 = 4;
const BOARD_SIZE: usize = (BOARD_DIMS as usize) * (BOARD_DIMS as usize);
const N_PLAYERS: usize = 4;
const MAX_ACTIONS: u8 = 5;

fn main() {
    set_parameter_set(ParameterSelector::NonInteractiveLTE4Party);

    // set application's common reference seed
    let mut seed = [0u8; 32];
    thread_rng().fill_bytes(&mut seed);
    set_common_reference_seed(seed);

    let no_of_parties = N_PLAYERS;

    // Client side //

    // Generate client keys
    let cks = (0..no_of_parties).map(|_| gen_client_key()).collect_vec();

    // Clients independently generate their server key shares
    let now = std::time::Instant::now();
    let server_key_shares = cks
        .iter()
        .enumerate()
        .map(|(id, k)| gen_server_key_share(id, no_of_parties, k))
        .collect_vec();
    println!("Clients server key share gen time: {:?}", now.elapsed());

    // Hack: client 0 encrypts constants
    let now = std::time::Instant::now();
    let mut range: Vec<u8> = (0..MAX_ACTIONS).collect();
    range.push(BOARD_DIMS);
    let constants_enc = cks[0].encrypt(range.as_slice());

    // starting coordinates for each player
    let starting_coords = vec![(0u8, 1u8), (1u8, 0u8), (0u8, 0u8), (1u8, 1u8)];
    // each client encrypted their private input which is their initial coordinates
    let staring_coords_enc = cks
        .iter()
        .enumerate()
        .map(|(id, k)| {
            let (x, y) = starting_coords[id];
            k.encrypt(vec![x, y].as_slice())
        })
        .collect_vec();
    println!("Client encrypt private inputs time: {:?}", now.elapsed());

    // Each client uploads their server key shares and encrypted private inputs to
    // the server in a single shot message.

    // Server side //

    // Server receives server key shares from each client and proceeds to aggregate
    // them to produce the server key. After this point, server can use the server
    // key to evaluate any arbitrary function on encrypted private inputs from
    // the fixed set of clients

    // aggregate server shares and generate the server key
    let now = std::time::Instant::now();
    let server_key = aggregate_server_key_shares(&server_key_shares);
    server_key.set_server_key();
    println!("Server key gen time: {:?}", now.elapsed());

    // Server proceeds to extract private constants sent by client 0
    let now = std::time::Instant::now();
    let encrypted_constants = constants_enc
        .unseed::<Vec<Vec<u64>>>()
        .key_switch(0)
        .extract_all();
    println!("Key switch time: {:?}", now.elapsed());

    // Server proceeds to extract private starting coordinates sent by clients
    let now = std::time::Instant::now();
    let encrypted_starting_coords = staring_coords_enc
        .iter()
        .enumerate()
        .map(|(id, enc)| {
            let (encrypted_x, encrypted_y) = {
                let mut tmp = enc.unseed::<Vec<Vec<u64>>>().key_switch(id).extract_all();
                (tmp.swap_remove(0), tmp.swap_remove(0))
            };
            return (encrypted_x, encrypted_y);
        })
        .collect_vec();
    println!("Key switch time: {:?}", now.elapsed());

    // Server setup board
    let mut server_players_coords = encrypted_starting_coords;
    // let mut server_eggs

    // Game loop //

    // client side //
    let now = std::time::Instant::now();
    let actions = vec![
        Action::MoveUp,
        Action::MoveDown,
        Action::LayEgg,
        Action::PickupEgg,
    ];
    let actions_enc = cks
        .iter()
        .enumerate()
        .map(|(id, k)| {
            let action = actions[id] as u8;
            k.encrypt(vec![action].as_slice())
        })
        .collect_vec();
    println!("Step 1: Client encrypt action: {:?}", now.elapsed());

    // server side //

    let now = std::time::Instant::now();
    let encrypted_actions = actions_enc
        .iter()
        .enumerate()
        .map(|(id, enc)| {
            let (encrypted_action) = {
                let mut tmp = enc.unseed::<Vec<Vec<u64>>>().key_switch(id).extract_all();
                (tmp.swap_remove(0))
            };
            return encrypted_action;
        })
        .collect_vec();
    println!("Step 1: Server key switch: {:?}", now.elapsed());

    // After extracting each client's private inputs, server proceeds to evaluate the circuit
    let server_players_coords = play(
        &encrypted_constants,
        &server_players_coords,
        &encrypted_actions[0],
        &encrypted_actions[1],
        &encrypted_actions[2],
        &encrypted_actions[3],
    );

    // client side //

    // After extracting each client's private inputs, server proceeds to evaluate
    // fibonacci_number
    // let now = std::time::Instant::now();
    // // let cts_out = fibonacci_number(&cts_0);
    // let cts_out;
    // println!("FHE circuit evaluation time: {:?}", now.elapsed());

    // // Server has finished running compute. Clients can proceed to decrypt the
    // // output ciphertext using multi-party decryption.

    // // Client side //

    // // each client produces decryption share
    // let dec_shares = cts_out
    //     .iter()
    //     .map(|ct| cks.iter().map(|k| k.gen_decryption_share(ct)).collect_vec())
    //     .collect_vec();

    // // With all decryption shares, clients can aggregate the shares and decrypt the
    // // ciphertext
    // let out_back = cts_out
    //     .iter()
    //     .zip(dec_shares.iter())
    //     .map(|(ct, dec_shares)| cks[0].aggregate_decryption_shares(ct, dec_shares))
    //     .collect_vec();

    // println!("Result: {:?}", out_back);
}
