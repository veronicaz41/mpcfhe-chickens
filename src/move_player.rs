
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


static LEVEL_0: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((9, false, AND2), &[Arg(0, 20), Arg(0, 21)]),
    ((10, false, AND2), &[Arg(0, 22), Arg(0, 23)]),
    ((12, false, AND2), &[Arg(0, 16), Arg(0, 19)]),
    ((13, false, AND2), &[Arg(0, 17), Arg(0, 18)]),
    ((15, false, AND2), &[Arg(0, 4), Arg(0, 7)]),
    ((16, false, AND2), &[Arg(0, 5), Arg(0, 6)]),
    ((18, false, OR2), &[Arg(0, 1), Arg(0, 0)]),
    ((19, false, AND2), &[Arg(0, 2), Arg(0, 3)]),
    ((21, false, AND2), &[Arg(0, 8), Arg(0, 9)]),
    ((22, false, AND2), &[Arg(0, 10), Arg(0, 11)]),
    ((24, false, AND2), &[Arg(0, 12), Arg(0, 13)]),
    ((25, false, AND2), &[Arg(0, 14), Arg(0, 15)]),
    ((27, false, AND2), &[Arg(0, 24), Arg(0, 27)]),
    ((32, false, AND2), &[Arg(0, 28), Arg(0, 30)]),
    ((33, false, AND2), &[Arg(0, 25), Arg(0, 26)]),
];

static LEVEL_1: [((usize, bool, CellType), &[GateInput]); 23] = [
    ((11, false, AND2), &[Tv(9), Tv(10)]),
    ((14, false, AND2), &[Tv(12), Tv(13)]),
    ((17, false, AND2), &[Tv(15), Tv(16)]),
    ((20, false, AND2), &[Tv(18), Tv(19)]),
    ((23, false, AND2), &[Tv(21), Tv(22)]),
    ((26, false, AND2), &[Tv(24), Tv(25)]),
    ((28, false, AND2), &[Arg(0, 29), Tv(27)]),
    ((34, false, AND2), &[Tv(32), Tv(33)]),
    ((81, false, AND2), &[Arg(0, 52), Arg(0, 53)]),
    ((82, false, AND2), &[Arg(0, 54), Arg(0, 55)]),
    ((84, false, AND2), &[Arg(0, 48), Arg(0, 49)]),
    ((85, false, AND2), &[Arg(0, 50), Arg(0, 51)]),
    ((88, false, AND2), &[Arg(0, 60), Arg(0, 61)]),
    ((90, false, AND2), &[Arg(0, 56), Arg(0, 57)]),
    ((91, false, AND2), &[Arg(0, 58), Arg(0, 59)]),
    ((95, false, AND2), &[Arg(0, 36), Arg(0, 37)]),
    ((96, false, AND2), &[Arg(0, 38), Arg(0, 39)]),
    ((98, false, OR2), &[Arg(0, 33), Arg(0, 32)]),
    ((99, false, AND2), &[Arg(0, 34), Arg(0, 35)]),
    ((102, false, AND2), &[Arg(0, 45), Arg(0, 46)]),
    ((103, false, AND2), &[Arg(0, 44), Arg(0, 47)]),
    ((105, false, AND2), &[Arg(0, 40), Arg(0, 41)]),
    ((106, false, AND2), &[Arg(0, 42), Arg(0, 43)]),
];

static LEVEL_2: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((29, false, AND2), &[Tv(11), Tv(28)]),
    ((30, false, AND2), &[Tv(17), Tv(23)]),
    ((35, false, AND2), &[Tv(14), Tv(34)]),
    ((36, false, AND2), &[Tv(20), Tv(26)]),
    ((83, false, AND2), &[Tv(81), Tv(82)]),
    ((86, false, AND2), &[Tv(84), Tv(85)]),
    ((89, false, AND2), &[Arg(0, 62), Tv(88)]),
    ((92, false, AND2), &[Tv(90), Tv(91)]),
    ((97, false, AND2), &[Tv(95), Tv(96)]),
    ((100, false, AND2), &[Tv(98), Tv(99)]),
    ((104, false, AND2), &[Tv(102), Tv(103)]),
    ((107, false, AND2), &[Tv(105), Tv(106)]),
];

