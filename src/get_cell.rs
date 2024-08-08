
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


static LEVEL_0: [((usize, bool, CellType), &[GateInput]); 2] = [
    ((9, false, NOR2), &[Arg(2, 12), Arg(2, 13)]),
    ((10, false, NOR2), &[Arg(2, 14), Arg(2, 15)]),
];

static LEVEL_1: [((usize, bool, CellType), &[GateInput]); 6] = [
    ((0, false, INV), &[Arg(2, 9)]),
    ((1, false, INV), &[Arg(2, 8)]),
    ((11, false, AND2), &[Tv(9), Tv(10)]),
    ((13, false, NOR2), &[Arg(2, 10), Arg(2, 11)]),
    ((17, false, NOR2), &[Arg(2, 4), Arg(2, 5)]),
    ((18, false, NOR2), &[Arg(2, 6), Arg(2, 7)]),
];

static LEVEL_2: [((usize, bool, CellType), &[GateInput]); 6] = [
    ((12, false, AND2), &[Tv(1), Tv(11)]),
    ((14, false, AND2), &[Arg(2, 9), Tv(13)]),
    ((16, false, NOR2), &[Arg(2, 2), Arg(2, 3)]),
    ((19, false, AND2), &[Tv(17), Tv(18)]),
    ((32, false, AND2), &[Tv(0), Tv(13)]),
    ((36, false, AND2), &[Arg(2, 8), Tv(11)]),
];

static LEVEL_3: [((usize, bool, CellType), &[GateInput]); 6] = [
    ((2, false, INV), &[Arg(2, 1)]),
    ((15, false, AND2), &[Tv(12), Tv(14)]),
    ((20, false, AND2), &[Tv(16), Tv(19)]),
    ((33, false, AND2), &[Tv(12), Tv(32)]),
    ((37, false, AND2), &[Tv(14), Tv(36)]),
    ((40, false, AND2), &[Tv(32), Tv(36)]),
];

static LEVEL_4: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((3, false, INV), &[Arg(2, 0)]),
    ((4, false, INV), &[Arg(2, 5)]),
    ((5, false, INV), &[Arg(0, 5)]),
    ((6, false, INV), &[Arg(0, 21)]),
    ((7, false, INV), &[Arg(0, 37)]),
    ((8, false, INV), &[Arg(0, 53)]),
    ((21, false, AND2), &[Tv(2), Tv(20)]),
    ((39, false, NAND2), &[Arg(1, 14), Tv(15)]),
    ((41, false, NAND2), &[Arg(1, 13), Tv(40)]),
    ((45, false, NAND2), &[Arg(1, 8), Tv(33)]),
    ((47, false, NAND2), &[Arg(1, 9), Tv(40)]),
    ((48, false, NAND2), &[Arg(1, 11), Tv(37)]),
    ((54, false, NAND2), &[Arg(1, 3), Tv(37)]),
    ((55, false, NAND2), &[Arg(1, 0), Tv(33)]),
    ((60, false, NAND2), &[Arg(1, 4), Tv(33)]),
    ((61, false, NAND2), &[Arg(1, 5), Tv(40)]),
];

