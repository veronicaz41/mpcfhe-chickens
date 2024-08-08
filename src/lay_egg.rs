
use rayon::prelude::*;
use std::collections::HashMap;

use phantom_zone::*;

type Ciphertext = FheBool;

enum GateInput {
    Arg(usize, usize), // arg + index
    Output(usize), // reuse of output wire
    Tv(usize),  // temp value
    Cst(bool),  // constant
}

use GateInput::*;

#[derive(PartialEq, Eq, Hash)]
enum CellType {
    AND2,
    NAND2,
    XOR2,
    XNOR2,
    OR2,
    NOR2,
    INV,
    // TODO: Add back MUX2
}

use CellType::*;


static LEVEL_0: [((usize, bool, CellType), &[GateInput]); 4] = [
    ((36, false, NOR2), &[Arg(1, 6), Arg(1, 7)]),
    ((37, false, NOR2), &[Arg(1, 4), Arg(1, 5)]),
    ((43, false, NOR2), &[Arg(1, 12), Arg(1, 13)]),
    ((44, false, NOR2), &[Arg(1, 14), Arg(1, 15)]),
];

static LEVEL_1: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((14, false, INV), &[Arg(1, 8)]),
    ((15, false, INV), &[Arg(1, 9)]),
    ((16, false, INV), &[Arg(1, 0)]),
    ((17, false, INV), &[Arg(1, 1)]),
    ((34, false, NOR2), &[Arg(1, 2), Arg(1, 3)]),
    ((38, false, AND2), &[Tv(36), Tv(37)]),
    ((41, false, NOR2), &[Arg(1, 10), Arg(1, 11)]),
    ((45, false, AND2), &[Tv(43), Tv(44)]),
];

static LEVEL_2: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((35, false, AND2), &[Tv(16), Tv(34)]),
    ((39, false, AND2), &[Tv(17), Tv(38)]),
    ((42, false, AND2), &[Tv(14), Tv(41)]),
    ((46, false, AND2), &[Tv(15), Tv(45)]),
    ((49, false, AND2), &[Arg(1, 8), Tv(41)]),
    ((52, false, AND2), &[Arg(1, 9), Tv(45)]),
    ((57, false, AND2), &[Arg(1, 0), Tv(34)]),
    ((3, false, AND2), &[Arg(1, 1), Tv(38)]),
];

static LEVEL_3: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((40, false, AND2), &[Tv(35), Tv(39)]),
    ((47, false, AND2), &[Tv(42), Tv(46)]),
    ((50, false, AND2), &[Tv(46), Tv(49)]),
    ((53, false, AND2), &[Tv(42), Tv(52)]),
    ((55, false, AND2), &[Tv(49), Tv(52)]),
    ((58, false, AND2), &[Tv(39), Tv(57)]),
    ((4, false, AND2), &[Tv(35), Tv(3)]),
    ((9, false, AND2), &[Tv(57), Tv(3)]),
];

static LEVEL_4: [((usize, bool, CellType), &[GateInput]); 32] = [
    ((18, false, INV), &[Arg(0, 0)]),
    ((19, false, INV), &[Arg(0, 1)]),
    ((20, false, INV), &[Arg(0, 2)]),
    ((21, false, INV), &[Arg(0, 3)]),
    ((22, false, INV), &[Arg(0, 4)]),
    ((23, false, INV), &[Arg(0, 5)]),
    ((24, false, INV), &[Arg(0, 6)]),
    ((25, false, INV), &[Arg(0, 7)]),
    ((26, false, INV), &[Arg(0, 8)]),
    ((27, false, INV), &[Arg(0, 9)]),
    ((28, false, INV), &[Arg(0, 10)]),
    ((29, false, INV), &[Arg(0, 11)]),
    ((30, false, INV), &[Arg(0, 12)]),
    ((31, false, INV), &[Arg(0, 13)]),
    ((32, false, INV), &[Arg(0, 14)]),
    ((33, false, INV), &[Arg(0, 15)]),
    ((48, false, NAND2), &[Tv(40), Tv(47)]),
    ((51, false, NAND2), &[Tv(40), Tv(50)]),
    ((54, false, NAND2), &[Tv(40), Tv(53)]),
    ((56, false, NAND2), &[Tv(40), Tv(55)]),
    ((59, false, NAND2), &[Tv(47), Tv(58)]),
    ((0, false, NAND2), &[Tv(50), Tv(58)]),
    ((1, false, NAND2), &[Tv(53), Tv(58)]),
    ((2, false, NAND2), &[Tv(55), Tv(58)]),
    ((5, false, NAND2), &[Tv(47), Tv(4)]),
    ((6, false, NAND2), &[Tv(50), Tv(4)]),
    ((7, false, NAND2), &[Tv(53), Tv(4)]),
    ((8, false, NAND2), &[Tv(55), Tv(4)]),
    ((10, false, NAND2), &[Tv(47), Tv(9)]),
    ((11, false, NAND2), &[Tv(50), Tv(9)]),
    ((12, false, NAND2), &[Tv(53), Tv(9)]),
    ((13, false, NAND2), &[Tv(55), Tv(9)]),
];