static LEVEL_3: [((usize, bool, CellType), &[GateInput]); 7] = [
    ((0, false, INV), &[Arg(0, 1)]),
    ((31, false, AND2), &[Tv(29), Tv(30)]),
    ((37, false, AND2), &[Tv(35), Tv(36)]),
    ((87, false, AND2), &[Tv(83), Tv(86)]),
    ((93, false, AND2), &[Tv(89), Tv(92)]),
    ((101, false, AND2), &[Tv(97), Tv(100)]),
    ((108, false, AND2), &[Tv(104), Tv(107)]),
];

static LEVEL_4: [((usize, bool, CellType), &[GateInput]); 7] = [
    ((2, false, INV), &[Arg(0, 33)]),
    ((3, false, INV), &[Arg(1, 0)]),
    ((4, false, INV), &[Arg(0, 31)]),
    ((38, false, AND2), &[Tv(31), Tv(37)]),
    ((39, false, NAND2), &[Tv(0), Arg(0, 0)]),
    ((94, false, AND2), &[Tv(87), Tv(93)]),
    ((109, false, AND2), &[Tv(101), Tv(108)]),
];

static LEVEL_5: [((usize, bool, CellType), &[GateInput]); 9] = [
    ((5, false, INV), &[Arg(0, 63)]),
    ((40, false, AND2), &[Tv(3), Tv(39)]),
    ((41, false, XNOR2), &[Tv(4), Tv(38)]),
    ((43, false, NAND2), &[Arg(0, 1), Arg(0, 0)]),
    ((44, false, AND2), &[Arg(1, 0), Arg(0, 31)]),
    ((79, false, NAND2), &[Tv(2), Arg(0, 32)]),
    ((110, false, AND2), &[Tv(94), Tv(109)]),
    ((112, false, AND2), &[Arg(1, 1), Tv(3)]),
    ((115, false, NAND2), &[Arg(0, 33), Arg(0, 32)]),
];

static LEVEL_6: [((usize, bool, CellType), &[GateInput]); 7] = [
    ((1, false, INV), &[Arg(1, 1)]),
    ((42, false, NAND2), &[Tv(40), Tv(41)]),
    ((45, false, NAND2), &[Tv(43), Tv(44)]),
    ((111, false, XNOR2), &[Tv(5), Tv(110)]),
    ((113, false, AND2), &[Tv(79), Tv(112)]),
    ((116, false, AND2), &[Arg(1, 0), Tv(115)]),
    ((117, false, AND2), &[Arg(1, 1), Arg(0, 63)]),
];

static LEVEL_7: [((usize, bool, CellType), &[GateInput]); 6] = [
    ((6, false, XNOR2), &[Arg(0, 0), Arg(1, 0)]),
    ((46, false, NAND2), &[Tv(42), Tv(45)]),
    ((77, false, XNOR2), &[Arg(1, 0), Arg(0, 32)]),
    ((114, false, NAND2), &[Tv(111), Tv(113)]),
    ((118, false, NAND2), &[Tv(116), Tv(117)]),
    ((148, false, OR2), &[Tv(1), Tv(116)]),
];

