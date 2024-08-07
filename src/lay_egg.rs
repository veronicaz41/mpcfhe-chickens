
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
    ((21, false, NOR2), &[Arg(1, 8), Arg(1, 9)]),
    ((22, false, NOR2), &[Arg(1, 10), Arg(1, 11)]),
    ((31, false, NOR2), &[Arg(1, 21), Arg(1, 22)]),
    ((32, false, NOR2), &[Arg(1, 16), Arg(1, 19)]),
    ((34, false, NOR2), &[Arg(1, 28), Arg(1, 31)]),
    ((35, false, NOR2), &[Arg(1, 25), Arg(1, 26)]),
    ((38, false, NOR2), &[Arg(1, 20), Arg(1, 23)]),
    ((39, false, NOR2), &[Arg(1, 17), Arg(1, 18)]),
    ((41, false, NOR2), &[Arg(1, 29), Arg(1, 30)]),
    ((42, false, NOR2), &[Arg(1, 24), Arg(1, 27)]),
    ((52, false, NOR2), &[Arg(1, 40), Arg(1, 41)]),
    ((53, false, NOR2), &[Arg(1, 42), Arg(1, 43)]),
    ((62, false, NOR2), &[Arg(1, 53), Arg(1, 54)]),
    ((63, false, NOR2), &[Arg(1, 48), Arg(1, 51)]),
    ((65, false, NOR2), &[Arg(1, 60), Arg(1, 63)]),
    ((66, false, NOR2), &[Arg(1, 57), Arg(1, 58)]),
    ((69, false, NOR2), &[Arg(1, 52), Arg(1, 55)]),
    ((70, false, NOR2), &[Arg(1, 49), Arg(1, 50)]),
    ((72, false, NOR2), &[Arg(1, 61), Arg(1, 62)]),
    ((73, false, NOR2), &[Arg(1, 56), Arg(1, 59)]),
];

static LEVEL_1: [((usize, bool, CellType), &[GateInput]); 20] = [
    ((17, false, NOR2), &[Arg(1, 12), Arg(1, 13)]),
    ((18, false, NOR2), &[Arg(1, 14), Arg(1, 15)]),
    ((20, false, NOR2), &[Arg(1, 2), Arg(1, 3)]),
    ((23, false, AND2), &[Tv(21), Tv(22)]),
    ((26, false, NOR2), &[Arg(1, 6), Arg(1, 7)]),
    ((27, false, NOR2), &[Arg(1, 4), Arg(1, 5)]),
    ((33, false, AND2), &[Tv(31), Tv(32)]),
    ((36, false, AND2), &[Tv(34), Tv(35)]),
    ((40, false, AND2), &[Tv(38), Tv(39)]),
    ((43, false, AND2), &[Tv(41), Tv(42)]),
    ((48, false, NOR2), &[Arg(1, 44), Arg(1, 45)]),
    ((49, false, NOR2), &[Arg(1, 46), Arg(1, 47)]),
    ((51, false, NOR2), &[Arg(1, 34), Arg(1, 35)]),
    ((54, false, AND2), &[Tv(52), Tv(53)]),
    ((57, false, NOR2), &[Arg(1, 36), Arg(1, 37)]),
    ((58, false, NOR2), &[Arg(1, 38), Arg(1, 39)]),
    ((64, false, AND2), &[Tv(62), Tv(63)]),
    ((67, false, AND2), &[Tv(65), Tv(66)]),
    ((71, false, AND2), &[Tv(69), Tv(70)]),
    ((74, false, AND2), &[Tv(72), Tv(73)]),
];

static LEVEL_2: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((108, false, INV), &[Arg(1, 33)]),
    ((0, false, INV), &[Arg(1, 1)]),
    ((19, false, AND2), &[Tv(17), Tv(18)]),
    ((24, false, AND2), &[Tv(20), Tv(23)]),
    ((28, false, AND2), &[Tv(26), Tv(27)]),
    ((37, false, AND2), &[Tv(33), Tv(36)]),
    ((44, false, AND2), &[Tv(40), Tv(43)]),
    ((50, false, AND2), &[Tv(48), Tv(49)]),
    ((55, false, AND2), &[Tv(51), Tv(54)]),
    ((59, false, AND2), &[Tv(57), Tv(58)]),
    ((68, false, AND2), &[Tv(64), Tv(67)]),
    ((75, false, AND2), &[Tv(71), Tv(74)]),
];

static LEVEL_3: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((107, false, INV), &[Arg(1, 32)]),
    ((109, false, INV), &[Arg(1, 0)]),
    ((25, false, AND2), &[Tv(19), Tv(24)]),
    ((29, false, AND2), &[Tv(0), Tv(28)]),
    ((45, false, AND2), &[Tv(37), Tv(44)]),
    ((56, false, AND2), &[Tv(50), Tv(55)]),
    ((60, false, AND2), &[Tv(108), Tv(59)]),
    ((76, false, AND2), &[Tv(68), Tv(75)]),
    ((83, false, AND2), &[Arg(1, 33), Tv(59)]),
    ((95, false, AND2), &[Arg(1, 1), Tv(28)]),
];

static LEVEL_4: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((30, false, AND2), &[Tv(25), Tv(29)]),
    ((46, false, AND2), &[Tv(109), Tv(45)]),
    ((61, false, AND2), &[Tv(56), Tv(60)]),
    ((77, false, AND2), &[Tv(107), Tv(76)]),
    ((80, false, AND2), &[Arg(1, 32), Tv(76)]),
    ((84, false, AND2), &[Tv(56), Tv(83)]),
    ((89, false, AND2), &[Arg(1, 0), Tv(45)]),
    ((96, false, AND2), &[Tv(25), Tv(95)]),
];