static LEVEL_5: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((0, true, NAND2), &[Tv(18), Tv(48)]),
    ((1, true, NAND2), &[Tv(19), Tv(51)]),
    ((2, true, NAND2), &[Tv(20), Tv(54)]),
    ((3, true, NAND2), &[Tv(21), Tv(56)]),
    ((4, true, NAND2), &[Tv(22), Tv(59)]),
    ((5, true, NAND2), &[Tv(23), Tv(0)]),
    ((6, true, NAND2), &[Tv(24), Tv(1)]),
    ((7, true, NAND2), &[Tv(25), Tv(2)]),
    ((8, true, NAND2), &[Tv(26), Tv(5)]),
    ((9, true, NAND2), &[Tv(27), Tv(6)]),
    ((10, true, NAND2), &[Tv(28), Tv(7)]),
    ((11, true, NAND2), &[Tv(29), Tv(8)]),
    ((12, true, NAND2), &[Tv(30), Tv(10)]),
    ((13, true, NAND2), &[Tv(31), Tv(11)]),
    ((14, true, NAND2), &[Tv(32), Tv(12)]),
    ((15, true, NAND2), &[Tv(33), Tv(13)]),
];

static PRUNE_4: [usize; 8] = [
  58,
  55,
  4,
  53,
  50,
  47,
  40,
  9,
];

static PRUNE_3: [usize; 8] = [
  3,
  35,
  42,
  52,
  49,
  39,
  46,
  57,
];

static PRUNE_1: [usize; 4] = [
  36,
  43,
  44,
  37,
];

static PRUNE_2: [usize; 8] = [
  41,
  17,
  34,
  38,
  45,
  14,
  15,
  16,
];

static PRUNE_5: [usize; 32] = [
  10,
  48,
  27,
  7,
  24,
  0,
  31,
  11,
  21,
  59,
  28,
  56,
  1,
  32,
  8,
  18,
  25,
  22,
  29,
  19,
  26,
  5,
  12,
  23,
  54,
  33,
  2,
  13,
  20,
  51,
  30,
  6,
];

fn prune(temp_nodes: &mut HashMap<usize, Ciphertext>, temp_node_ids: &[usize]) {
  for x in temp_node_ids {
    temp_nodes.remove(&x);
  }
}

pub fn lay_egg(coords: &Vec<Ciphertext>, eggs: &Vec<Ciphertext>) -> Vec<Ciphertext> {
    let parameter_set = get_active_parameter_set();
    rayon::ThreadPoolBuilder::new()
        .build_scoped(
            |thread| {
                set_parameter_set(parameter_set);
                thread.run()
            },
            |pool| pool.install(|| {

                let args: &[&Vec<Ciphertext>] = &[eggs, coords];

                let mut temp_nodes = HashMap::new();
                let mut out = Vec::new();
                out.resize(16, None);

                let mut run_level = |
                temp_nodes: &mut HashMap<usize, Ciphertext>,
                tasks: &[((usize, bool, CellType), &[GateInput])]
                | {
                    let updates = tasks
                        .into_par_iter()
                        .map(|(k, task_args)| {
                            let (id, is_output, celltype) = k;
                            let task_args = task_args.into_iter()
                            .map(|arg| match arg {
                                Cst(false) => todo!(),
                                Cst(true) => todo!(),
                                Arg(pos, ndx) => &args[*pos][*ndx],
                                Tv(ndx) => &temp_nodes[ndx],
                                Output(ndx) => &out[*ndx]
                                            .as_ref()
                                            .expect(&format!("Output node {ndx} not found")),
                            }).collect::<Vec<_>>();

                            let gate_func = |args: &[&Ciphertext]| match celltype {
                                AND2 => args[0] & args[1],
                                NAND2 => args[0].nand(args[1]),
                                OR2 => args[0] | args[1],
                                NOR2 => args[0].nor(args[1]),
                                XOR2 => args[0] ^ args[1],
                                XNOR2 => args[0].xnor(args[1]),
                                INV => !args[0],
                            };
                            
                            ((*id, *is_output), gate_func(&task_args))
                        })
                        .collect::<Vec<_>>();
                    updates.into_iter().for_each(|(k, v)| {
                        let (index, is_output) = k;
                        if is_output {
                            out[index] = Some(v);
                        } else {
                            temp_nodes.insert(index, v);
                        }
                    });
                };

                run_level(&mut temp_nodes, &LEVEL_0);
    run_level(&mut temp_nodes, &LEVEL_1);
    prune(&mut temp_nodes, &PRUNE_1);
    run_level(&mut temp_nodes, &LEVEL_2);
    prune(&mut temp_nodes, &PRUNE_2);
    run_level(&mut temp_nodes, &LEVEL_3);
    prune(&mut temp_nodes, &PRUNE_3);
    run_level(&mut temp_nodes, &LEVEL_4);
    prune(&mut temp_nodes, &PRUNE_4);
    run_level(&mut temp_nodes, &LEVEL_5);
    prune(&mut temp_nodes, &PRUNE_5);

            

                out.into_iter().map(|c| c.unwrap()).collect()
            }),
        )
        .unwrap()
}