static LEVEL_8: [((usize, bool, CellType), &[GateInput]); 64] = [
    ((7, false, AND2), &[Tv(1), Tv(6)]),
    ((8, false, NAND2), &[Arg(1, 1), Arg(0, 2)]),
    ((47, false, NAND2), &[Tv(1), Tv(46)]),
    ((48, false, NAND2), &[Arg(1, 1), Arg(0, 3)]),
    ((49, false, NAND2), &[Arg(1, 1), Arg(0, 4)]),
    ((50, false, NAND2), &[Arg(1, 1), Arg(0, 5)]),
    ((51, false, NAND2), &[Arg(1, 1), Arg(0, 6)]),
    ((52, false, NAND2), &[Arg(1, 1), Arg(0, 7)]),
    ((53, false, NAND2), &[Arg(1, 1), Arg(0, 8)]),
    ((54, false, NAND2), &[Arg(1, 1), Arg(0, 9)]),
    ((55, false, NAND2), &[Arg(1, 1), Arg(0, 10)]),
    ((56, false, NAND2), &[Arg(1, 1), Arg(0, 11)]),
    ((57, false, NAND2), &[Arg(1, 1), Arg(0, 12)]),
    ((58, false, NAND2), &[Arg(1, 1), Arg(0, 13)]),
    ((59, false, NAND2), &[Arg(1, 1), Arg(0, 14)]),
    ((60, false, NAND2), &[Arg(1, 1), Arg(0, 15)]),
    ((61, false, NAND2), &[Arg(1, 1), Arg(0, 16)]),
    ((62, false, NAND2), &[Arg(1, 1), Arg(0, 17)]),
    ((63, false, NAND2), &[Arg(1, 1), Arg(0, 18)]),
    ((64, false, NAND2), &[Arg(1, 1), Arg(0, 19)]),
    ((65, false, NAND2), &[Arg(1, 1), Arg(0, 20)]),
    ((66, false, NAND2), &[Arg(1, 1), Arg(0, 21)]),
    ((67, false, NAND2), &[Arg(1, 1), Arg(0, 22)]),
    ((68, false, NAND2), &[Arg(1, 1), Arg(0, 23)]),
    ((69, false, NAND2), &[Arg(1, 1), Arg(0, 24)]),
    ((70, false, NAND2), &[Arg(1, 1), Arg(0, 25)]),
    ((71, false, NAND2), &[Arg(1, 1), Arg(0, 26)]),
    ((72, false, NAND2), &[Arg(1, 1), Arg(0, 27)]),
    ((73, false, NAND2), &[Arg(1, 1), Arg(0, 28)]),
    ((74, false, NAND2), &[Arg(1, 1), Arg(0, 29)]),
    ((75, false, NAND2), &[Arg(1, 1), Arg(0, 30)]),
    ((76, false, NAND2), &[Arg(1, 1), Arg(0, 31)]),
    ((78, false, AND2), &[Arg(1, 1), Tv(77)]),
    ((80, false, NAND2), &[Tv(1), Arg(0, 34)]),
    ((119, false, AND2), &[Tv(114), Tv(118)]),
    ((120, false, NAND2), &[Tv(1), Arg(0, 35)]),
    ((121, false, NAND2), &[Tv(1), Arg(0, 36)]),
    ((122, false, NAND2), &[Tv(1), Arg(0, 37)]),
    ((123, false, NAND2), &[Tv(1), Arg(0, 38)]),
    ((124, false, NAND2), &[Tv(1), Arg(0, 39)]),
    ((125, false, NAND2), &[Tv(1), Arg(0, 40)]),
    ((126, false, NAND2), &[Tv(1), Arg(0, 41)]),
    ((127, false, NAND2), &[Tv(1), Arg(0, 42)]),
    ((128, false, NAND2), &[Tv(1), Arg(0, 43)]),
    ((129, false, NAND2), &[Tv(1), Arg(0, 44)]),
    ((130, false, NAND2), &[Tv(1), Arg(0, 45)]),
    ((131, false, NAND2), &[Tv(1), Arg(0, 46)]),
    ((132, false, NAND2), &[Tv(1), Arg(0, 47)]),
    ((133, false, NAND2), &[Tv(1), Arg(0, 48)]),
    ((134, false, NAND2), &[Tv(1), Arg(0, 49)]),
    ((135, false, NAND2), &[Tv(1), Arg(0, 50)]),
    ((136, false, NAND2), &[Tv(1), Arg(0, 51)]),
    ((137, false, NAND2), &[Tv(1), Arg(0, 52)]),
    ((138, false, NAND2), &[Tv(1), Arg(0, 53)]),
    ((139, false, NAND2), &[Tv(1), Arg(0, 54)]),
    ((140, false, NAND2), &[Tv(1), Arg(0, 55)]),
    ((141, false, NAND2), &[Tv(1), Arg(0, 56)]),
    ((142, false, NAND2), &[Tv(1), Arg(0, 57)]),
    ((143, false, NAND2), &[Tv(1), Arg(0, 58)]),
    ((144, false, NAND2), &[Tv(1), Arg(0, 59)]),
    ((145, false, NAND2), &[Tv(1), Arg(0, 60)]),
    ((146, false, NAND2), &[Tv(1), Arg(0, 61)]),
    ((147, false, NAND2), &[Tv(1), Arg(0, 62)]),
    ((149, false, NAND2), &[Arg(0, 63), Tv(148)]),
];