static LEVEL_5: [((usize, bool, CellType), &[GateInput]); 62] = [
    ((24, false, AND2), &[Arg(2, 0), Tv(21)]),
    ((26, false, AND2), &[Arg(2, 1), Tv(20)]),
    ((27, false, AND2), &[Tv(3), Arg(1, 10)]),
    ((34, false, NAND2), &[Arg(1, 12), Tv(33)]),
    ((38, false, NAND2), &[Arg(1, 15), Tv(37)]),
    ((42, false, AND2), &[Tv(39), Tv(41)]),
    ((46, false, AND2), &[Tv(3), Tv(45)]),
    ((49, false, AND2), &[Tv(47), Tv(48)]),
    ((53, false, NAND2), &[Arg(1, 1), Tv(40)]),
    ((56, false, AND2), &[Tv(54), Tv(55)]),
    ((59, false, NAND2), &[Arg(1, 7), Tv(37)]),
    ((62, false, AND2), &[Tv(60), Tv(61)]),
    ((67, false, XNOR2), &[Arg(2, 3), Arg(0, 3)]),
    ((68, false, XNOR2), &[Arg(2, 4), Arg(0, 4)]),
    ((70, false, XNOR2), &[Arg(2, 13), Arg(0, 13)]),
    ((71, false, XNOR2), &[Arg(2, 6), Arg(0, 6)]),
    ((72, false, XNOR2), &[Arg(2, 7), Arg(0, 7)]),
    ((73, false, XNOR2), &[Arg(2, 12), Arg(0, 12)]),
    ((75, false, XNOR2), &[Arg(2, 15), Arg(0, 15)]),
    ((79, false, NAND2), &[Tv(4), Arg(0, 5)]),
    ((80, false, NAND2), &[Arg(2, 5), Tv(5)]),
    ((82, false, XNOR2), &[Arg(2, 8), Arg(0, 8)]),
    ((90, false, XNOR2), &[Arg(2, 2), Arg(0, 2)]),
    ((92, false, XNOR2), &[Arg(2, 9), Arg(0, 9)]),
    ((99, false, XNOR2), &[Arg(2, 15), Arg(0, 31)]),
    ((100, false, NAND2), &[Arg(2, 5), Tv(6)]),
    ((101, false, XNOR2), &[Arg(2, 12), Arg(0, 28)]),
    ((103, false, XNOR2), &[Arg(2, 6), Arg(0, 22)]),
    ((105, false, XNOR2), &[Arg(2, 13), Arg(0, 29)]),
    ((107, false, NAND2), &[Tv(4), Arg(0, 21)]),
    ((108, false, XNOR2), &[Arg(2, 8), Arg(0, 24)]),
    ((109, false, XNOR2), &[Arg(2, 4), Arg(0, 20)]),
    ((112, false, XNOR2), &[Arg(2, 3), Arg(0, 19)]),
    ((114, false, XNOR2), &[Arg(2, 7), Arg(0, 23)]),
    ((122, false, XNOR2), &[Arg(2, 2), Arg(0, 18)]),
    ((124, false, XNOR2), &[Arg(2, 9), Arg(0, 25)]),
    ((131, false, NAND2), &[Arg(2, 5), Tv(7)]),
    ((133, false, XNOR2), &[Arg(2, 7), Arg(0, 39)]),
    ((134, false, XNOR2), &[Arg(2, 9), Arg(0, 41)]),
    ((135, false, XNOR2), &[Arg(2, 12), Arg(0, 44)]),
    ((136, false, NAND2), &[Tv(4), Arg(0, 37)]),
    ((138, false, XNOR2), &[Arg(2, 6), Arg(0, 38)]),
    ((139, false, XNOR2), &[Arg(2, 3), Arg(0, 35)]),
    ((140, false, XNOR2), &[Arg(2, 13), Arg(0, 45)]),
    ((141, false, XNOR2), &[Arg(2, 4), Arg(0, 36)]),
    ((142, false, XNOR2), &[Arg(2, 14), Arg(0, 46)]),
    ((144, false, XNOR2), &[Arg(2, 15), Arg(0, 47)]),
    ((145, false, XNOR2), &[Arg(2, 11), Arg(0, 43)]),
    ((146, false, XNOR2), &[Arg(2, 8), Arg(0, 40)]),
    ((155, false, XNOR2), &[Arg(2, 2), Arg(0, 34)]),
    ((164, false, XNOR2), &[Arg(2, 7), Arg(0, 55)]),
    ((165, false, XNOR2), &[Arg(2, 15), Arg(0, 63)]),
    ((166, false, NAND2), &[Tv(4), Arg(0, 53)]),
    ((167, false, NAND2), &[Arg(2, 5), Tv(8)]),
    ((168, false, XNOR2), &[Arg(2, 6), Arg(0, 54)]),
    ((170, false, XNOR2), &[Arg(2, 13), Arg(0, 61)]),
    ((171, false, XNOR2), &[Arg(2, 2), Arg(0, 50)]),
    ((172, false, XNOR2), &[Arg(2, 3), Arg(0, 51)]),
    ((174, false, XNOR2), &[Arg(2, 4), Arg(0, 52)]),
    ((176, false, XNOR2), &[Arg(2, 8), Arg(0, 56)]),
    ((179, false, XNOR2), &[Arg(2, 12), Arg(0, 60)]),
    ((188, false, XNOR2), &[Arg(2, 9), Arg(0, 57)]),
];

