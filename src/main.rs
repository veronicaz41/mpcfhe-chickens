use itertools::Itertools;
use phantom_zone::*;

use rand::{thread_rng, RngCore};
mod get_cell;
mod lay_egg;
mod move_player;
mod pickup_egg;

fn u8_to_binary<const N: usize>(v: u8) -> [bool; N] {
    assert!((v as u16) < 2u16.pow(N as u32));
    let mut result = [false; N];
    for i in 0..N {
        if (v >> i) & 1 == 1 {
            result[i] = true;
        }
    }
    result
}

fn coords_to_binary<const N: usize>(x: u8, y: u8) -> [bool; N] {
    let mut result = [false; N];
    for i in 0..N / 2 {
        if (x >> i) & 1 == 1 {
            result[i] = true;
        }
    }
    for i in N / 2..N {
        if (y >> i) & 1 == 1 {
            result[i] = true;
        }
    }
    result
}

fn binary_to_coords<const N: usize>(coords: &[bool]) -> [u8; 2] {
    let mut x = 0u8;
    let mut y = 0u8;
    for i in (0..N / 2).rev() {
        x = (x << 1) + coords[i] as u8;
    }
    for i in (N / 2..N).rev() {
        y = (y << 1) + coords[i] as u8;
    }
    return [x, y];
}

#[derive(Copy, Clone)]
#[repr(u8)]
enum Direction {
    Up = 0,
    Down,
    Left,
    Right,
}