static LEVEL_9: [((usize, bool, CellType), &[GateInput]); 64] = [
    ((0, true, XNOR2), &[Arg(0, 0), Arg(1, 1)]),
    ((1, true, XNOR2), &[Tv(0), Tv(7)]),
    ((2, true, NAND2), &[Tv(8), Tv(47)]),
    ((3, true, NAND2), &[Tv(47), Tv(48)]),
    ((4, true, NAND2), &[Tv(47), Tv(49)]),
    ((5, true, NAND2), &[Tv(47), Tv(50)]),
    ((6, true, NAND2), &[Tv(47), Tv(51)]),
    ((7, true, NAND2), &[Tv(47), Tv(52)]),
    ((8, true, NAND2), &[Tv(47), Tv(53)]),
    ((9, true, NAND2), &[Tv(47), Tv(54)]),
    ((10, true, NAND2), &[Tv(47), Tv(55)]),
    ((11, true, NAND2), &[Tv(47), Tv(56)]),
    ((12, true, NAND2), &[Tv(47), Tv(57)]),
    ((13, true, NAND2), &[Tv(47), Tv(58)]),
    ((14, true, NAND2), &[Tv(47), Tv(59)]),
    ((15, true, NAND2), &[Tv(47), Tv(60)]),
    ((16, true, NAND2), &[Tv(47), Tv(61)]),
    ((17, true, NAND2), &[Tv(47), Tv(62)]),
    ((18, true, NAND2), &[Tv(47), Tv(63)]),
    ((19, true, NAND2), &[Tv(47), Tv(64)]),
    ((20, true, NAND2), &[Tv(47), Tv(65)]),
    ((21, true, NAND2), &[Tv(47), Tv(66)]),
    ((22, true, NAND2), &[Tv(47), Tv(67)]),
    ((23, true, NAND2), &[Tv(47), Tv(68)]),
    ((24, true, NAND2), &[Tv(47), Tv(69)]),
    ((25, true, NAND2), &[Tv(47), Tv(70)]),
    ((26, true, NAND2), &[Tv(47), Tv(71)]),
    ((27, true, NAND2), &[Tv(47), Tv(72)]),
    ((28, true, NAND2), &[Tv(47), Tv(73)]),
    ((29, true, NAND2), &[Tv(47), Tv(74)]),
    ((30, true, NAND2), &[Tv(47), Tv(75)]),
    ((31, true, NAND2), &[Tv(47), Tv(76)]),
    ((32, true, XOR2), &[Arg(1, 1), Arg(0, 32)]),
    ((33, true, XNOR2), &[Tv(2), Tv(78)]),
    ((34, true, NAND2), &[Tv(80), Tv(119)]),
    ((35, true, NAND2), &[Tv(119), Tv(120)]),
    ((36, true, NAND2), &[Tv(119), Tv(121)]),
    ((37, true, NAND2), &[Tv(119), Tv(122)]),
    ((38, true, NAND2), &[Tv(119), Tv(123)]),
    ((39, true, NAND2), &[Tv(119), Tv(124)]),
    ((40, true, NAND2), &[Tv(119), Tv(125)]),
    ((41, true, NAND2), &[Tv(119), Tv(126)]),
    ((42, true, NAND2), &[Tv(119), Tv(127)]),
    ((43, true, NAND2), &[Tv(119), Tv(128)]),
    ((44, true, NAND2), &[Tv(119), Tv(129)]),
    ((45, true, NAND2), &[Tv(119), Tv(130)]),
    ((46, true, NAND2), &[Tv(119), Tv(131)]),
    ((47, true, NAND2), &[Tv(119), Tv(132)]),
    ((48, true, NAND2), &[Tv(119), Tv(133)]),
    ((49, true, NAND2), &[Tv(119), Tv(134)]),
    ((50, true, NAND2), &[Tv(119), Tv(135)]),
    ((51, true, NAND2), &[Tv(119), Tv(136)]),
    ((52, true, NAND2), &[Tv(119), Tv(137)]),
    ((53, true, NAND2), &[Tv(119), Tv(138)]),
    ((54, true, NAND2), &[Tv(119), Tv(139)]),
    ((55, true, NAND2), &[Tv(119), Tv(140)]),
    ((56, true, NAND2), &[Tv(119), Tv(141)]),
    ((57, true, NAND2), &[Tv(119), Tv(142)]),
    ((58, true, NAND2), &[Tv(119), Tv(143)]),
    ((59, true, NAND2), &[Tv(119), Tv(144)]),
    ((60, true, NAND2), &[Tv(119), Tv(145)]),
    ((61, true, NAND2), &[Tv(119), Tv(146)]),
    ((62, true, NAND2), &[Tv(119), Tv(147)]),
    ((63, true, NAND2), &[Tv(114), Tv(149)]),
];

