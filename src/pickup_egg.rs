
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


static LEVEL_0: [((usize, bool, CellType), &[GateInput]); 20] = [
    ((5, false, NOR2), &[Arg(1, 8), Arg(1, 9)]),
    ((6, false, NOR2), &[Arg(1, 10), Arg(1, 11)]),
    ((15, false, NOR2), &[Arg(1, 21), Arg(1, 22)]),
    ((16, false, NOR2), &[Arg(1, 16), Arg(1, 19)]),
    ((18, false, NOR2), &[Arg(1, 28), Arg(1, 31)]),
    ((19, false, NOR2), &[Arg(1, 25), Arg(1, 26)]),
    ((22, false, NOR2), &[Arg(1, 20), Arg(1, 23)]),
    ((23, false, NOR2), &[Arg(1, 17), Arg(1, 18)]),
    ((25, false, NOR2), &[Arg(1, 29), Arg(1, 30)]),
    ((26, false, NOR2), &[Arg(1, 24), Arg(1, 27)]),
    ((36, false, NOR2), &[Arg(1, 40), Arg(1, 41)]),
    ((37, false, NOR2), &[Arg(1, 42), Arg(1, 43)]),
    ((46, false, NOR2), &[Arg(1, 53), Arg(1, 54)]),
    ((47, false, NOR2), &[Arg(1, 48), Arg(1, 51)]),
    ((49, false, NOR2), &[Arg(1, 60), Arg(1, 63)]),
    ((50, false, NOR2), &[Arg(1, 57), Arg(1, 58)]),
    ((53, false, NOR2), &[Arg(1, 52), Arg(1, 55)]),
    ((54, false, NOR2), &[Arg(1, 49), Arg(1, 50)]),
    ((56, false, NOR2), &[Arg(1, 61), Arg(1, 62)]),
    ((57, false, NOR2), &[Arg(1, 56), Arg(1, 59)]),
];

static LEVEL_1: [((usize, bool, CellType), &[GateInput]); 20] = [
    ((1, false, NOR2), &[Arg(1, 12), Arg(1, 13)]),
    ((2, false, NOR2), &[Arg(1, 14), Arg(1, 15)]),
    ((4, false, NOR2), &[Arg(1, 2), Arg(1, 3)]),
    ((7, false, AND2), &[Tv(5), Tv(6)]),
    ((10, false, NOR2), &[Arg(1, 6), Arg(1, 7)]),
    ((11, false, NOR2), &[Arg(1, 4), Arg(1, 5)]),
    ((17, false, AND2), &[Tv(15), Tv(16)]),
    ((20, false, AND2), &[Tv(18), Tv(19)]),
    ((24, false, AND2), &[Tv(22), Tv(23)]),
    ((27, false, AND2), &[Tv(25), Tv(26)]),
    ((32, false, NOR2), &[Arg(1, 44), Arg(1, 45)]),
    ((33, false, NOR2), &[Arg(1, 46), Arg(1, 47)]),
    ((35, false, NOR2), &[Arg(1, 34), Arg(1, 35)]),
    ((38, false, AND2), &[Tv(36), Tv(37)]),
    ((41, false, NOR2), &[Arg(1, 36), Arg(1, 37)]),
    ((42, false, NOR2), &[Arg(1, 38), Arg(1, 39)]),
    ((48, false, AND2), &[Tv(46), Tv(47)]),
    ((51, false, AND2), &[Tv(49), Tv(50)]),
    ((55, false, AND2), &[Tv(53), Tv(54)]),
    ((58, false, AND2), &[Tv(56), Tv(57)]),
];

static LEVEL_2: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((92, false, INV), &[Arg(1, 33)]),
    ((0, false, INV), &[Arg(1, 1)]),
    ((3, false, AND2), &[Tv(1), Tv(2)]),
    ((8, false, AND2), &[Tv(4), Tv(7)]),
    ((12, false, AND2), &[Tv(10), Tv(11)]),
    ((21, false, AND2), &[Tv(17), Tv(20)]),
    ((28, false, AND2), &[Tv(24), Tv(27)]),
    ((34, false, AND2), &[Tv(32), Tv(33)]),
    ((39, false, AND2), &[Tv(35), Tv(38)]),
    ((43, false, AND2), &[Tv(41), Tv(42)]),
    ((52, false, AND2), &[Tv(48), Tv(51)]),
    ((59, false, AND2), &[Tv(55), Tv(58)]),
];

static LEVEL_3: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((91, false, INV), &[Arg(1, 32)]),
    ((93, false, INV), &[Arg(1, 0)]),
    ((9, false, AND2), &[Tv(3), Tv(8)]),
    ((13, false, AND2), &[Tv(0), Tv(12)]),
    ((29, false, AND2), &[Tv(21), Tv(28)]),
    ((40, false, AND2), &[Tv(34), Tv(39)]),
    ((44, false, AND2), &[Tv(92), Tv(43)]),
    ((60, false, AND2), &[Tv(52), Tv(59)]),
    ((67, false, AND2), &[Arg(1, 33), Tv(43)]),
    ((79, false, AND2), &[Arg(1, 1), Tv(12)]),
];