static LEVEL_6: [((usize, bool, CellType), &[GateInput]); 40] = [
    ((22, false, AND2), &[Tv(3), Tv(21)]),
    ((25, false, NAND2), &[Arg(1, 6), Tv(24)]),
    ((28, false, NAND2), &[Tv(26), Tv(27)]),
    ((35, false, AND2), &[Arg(2, 0), Tv(34)]),
    ((43, false, AND2), &[Tv(38), Tv(42)]),
    ((50, false, NAND2), &[Tv(46), Tv(49)]),
    ((57, false, NAND2), &[Tv(53), Tv(56)]),
    ((63, false, NAND2), &[Tv(59), Tv(62)]),
    ((76, false, XNOR2), &[Arg(2, 14), Arg(0, 14)]),
    ((77, false, XNOR2), &[Arg(2, 1), Arg(0, 1)]),
    ((81, false, AND2), &[Tv(67), Tv(71)]),
    ((83, false, AND2), &[Tv(68), Tv(73)]),
    ((85, false, AND2), &[Tv(70), Tv(72)]),
    ((91, false, AND2), &[Tv(79), Tv(90)]),
    ((93, false, AND2), &[Tv(82), Tv(92)]),
    ((95, false, AND2), &[Tv(75), Tv(80)]),
    ((106, false, XNOR2), &[Arg(2, 1), Arg(0, 17)]),
    ((110, false, XNOR2), &[Arg(2, 14), Arg(0, 30)]),
    ((113, false, AND2), &[Tv(103), Tv(112)]),
    ((115, false, AND2), &[Tv(101), Tv(109)]),
    ((117, false, AND2), &[Tv(105), Tv(114)]),
    ((123, false, AND2), &[Tv(107), Tv(122)]),
    ((125, false, AND2), &[Tv(108), Tv(124)]),
    ((127, false, AND2), &[Tv(99), Tv(100)]),
    ((132, false, XNOR2), &[Arg(2, 1), Arg(0, 33)]),
    ((148, false, AND2), &[Tv(138), Tv(139)]),
    ((149, false, AND2), &[Tv(135), Tv(141)]),
    ((151, false, AND2), &[Tv(133), Tv(140)]),
    ((152, false, AND2), &[Tv(142), Tv(145)]),
    ((156, false, AND2), &[Tv(136), Tv(155)]),
    ((157, false, AND2), &[Tv(134), Tv(146)]),
    ((159, false, AND2), &[Tv(131), Tv(144)]),
    ((173, false, XNOR2), &[Arg(2, 14), Arg(0, 62)]),
    ((177, false, XNOR2), &[Arg(2, 1), Arg(0, 49)]),
    ((178, false, AND2), &[Tv(168), Tv(172)]),
    ((180, false, AND2), &[Tv(174), Tv(179)]),
    ((182, false, AND2), &[Tv(164), Tv(170)]),
    ((187, false, AND2), &[Tv(166), Tv(171)]),
    ((189, false, AND2), &[Tv(176), Tv(188)]),
    ((191, false, AND2), &[Tv(165), Tv(167)]),
];