static LEVEL_5: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((47, false, AND2), &[Tv(30), Tv(46)]),
    ((78, false, AND2), &[Tv(61), Tv(77)]),
    ((81, false, AND2), &[Tv(61), Tv(80)]),
    ((85, false, AND2), &[Tv(77), Tv(84)]),
    ((87, false, AND2), &[Tv(80), Tv(84)]),
    ((90, false, AND2), &[Tv(30), Tv(89)]),
    ((97, false, AND2), &[Tv(46), Tv(96)]),
    ((102, false, AND2), &[Tv(89), Tv(96)]),
];

static LEVEL_6: [((usize, bool, CellType), &[GateInput]); 32] = [
    ((1, false, INV), &[Arg(0, 0)]),
    ((2, false, INV), &[Arg(0, 1)]),
    ((3, false, INV), &[Arg(0, 2)]),
    ((4, false, INV), &[Arg(0, 3)]),
    ((5, false, INV), &[Arg(0, 4)]),
    ((6, false, INV), &[Arg(0, 5)]),
    ((7, false, INV), &[Arg(0, 6)]),
    ((8, false, INV), &[Arg(0, 7)]),
    ((9, false, INV), &[Arg(0, 8)]),
    ((10, false, INV), &[Arg(0, 9)]),
    ((11, false, INV), &[Arg(0, 10)]),
    ((12, false, INV), &[Arg(0, 11)]),
    ((13, false, INV), &[Arg(0, 12)]),
    ((14, false, INV), &[Arg(0, 13)]),
    ((15, false, INV), &[Arg(0, 14)]),
    ((16, false, INV), &[Arg(0, 15)]),
    ((79, false, NAND2), &[Tv(47), Tv(78)]),
    ((82, false, NAND2), &[Tv(47), Tv(81)]),
    ((86, false, NAND2), &[Tv(47), Tv(85)]),
    ((88, false, NAND2), &[Tv(47), Tv(87)]),
    ((91, false, NAND2), &[Tv(78), Tv(90)]),
    ((92, false, NAND2), &[Tv(81), Tv(90)]),
    ((93, false, NAND2), &[Tv(85), Tv(90)]),
    ((94, false, NAND2), &[Tv(87), Tv(90)]),
    ((98, false, NAND2), &[Tv(78), Tv(97)]),
    ((99, false, NAND2), &[Tv(81), Tv(97)]),
    ((100, false, NAND2), &[Tv(85), Tv(97)]),
    ((101, false, NAND2), &[Tv(87), Tv(97)]),
    ((103, false, NAND2), &[Tv(78), Tv(102)]),
    ((104, false, NAND2), &[Tv(81), Tv(102)]),
    ((105, false, NAND2), &[Tv(85), Tv(102)]),
    ((106, false, NAND2), &[Tv(87), Tv(102)]),
];

static LEVEL_7: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((0, true, NAND2), &[Tv(1), Tv(79)]),
    ((1, true, NAND2), &[Tv(2), Tv(82)]),
    ((2, true, NAND2), &[Tv(3), Tv(86)]),
    ((3, true, NAND2), &[Tv(4), Tv(88)]),
    ((4, true, NAND2), &[Tv(5), Tv(91)]),
    ((5, true, NAND2), &[Tv(6), Tv(92)]),
    ((6, true, NAND2), &[Tv(7), Tv(93)]),
    ((7, true, NAND2), &[Tv(8), Tv(94)]),
    ((8, true, NAND2), &[Tv(9), Tv(98)]),
    ((9, true, NAND2), &[Tv(10), Tv(99)]),
    ((10, true, NAND2), &[Tv(11), Tv(100)]),
    ((11, true, NAND2), &[Tv(12), Tv(101)]),
    ((12, true, NAND2), &[Tv(13), Tv(103)]),
    ((13, true, NAND2), &[Tv(14), Tv(104)]),
    ((14, true, NAND2), &[Tv(15), Tv(105)]),
    ((15, true, NAND2), &[Tv(16), Tv(106)]),
];

static PRUNE_1: [usize; 20] = [
  22,
  53,
  39,
  70,
  32,
  63,
  73,
  35,
  42,
  66,
  21,
  52,
  69,
  38,
  31,
  62,
  41,
  72,
  34,
  65,
];

static PRUNE_2: [usize; 20] = [
  51,
  20,
  54,
  23,
  33,
  71,
  64,
  40,
  26,
  57,
  36,
  67,
  43,
  74,
  18,
  49,
  17,
  48,
  27,
  58,
];

static PRUNE_5: [usize; 8] = [
  61,
  30,
  84,
  46,
  77,
  80,
  89,
  96,
];

static PRUNE_7: [usize; 32] = [
  3,
  13,
  82,
  106,
  92,
  99,
  6,
  16,
  2,
  9,
  88,
  98,
  5,
  12,
  105,
  91,
  101,
  15,
  1,
  94,
  8,
  104,
  4,
  11,
  7,
  93,
  14,
  100,
  86,
  10,
  103,
  79,
];

static PRUNE_3: [usize; 12] = [
  44,
  75,
  68,
  37,
  19,
  50,
  108,
  28,
  59,
  24,
  55,
  0,
];

static PRUNE_6: [usize; 8] = [
  78,
  47,
  85,
  102,
  81,
  87,
  90,
  97,
];

static PRUNE_4: [usize; 10] = [
  109,
  95,
  29,
  60,
  56,
  25,
  107,
  83,
  45,
  76,
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
    run_level(&mut temp_nodes, &LEVEL_6);
    prune(&mut temp_nodes, &PRUNE_6);
    run_level(&mut temp_nodes, &LEVEL_7);
    prune(&mut temp_nodes, &PRUNE_7);

            

                out.into_iter().map(|c| c.unwrap()).collect()
            }),
        )
        .unwrap()
}