static PRUNE_8: [usize; 6] = [
  6,
  77,
  46,
  1,
  148,
  118,
];

static PRUNE_1: [usize; 15] = [
  9,
  15,
  12,
  21,
  18,
  32,
  27,
  10,
  24,
  33,
  16,
  13,
  22,
  19,
  25,
];

static PRUNE_4: [usize; 6] = [
  37,
  108,
  101,
  87,
  31,
  93,
];

static PRUNE_6: [usize; 9] = [
  40,
  43,
  41,
  44,
  115,
  5,
  112,
  79,
  110,
];

static PRUNE_9: [usize; 67] = [
  76,
  71,
  133,
  147,
  54,
  68,
  130,
  51,
  139,
  60,
  122,
  74,
  136,
  57,
  119,
  52,
  114,
  145,
  66,
  128,
  49,
  80,
  142,
  63,
  125,
  120,
  58,
  72,
  134,
  55,
  69,
  7,
  131,
  2,
  64,
  126,
  47,
  78,
  140,
  123,
  61,
  137,
  75,
  132,
  70,
  53,
  146,
  67,
  129,
  50,
  143,
  138,
  59,
  121,
  73,
  135,
  149,
  56,
  8,
  144,
  65,
  127,
  48,
  141,
  62,
  0,
  124,
];

static PRUNE_7: [usize; 6] = [
  116,
  113,
  111,
  117,
  42,
  45,
];

static PRUNE_3: [usize; 12] = [
  29,
  83,
  35,
  97,
  89,
  86,
  100,
  30,
  92,
  36,
  107,
  104,
];

static PRUNE_2: [usize; 23] = [
  14,
  102,
  23,
  85,
  99,
  20,
  82,
  91,
  105,
  88,
  26,
  103,
  95,
  106,
  84,
  98,
  81,
  28,
  90,
  11,
  96,
  34,
  17,
];

static PRUNE_5: [usize; 6] = [
  4,
  94,
  38,
  109,
  39,
  3,
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
                out.resize(64, None);

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
    run_level(&mut temp_nodes, &LEVEL_8);
    prune(&mut temp_nodes, &PRUNE_8);
    run_level(&mut temp_nodes, &LEVEL_9);
    prune(&mut temp_nodes, &PRUNE_9);

            

                out.into_iter().map(|c| c.unwrap()).collect()
            }),
        )
        .unwrap()
}

