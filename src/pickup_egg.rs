
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
    ((11, false, NOR2), &[Arg(1, 6), Arg(1, 7)]),
    ((12, false, NOR2), &[Arg(1, 4), Arg(1, 5)]),
    ((18, false, NOR2), &[Arg(1, 12), Arg(1, 13)]),
    ((19, false, NOR2), &[Arg(1, 14), Arg(1, 15)]),
];

static LEVEL_1: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((5, false, INV), &[Arg(1, 8)]),
    ((6, false, INV), &[Arg(1, 9)]),
    ((7, false, INV), &[Arg(1, 0)]),
    ((8, false, INV), &[Arg(1, 1)]),
    ((9, false, NOR2), &[Arg(1, 2), Arg(1, 3)]),
    ((13, false, AND2), &[Tv(11), Tv(12)]),
    ((16, false, NOR2), &[Arg(1, 10), Arg(1, 11)]),
    ((20, false, AND2), &[Tv(18), Tv(19)]),
];

static LEVEL_2: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((10, false, AND2), &[Tv(7), Tv(9)]),
    ((14, false, AND2), &[Tv(8), Tv(13)]),
    ((17, false, AND2), &[Tv(5), Tv(16)]),
    ((21, false, AND2), &[Tv(6), Tv(20)]),
    ((24, false, AND2), &[Arg(1, 8), Tv(16)]),
    ((27, false, AND2), &[Arg(1, 9), Tv(20)]),
    ((32, false, AND2), &[Arg(1, 0), Tv(9)]),
    ((38, false, AND2), &[Arg(1, 1), Tv(13)]),
];

static LEVEL_3: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((15, false, AND2), &[Tv(10), Tv(14)]),
    ((22, false, AND2), &[Tv(17), Tv(21)]),
    ((25, false, AND2), &[Tv(21), Tv(24)]),
    ((28, false, AND2), &[Tv(17), Tv(27)]),
    ((30, false, AND2), &[Tv(24), Tv(27)]),
    ((33, false, AND2), &[Tv(14), Tv(32)]),
    ((39, false, AND2), &[Tv(10), Tv(38)]),
    ((0, false, AND2), &[Tv(32), Tv(38)]),
];

static LEVEL_4: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((23, false, NAND2), &[Tv(15), Tv(22)]),
    ((26, false, NAND2), &[Tv(15), Tv(25)]),
    ((29, false, NAND2), &[Tv(15), Tv(28)]),
    ((31, false, NAND2), &[Tv(15), Tv(30)]),
    ((34, false, NAND2), &[Tv(22), Tv(33)]),
    ((35, false, NAND2), &[Tv(25), Tv(33)]),
    ((36, false, NAND2), &[Tv(28), Tv(33)]),
    ((37, false, NAND2), &[Tv(30), Tv(33)]),
    ((40, false, NAND2), &[Tv(22), Tv(39)]),
    ((41, false, NAND2), &[Tv(25), Tv(39)]),
    ((42, false, NAND2), &[Tv(28), Tv(39)]),
    ((43, false, NAND2), &[Tv(30), Tv(39)]),
    ((1, false, NAND2), &[Tv(22), Tv(0)]),
    ((2, false, NAND2), &[Tv(25), Tv(0)]),
    ((3, false, NAND2), &[Tv(28), Tv(0)]),
    ((4, false, NAND2), &[Tv(30), Tv(0)]),
];

static LEVEL_5: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((0, true, AND2), &[Arg(0, 0), Tv(23)]),
    ((1, true, AND2), &[Arg(0, 1), Tv(26)]),
    ((2, true, AND2), &[Arg(0, 2), Tv(29)]),
    ((3, true, AND2), &[Arg(0, 3), Tv(31)]),
    ((4, true, AND2), &[Arg(0, 4), Tv(34)]),
    ((5, true, AND2), &[Arg(0, 5), Tv(35)]),
    ((6, true, AND2), &[Arg(0, 6), Tv(36)]),
    ((7, true, AND2), &[Arg(0, 7), Tv(37)]),
    ((8, true, AND2), &[Arg(0, 8), Tv(40)]),
    ((9, true, AND2), &[Arg(0, 9), Tv(41)]),
    ((10, true, AND2), &[Arg(0, 10), Tv(42)]),
    ((11, true, AND2), &[Arg(0, 11), Tv(43)]),
    ((12, true, AND2), &[Arg(0, 12), Tv(1)]),
    ((13, true, AND2), &[Arg(0, 13), Tv(2)]),
    ((14, true, AND2), &[Arg(0, 14), Tv(3)]),
    ((15, true, AND2), &[Arg(0, 15), Tv(4)]),
];

static PRUNE_2: [usize; 8] = [
  7,
  9,
  16,
  6,
  20,
  8,
  13,
  5,
];

static PRUNE_1: [usize; 4] = [
  11,
  18,
  12,
  19,
];

static PRUNE_3: [usize; 8] = [
  21,
  32,
  27,
  17,
  10,
  24,
  14,
  38,
];

static PRUNE_4: [usize; 8] = [
  28,
  33,
  25,
  30,
  15,
  39,
  22,
  0,
];

static PRUNE_5: [usize; 16] = [
  40,
  2,
  23,
  42,
  35,
  4,
  37,
  1,
  3,
  34,
  29,
  41,
  43,
  31,
  36,
  26,
];

fn prune(temp_nodes: &mut HashMap<usize, Ciphertext>, temp_node_ids: &[usize]) {
  for x in temp_node_ids {
    temp_nodes.remove(&x);
  }
}

pub fn pickup_egg(coords: &Vec<Ciphertext>, eggs: &Vec<Ciphertext>) -> Vec<Ciphertext> {
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