static LEVEL_7: [((usize, bool, CellType), &[GateInput]); 28] = [
    ((23, false, NAND2), &[Arg(1, 2), Tv(22)]),
    ((29, false, AND2), &[Tv(25), Tv(28)]),
    ((44, false, NAND2), &[Tv(35), Tv(43)]),
    ((51, false, AND2), &[Tv(26), Tv(50)]),
    ((58, false, NAND2), &[Tv(22), Tv(57)]),
    ((64, false, NAND2), &[Tv(24), Tv(63)]),
    ((69, false, XNOR2), &[Arg(2, 0), Arg(0, 0)]),
    ((74, false, XNOR2), &[Arg(2, 10), Arg(0, 10)]),
    ((84, false, AND2), &[Tv(81), Tv(83)]),
    ((86, false, AND2), &[Tv(76), Tv(85)]),
    ((94, false, AND2), &[Tv(91), Tv(93)]),
    ((96, false, AND2), &[Tv(77), Tv(95)]),
    ((104, false, XNOR2), &[Arg(2, 0), Arg(0, 16)]),
    ((111, false, XNOR2), &[Arg(2, 10), Arg(0, 26)]),
    ((116, false, AND2), &[Tv(113), Tv(115)]),
    ((118, false, AND2), &[Tv(110), Tv(117)]),
    ((126, false, AND2), &[Tv(123), Tv(125)]),
    ((128, false, AND2), &[Tv(106), Tv(127)]),
    ((150, false, AND2), &[Tv(148), Tv(149)]),
    ((153, false, AND2), &[Tv(151), Tv(152)]),
    ((158, false, AND2), &[Tv(156), Tv(157)]),
    ((160, false, AND2), &[Tv(132), Tv(159)]),
    ((163, false, XNOR2), &[Arg(2, 10), Arg(0, 58)]),
    ((169, false, XNOR2), &[Arg(2, 0), Arg(0, 48)]),
    ((181, false, AND2), &[Tv(178), Tv(180)]),
    ((183, false, AND2), &[Tv(173), Tv(182)]),
    ((190, false, AND2), &[Tv(187), Tv(189)]),
    ((192, false, AND2), &[Tv(177), Tv(191)]),
];

static LEVEL_8: [((usize, bool, CellType), &[GateInput]); 19] = [
    ((30, false, NAND2), &[Tv(23), Tv(29)]),
    ((52, false, NAND2), &[Tv(44), Tv(51)]),
    ((65, false, AND2), &[Tv(58), Tv(64)]),
    ((78, false, XNOR2), &[Arg(2, 11), Arg(0, 11)]),
    ((87, false, AND2), &[Tv(84), Tv(86)]),
    ((88, false, AND2), &[Tv(69), Tv(74)]),
    ((97, false, AND2), &[Tv(94), Tv(96)]),
    ((102, false, XNOR2), &[Arg(2, 11), Arg(0, 27)]),
    ((119, false, AND2), &[Tv(116), Tv(118)]),
    ((120, false, AND2), &[Tv(104), Tv(111)]),
    ((129, false, AND2), &[Tv(126), Tv(128)]),
    ((137, false, XNOR2), &[Arg(2, 0), Arg(0, 32)]),
    ((143, false, XNOR2), &[Arg(2, 10), Arg(0, 42)]),
    ((154, false, AND2), &[Tv(150), Tv(153)]),
    ((161, false, AND2), &[Tv(158), Tv(160)]),
    ((175, false, XNOR2), &[Arg(2, 11), Arg(0, 59)]),
    ((184, false, AND2), &[Tv(181), Tv(183)]),
    ((185, false, AND2), &[Tv(163), Tv(169)]),
    ((193, false, AND2), &[Tv(190), Tv(192)]),
];

static LEVEL_9: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((31, false, NAND2), &[Tv(15), Tv(30)]),
    ((66, false, AND2), &[Tv(52), Tv(65)]),
    ((89, false, AND2), &[Tv(87), Tv(88)]),
    ((98, false, AND2), &[Tv(78), Tv(97)]),
    ((121, false, AND2), &[Tv(119), Tv(120)]),
    ((130, false, AND2), &[Tv(102), Tv(129)]),
    ((147, false, AND2), &[Tv(137), Tv(143)]),
    ((162, false, AND2), &[Tv(154), Tv(161)]),
    ((186, false, AND2), &[Tv(184), Tv(185)]),
    ((194, false, AND2), &[Tv(175), Tv(193)]),
];