static LEVEL_4: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((14, false, AND2), &[Tv(9), Tv(13)]),
    ((30, false, AND2), &[Tv(93), Tv(29)]),
    ((45, false, AND2), &[Tv(40), Tv(44)]),
    ((61, false, AND2), &[Tv(91), Tv(60)]),
    ((64, false, AND2), &[Arg(1, 32), Tv(60)]),
    ((68, false, AND2), &[Tv(40), Tv(67)]),
    ((73, false, AND2), &[Arg(1, 0), Tv(29)]),
    ((80, false, AND2), &[Tv(9), Tv(79)]),
];

static LEVEL_5: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((31, false, AND2), &[Tv(14), Tv(30)]),
    ((62, false, AND2), &[Tv(45), Tv(61)]),
    ((65, false, AND2), &[Tv(45), Tv(64)]),
    ((69, false, AND2), &[Tv(61), Tv(68)]),
    ((71, false, AND2), &[Tv(64), Tv(68)]),
    ((74, false, AND2), &[Tv(14), Tv(73)]),
    ((81, false, AND2), &[Tv(30), Tv(80)]),
    ((86, false, AND2), &[Tv(73), Tv(80)]),
];

static LEVEL_6: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((63, false, NAND2), &[Tv(31), Tv(62)]),
    ((66, false, NAND2), &[Tv(31), Tv(65)]),
    ((70, false, NAND2), &[Tv(31), Tv(69)]),
    ((72, false, NAND2), &[Tv(31), Tv(71)]),
    ((75, false, NAND2), &[Tv(62), Tv(74)]),
    ((76, false, NAND2), &[Tv(65), Tv(74)]),
    ((77, false, NAND2), &[Tv(69), Tv(74)]),
    ((78, false, NAND2), &[Tv(71), Tv(74)]),
    ((82, false, NAND2), &[Tv(62), Tv(81)]),
    ((83, false, NAND2), &[Tv(65), Tv(81)]),
    ((84, false, NAND2), &[Tv(69), Tv(81)]),
    ((85, false, NAND2), &[Tv(71), Tv(81)]),
    ((87, false, NAND2), &[Tv(62), Tv(86)]),
    ((88, false, NAND2), &[Tv(65), Tv(86)]),
    ((89, false, NAND2), &[Tv(69), Tv(86)]),
    ((90, false, NAND2), &[Tv(71), Tv(86)]),
];

static LEVEL_7: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((0, true, AND2), &[Arg(0, 0), Tv(63)]),
    ((1, true, AND2), &[Arg(0, 1), Tv(66)]),
    ((2, true, AND2), &[Arg(0, 2), Tv(70)]),
    ((3, true, AND2), &[Arg(0, 3), Tv(72)]),
    ((4, true, AND2), &[Arg(0, 4), Tv(75)]),
    ((5, true, AND2), &[Arg(0, 5), Tv(76)]),
    ((6, true, AND2), &[Arg(0, 6), Tv(77)]),
    ((7, true, AND2), &[Arg(0, 7), Tv(78)]),
    ((8, true, AND2), &[Arg(0, 8), Tv(82)]),
    ((9, true, AND2), &[Arg(0, 9), Tv(83)]),
    ((10, true, AND2), &[Arg(0, 10), Tv(84)]),
    ((11, true, AND2), &[Arg(0, 11), Tv(85)]),
    ((12, true, AND2), &[Arg(0, 12), Tv(87)]),
    ((13, true, AND2), &[Arg(0, 13), Tv(88)]),
    ((14, true, AND2), &[Arg(0, 14), Tv(89)]),
    ((15, true, AND2), &[Arg(0, 15), Tv(90)]),
];

static PRUNE_3: [usize; 12] = [
  3,
  34,
  0,
  28,
  59,
  21,
  52,
  8,
  39,
  43,
  12,
  92,
];

static PRUNE_4: [usize; 10] = [
  44,
  79,
  93,
  60,
  29,
  91,
  67,
  40,
  9,
  13,
];

static PRUNE_7: [usize; 16] = [
  75,
  89,
  72,
  76,
  83,
  66,
  90,
  63,
  87,
  77,
  70,
  84,
  88,
  85,
  78,
  82,
];

static PRUNE_6: [usize; 8] = [
  65,
  62,
  31,
  86,
  69,
  74,
  81,
  71,
];

static PRUNE_2: [usize; 20] = [
  27,
  58,
  17,
  48,
  10,
  41,
  55,
  24,
  7,
  38,
  11,
  42,
  4,
  35,
  32,
  1,
  33,
  2,
  20,
  51,
];

static PRUNE_5: [usize; 8] = [
  45,
  14,
  80,
  73,
  64,
  68,
  30,
  61,
];

static PRUNE_1: [usize; 20] = [
  49,
  25,
  18,
  56,
  15,
  46,
  22,
  53,
  36,
  5,
  57,
  26,
  50,
  19,
  23,
  54,
  16,
  47,
  6,
  37,
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
    run_level(&mut temp_nodes, &LEVEL_6);
    prune(&mut temp_nodes, &PRUNE_6);
    run_level(&mut temp_nodes, &LEVEL_7);
    prune(&mut temp_nodes, &PRUNE_7);

            

                out.into_iter().map(|c| c.unwrap()).collect()
            }),
        )
        .unwrap()
}