const N_PLAYERS: usize = 4;
const BOARD_DIM: usize = 4; // board size is BOARD_DIM * BOARD_DIM
const BOARD_SIZE: usize = BOARD_DIM * BOARD_DIM;

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

    // starting coordinates for each player
    let starting_coords = vec![(0u8, 0u8), (2u8, 0u8), (1u8, 1u8), (1u8, 1u8)];

    // each client encrypted their private input which is their initial coordinates
    let staring_coords_enc = cks
        .iter()
        .enumerate()
        .map(|(id, k)| {
            let coords = coords_to_binary::<64>(starting_coords[id].0, starting_coords[id].1);
            k.encrypt(coords.as_slice())
        })
        .collect_vec();
    println!("Client encrypt private inputs time: {:?}", now.elapsed());

    // client 0 encrypt the initial board for eggs
    let mut eggs = [false; BOARD_SIZE];
    let eggs_enc = cks[0].encrypt(eggs.as_slice());
    println!("Client 0 encrypt eggs: {:?}", now.elapsed());

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

    // Server proceeds to extract private starting coordinates sent by clients
    let now = std::time::Instant::now();
    let encrypted_starting_coords = staring_coords_enc
        .iter()
        .enumerate()
        .map(|(id, enc)| {
            return enc.unseed::<Vec<Vec<u64>>>().key_switch(id).extract_all();
        })
        .collect_vec();
    println!("Key switch time: {:?}", now.elapsed());

    // Server proceeds to extract private starging eggs state sent by clients
    let now = std::time::Instant::now();
    let encrypted_eggs = eggs_enc
        .unseed::<Vec<Vec<u64>>>()
        .key_switch(0)
        .extract_all();
    println!("Server key switch: {:?}", now.elapsed());

    // Server state
    let mut server_players_coords = encrypted_starting_coords.clone();
    let mut server_eggs = encrypted_eggs.clone();

    // Game loop //

    // -------------- Move player --------------//

    let now = std::time::Instant::now();
    // each player's move direction for this round
    let directions = vec![
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];
    for i in 1..no_of_parties {
        println!("Player i: {:?}", i);

        // client side //

        let direction = u8_to_binary::<8>(directions[i] as u8);
        let direction_enc = cks[i].encrypt(direction.as_slice());
        println!("Client encrypt direction: {:?}", now.elapsed());

        // client send player_id, the function to call "move_player" and direction_env to the server

        // server side //

        let now = std::time::Instant::now();
        let encrypted_direction = direction_enc
            .unseed::<Vec<Vec<u64>>>()
            .key_switch(i)
            .extract_all();
        println!("Server key switch: {:?}", now.elapsed());

        // After extracting client's private inputs, server proceeds to evaluate the circuit
        let now = std::time::Instant::now();
        let encrypted_new_coords =
            move_player::move_player(&server_players_coords[i], &encrypted_direction);
        println!("move_player circuit evaluation time: {:?}", now.elapsed());

        // update coordinates for player
        server_players_coords[i] = encrypted_new_coords.clone();

        // client side //

        // each client produces decryption share
        let dec_shares = encrypted_new_coords
            .iter()
            .map(|ct| cks.iter().map(|k| k.gen_decryption_share(ct)).collect_vec())
            .collect_vec();

        // With all decryption shares, clients can aggregate the shares and decrypt the
        // ciphertext
        let decrypted_new_coords = encrypted_new_coords
            .iter()
            .zip(dec_shares.iter())
            .map(|(ct, dec_shares)| cks[0].aggregate_decryption_shares(ct, dec_shares))
            .collect_vec();

        let new_coords = binary_to_coords::<64>(decrypted_new_coords.as_slice());
        println!("New coords: {:?}, {:?}", new_coords[0], new_coords[1]);
    }

    // -------------- Lay Egg --------------//

    // client side //

    // client send player_id, and the function to call "lay_egg"

    // server side //

    // After extracting client's private inputs, server proceeds to evaluate the circuit
    let now = std::time::Instant::now();
    let encrypted_new_eggs = lay_egg::lay_egg(&server_players_coords[0], &encrypted_eggs);
    println!("lay_egg circuit evaluation time: {:?}", now.elapsed());

    // Update eggs positions
    server_eggs = encrypted_new_eggs.clone();

    // client side //

    // each client produces decryption share
    let dec_shares = encrypted_new_eggs
        .iter()
        .map(|ct| cks.iter().map(|k| k.gen_decryption_share(ct)).collect_vec())
        .collect_vec();

    // With all decryption shares, clients can aggregate the shares and decrypt the
    // ciphertext
    let decrypted_new_eggs = encrypted_new_eggs
        .iter()
        .zip(dec_shares.iter())
        .map(|(ct, dec_shares)| cks[0].aggregate_decryption_shares(ct, dec_shares))
        .collect_vec();

    println!("New eggs: {:?}", decrypted_new_eggs);

    // -------------- Pickup Egg --------------//

    // client side //

    // client send player_id, and the function to call "lay_egg"

    // server side //

    // After extracting client's private inputs, server proceeds to evaluate the circuit
    let now = std::time::Instant::now();
    let encrypted_new_eggs = pickup_egg::pickup_egg(&server_players_coords[0], &server_eggs);
    println!("pickup_egg circuit evaluation time: {:?}", now.elapsed());

    // Update eggs positions
    server_eggs = encrypted_new_eggs.clone();

    // client side //

    // each client produces decryption share
    let dec_shares = encrypted_new_eggs
        .iter()
        .map(|ct| cks.iter().map(|k| k.gen_decryption_share(ct)).collect_vec())
        .collect_vec();

    // With all decryption shares, clients can aggregate the shares and decrypt the
    // ciphertext
    let decrypted_new_eggs = encrypted_new_eggs
        .iter()
        .zip(dec_shares.iter())
        .map(|(ct, dec_shares)| cks[0].aggregate_decryption_shares(ct, dec_shares))
        .collect_vec();

    println!("New eggs: {:?}", decrypted_new_eggs);

    // -------------- Get cell --------------//

    // client side //

    // client send player_id, and the function to call "get_cell"

    // server side //

    // After extracting client's private inputs, server proceeds to evaluate the circuit
    let now = std::time::Instant::now();
    let mut flattened_players_coords = server_players_coords[0].clone();
    for i in 1..no_of_parties {
        flattened_players_coords.append(&mut server_players_coords[i])
    }

    let encrypted_cell = get_cell::get_cell(
        &server_players_coords[0],
        &server_eggs,
        &flattened_players_coords,
    );
    println!("get_cell circuit evaluation time: {:?}", now.elapsed());

    // client side //

    // each client produces decryption share
    let dec_shares = encrypted_cell
        .iter()
        .map(|ct| cks.iter().map(|k| k.gen_decryption_share(ct)).collect_vec())
        .collect_vec();

    // With all decryption shares, clients can aggregate the shares and decrypt the
    // ciphertext
    let decrypted_cell = encrypted_cell
        .iter()
        .zip(dec_shares.iter())
        .map(|(ct, dec_shares)| cks[0].aggregate_decryption_shares(ct, dec_shares))
        .collect_vec();

    println!("Cell: {:?}", decrypted_cell);
}
