
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


static LEVEL_0: [((usize, bool, CellType), &[GateInput]); 1] = [
    ((2, false, INV), &[Arg(1, 1)]),
];

static LEVEL_1: [((usize, bool, CellType), &[GateInput]); 3] = [
    ((1, false, INV), &[Arg(0, 0)]),
    ((4, false, NAND2), &[Tv(2), Arg(1, 0)]),
    ((5, false, OR2), &[Arg(1, 1), Arg(1, 0)]),
];

static LEVEL_2: [((usize, bool, CellType), &[GateInput]); 3] = [
    ((6, false, NAND2), &[Tv(1), Tv(5)]),
    ((7, false, NAND2), &[Arg(0, 0), Tv(4)]),
    ((9, false, XNOR2), &[Arg(1, 0), Arg(0, 8)]),
];

static LEVEL_3: [((usize, bool, CellType), &[GateInput]); 4] = [
    ((0, false, INV), &[Arg(0, 1)]),
    ((3, false, INV), &[Arg(0, 9)]),
    ((8, false, AND2), &[Tv(6), Tv(7)]),
    ((10, false, AND2), &[Arg(1, 1), Tv(9)]),
];

static LEVEL_4: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((0, true, XNOR2), &[Arg(0, 0), Arg(1, 1)]),
    ((1, true, XNOR2), &[Tv(0), Tv(8)]),
    ((2, true, AND2), &[Arg(1, 1), Arg(0, 2)]),
    ((3, true, AND2), &[Arg(1, 1), Arg(0, 3)]),
    ((4, true, AND2), &[Arg(1, 1), Arg(0, 4)]),
    ((5, true, AND2), &[Arg(1, 1), Arg(0, 5)]),
    ((6, true, AND2), &[Arg(1, 1), Arg(0, 6)]),
    ((7, true, AND2), &[Arg(1, 1), Arg(0, 7)]),
    ((8, true, XOR2), &[Arg(1, 1), Arg(0, 8)]),
    ((9, true, XNOR2), &[Tv(3), Tv(10)]),
    ((10, true, AND2), &[Tv(2), Arg(0, 10)]),
    ((11, true, AND2), &[Tv(2), Arg(0, 11)]),
    ((12, true, AND2), &[Tv(2), Arg(0, 12)]),
    ((13, true, AND2), &[Tv(2), Arg(0, 13)]),
    ((14, true, AND2), &[Tv(2), Arg(0, 14)]),
    ((15, true, AND2), &[Tv(2), Arg(0, 15)]),
];

static PRUNE_4: [usize; 5] = [
  10,
  8,
  0,
  2,
  3,
];

static PRUNE_2: [usize; 3] = [
  1,
  4,
  5,
];

static PRUNE_3: [usize; 3] = [
  7,
  6,
  9,
];

fn prune(temp_nodes: &mut HashMap<usize, Ciphertext>, temp_node_ids: &[usize]) {
  for x in temp_node_ids {
    temp_nodes.remove(&x);
  }
}

pub fn move_player(coords: &Vec<Ciphertext>, direction: &Vec<Ciphertext>) -> Vec<Ciphertext> {
    let parameter_set = get_active_parameter_set();
    rayon::ThreadPoolBuilder::new()
        .build_scoped(
            |thread| {
                set_parameter_set(parameter_set);
                thread.run()
            },
            |pool| pool.install(|| {

                let args: &[&Vec<Ciphertext>] = &[coords, direction];

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
    run_level(&mut temp_nodes, &LEVEL_2);
    prune(&mut temp_nodes, &PRUNE_2);
    run_level(&mut temp_nodes, &LEVEL_3);
    prune(&mut temp_nodes, &PRUNE_3);
    run_level(&mut temp_nodes, &LEVEL_4);
    prune(&mut temp_nodes, &PRUNE_4);

            

                out.into_iter().map(|c| c.unwrap()).collect()
            }),
        )
        .unwrap()
}