static LEVEL_10: [((usize, bool, CellType), &[GateInput]); 5] = [
    ((4, true, NAND2), &[Tv(31), Tv(66)]),
    ((0, true, AND2), &[Tv(89), Tv(98)]),
    ((1, true, AND2), &[Tv(121), Tv(130)]),
    ((2, true, AND2), &[Tv(147), Tv(162)]),
    ((3, true, AND2), &[Tv(186), Tv(194)]),
];

static PRUNE_5: [usize; 18] = [
  40,
  54,
  37,
  6,
  20,
  60,
  4,
  41,
  55,
  7,
  33,
  47,
  61,
  39,
  5,
  8,
  48,
  45,
];

static PRUNE_2: [usize; 6] = [
  18,
  1,
  13,
  11,
  17,
  0,
];

static PRUNE_6: [usize; 62] = [
  71,
  133,
  164,
  68,
  99,
  82,
  108,
  46,
  139,
  170,
  122,
  105,
  136,
  167,
  145,
  114,
  176,
  21,
  142,
  49,
  80,
  27,
  134,
  165,
  72,
  103,
  179,
  131,
  100,
  38,
  188,
  171,
  140,
  109,
  92,
  168,
  75,
  101,
  70,
  53,
  146,
  67,
  112,
  174,
  59,
  138,
  107,
  42,
  90,
  135,
  73,
  166,
  56,
  34,
  3,
  144,
  79,
  62,
  172,
  141,
  155,
  124,
];

static PRUNE_9: [usize; 20] = [
  102,
  161,
  15,
  184,
  88,
  119,
  52,
  97,
  120,
  193,
  78,
  185,
  30,
  154,
  137,
  129,
  143,
  87,
  65,
  175,
];

static PRUNE_1: [usize; 2] = [
  9,
  10,
];

static PRUNE_10: [usize; 10] = [
  186,
  147,
  130,
  66,
  89,
  162,
  194,
  98,
  121,
  31,
];

static PRUNE_7: [usize; 42] = [
  85,
  178,
  113,
  77,
  91,
  43,
  26,
  57,
  83,
  159,
  35,
  173,
  125,
  63,
  156,
  187,
  182,
  151,
  117,
  148,
  24,
  95,
  157,
  123,
  106,
  132,
  115,
  22,
  177,
  191,
  50,
  28,
  81,
  152,
  25,
  149,
  180,
  189,
  127,
  110,
  76,
  93,
];

static PRUNE_3: [usize; 6] = [
  12,
  32,
  16,
  36,
  19,
  14,
];

static PRUNE_8: [usize; 28] = [
  116,
  23,
  51,
  192,
  29,
  153,
  74,
  150,
  181,
  128,
  190,
  111,
  94,
  58,
  86,
  69,
  64,
  126,
  44,
  163,
  84,
  160,
  169,
  183,
  104,
  118,
  96,
  158,
];

static PRUNE_4: [usize; 1] = [
  2,
];

fn prune(temp_nodes: &mut HashMap<usize, Ciphertext>, temp_node_ids: &[usize]) {
  for x in temp_node_ids {
    temp_nodes.remove(&x);
  }
}

pub fn get_cell(coords: &Vec<Ciphertext>, eggs: &Vec<Ciphertext>, players: &Vec<Ciphertext>) -> Vec<Ciphertext> {
    let parameter_set = get_active_parameter_set();
    rayon::ThreadPoolBuilder::new()
        .build_scoped(
            |thread| {
                set_parameter_set(parameter_set);
                thread.run()
            },
            |pool| pool.install(|| {

                let args: &[&Vec<Ciphertext>] = &[players, eggs, coords];

                let mut temp_nodes = HashMap::new();
                let mut out = Vec::new();
                out.resize(5, None);

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
    run_level(&mut temp_nodes, &LEVEL_10);
    prune(&mut temp_nodes, &PRUNE_10);

            

                out.into_iter().map(|c| c.unwrap()).collect()
            }),
        )
        .unwrap()
}

