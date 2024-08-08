
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


static LEVEL_0: [((usize, bool, CellType), &[GateInput]); 5] = [
    ((249, false, INV), &[Arg(0, 2)]),
    ((252, false, INV), &[Arg(0, 10)]),
    ((16, false, NOR2), &[Arg(0, 1), Arg(0, 0)]),
    ((101, false, NOR2), &[Arg(0, 9), Arg(0, 8)]),
    ((163, false, AND2), &[Arg(0, 9), Arg(0, 8)]),
];

static LEVEL_1: [((usize, bool, CellType), &[GateInput]); 4] = [
    ((263, false, AND2), &[Arg(0, 1), Arg(0, 0)]),
    ((17, false, NOR2), &[Tv(249), Tv(16)]),
    ((102, false, NOR2), &[Tv(252), Tv(101)]),
    ((164, false, AND2), &[Arg(0, 10), Tv(163)]),
];

static LEVEL_2: [((usize, bool, CellType), &[GateInput]); 6] = [
    ((243, false, INV), &[Arg(0, 4)]),
    ((246, false, INV), &[Arg(0, 12)]),
    ((264, false, AND2), &[Arg(0, 2), Tv(263)]),
    ((20, false, NAND2), &[Arg(0, 3), Tv(17)]),
    ((104, false, NAND2), &[Arg(0, 11), Tv(102)]),
    ((167, false, AND2), &[Arg(0, 11), Tv(164)]),
];

static LEVEL_3: [((usize, bool, CellType), &[GateInput]); 6] = [
    ((250, false, INV), &[Arg(0, 5)]),
    ((253, false, INV), &[Arg(0, 13)]),
    ((266, false, AND2), &[Arg(0, 3), Tv(264)]),
    ((21, false, AND2), &[Tv(243), Tv(20)]),
    ((105, false, AND2), &[Tv(246), Tv(104)]),
    ((168, false, AND2), &[Arg(0, 12), Tv(167)]),
];

static LEVEL_4: [((usize, bool, CellType), &[GateInput]); 11] = [
    ((251, false, INV), &[Arg(0, 7)]),
    ((255, false, INV), &[Arg(0, 15)]),
    ((267, false, AND2), &[Arg(0, 4), Tv(266)]),
    ((269, false, NOR2), &[Arg(0, 5), Arg(0, 6)]),
    ((23, false, NOR2), &[Tv(250), Tv(21)]),
    ((24, false, AND2), &[Tv(250), Tv(21)]),
    ((107, false, NAND2), &[Arg(0, 14), Arg(0, 15)]),
    ((108, false, NOR2), &[Tv(253), Tv(105)]),
    ((110, false, AND2), &[Tv(253), Tv(105)]),
    ((170, false, NOR2), &[Arg(0, 13), Tv(168)]),
    ((171, false, AND2), &[Arg(0, 13), Tv(168)]),
];

static LEVEL_5: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((254, false, INV), &[Arg(0, 14)]),
    ((272, false, AND2), &[Arg(0, 5), Tv(267)]),
    ((273, false, XNOR2), &[Arg(0, 5), Tv(267)]),
    ((25, false, XNOR2), &[Arg(0, 5), Tv(21)]),
    ((27, false, OR2), &[Tv(251), Tv(269)]),
    ((32, false, AND2), &[Tv(251), Tv(23)]),
    ((35, false, NAND2), &[Arg(0, 7), Tv(24)]),
    ((109, false, NAND2), &[Tv(255), Tv(108)]),
    ((111, false, NAND2), &[Tv(107), Tv(110)]),
    ((113, false, XNOR2), &[Arg(0, 13), Tv(105)]),
    ((172, false, XNOR2), &[Tv(253), Tv(168)]),
    ((174, false, AND2), &[Tv(255), Tv(171)]),
    ((175, false, AND2), &[Tv(107), Tv(170)]),
];

static LEVEL_6: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((274, false, AND2), &[Arg(0, 7), Tv(273)]),
    ((279, false, NAND2), &[Arg(0, 6), Tv(272)]),
    ((26, false, NAND2), &[Tv(251), Tv(25)]),
    ((28, false, OR2), &[Tv(25), Tv(27)]),
    ((33, false, NAND2), &[Arg(0, 6), Tv(32)]),
    ((36, false, AND2), &[Arg(0, 6), Tv(35)]),
    ((112, false, AND2), &[Tv(109), Tv(111)]),
    ((114, false, NAND2), &[Arg(0, 15), Tv(113)]),
    ((119, false, NAND2), &[Tv(254), Tv(109)]),
    ((120, false, NAND2), &[Arg(0, 15), Tv(110)]),
    ((173, false, NAND2), &[Arg(0, 15), Tv(172)]),
    ((176, false, NOR2), &[Tv(174), Tv(175)]),
    ((181, false, NAND2), &[Arg(0, 15), Tv(170)]),
    ((182, false, XNOR2), &[Tv(254), Tv(174)]),
];

static LEVEL_7: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((270, false, NAND2), &[Arg(0, 7), Tv(269)]),
    ((280, false, NAND2), &[Tv(274), Tv(279)]),
    ((282, false, OR2), &[Arg(0, 7), Tv(279)]),
    ((22, false, XNOR2), &[Arg(0, 4), Tv(20)]),
    ((29, false, AND2), &[Tv(26), Tv(28)]),
    ((34, false, INV), &[Tv(33)]),
    ((37, false, NOR2), &[Tv(32), Tv(36)]),
    ((106, false, XNOR2), &[Arg(0, 12), Tv(104)]),
    ((115, false, NAND2), &[Tv(112), Tv(114)]),
    ((118, false, OR2), &[Tv(254), Tv(109)]),
    ((121, false, AND2), &[Tv(119), Tv(120)]),
    ((169, false, XNOR2), &[Arg(0, 12), Tv(167)]),
    ((177, false, NAND2), &[Tv(173), Tv(176)]),
    ((183, false, NAND2), &[Tv(181), Tv(182)]),
];

static LEVEL_8: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((271, false, OR2), &[Tv(267), Tv(270)]),
    ((275, false, XNOR2), &[Arg(0, 7), Tv(273)]),
    ((281, false, OR2), &[Arg(0, 6), Tv(272)]),
    ((283, false, AND2), &[Tv(280), Tv(282)]),
    ((31, false, NAND2), &[Tv(22), Tv(29)]),
    ((38, false, NOR2), &[Tv(34), Tv(37)]),
    ((40, false, OR2), &[Arg(0, 6), Tv(35)]),
    ((117, false, NAND2), &[Tv(106), Tv(115)]),
    ((123, false, AND2), &[Tv(118), Tv(121)]),
    ((125, false, OR2), &[Arg(0, 14), Tv(120)]),
    ((179, false, NAND2), &[Tv(169), Tv(177)]),
    ((180, false, NAND2), &[Arg(0, 14), Tv(174)]),
    ((184, false, INV), &[Tv(183)]),
    ((186, false, OR2), &[Arg(0, 14), Tv(181)]),
];

static LEVEL_9: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((268, false, XNOR2), &[Arg(0, 4), Tv(266)]),
    ((276, false, NAND2), &[Tv(271), Tv(275)]),
    ((284, false, NAND2), &[Tv(281), Tv(283)]),
    ((39, false, NAND2), &[Tv(31), Tv(38)]),
    ((41, false, AND2), &[Tv(33), Tv(40)]),
    ((122, false, NAND2), &[Tv(119), Tv(120)]),
    ((124, false, NAND2), &[Tv(117), Tv(123)]),
    ((126, false, AND2), &[Tv(118), Tv(125)]),
    ((185, false, NAND2), &[Tv(179), Tv(184)]),
    ((187, false, AND2), &[Tv(180), Tv(186)]),
];

static LEVEL_10: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((278, false, NAND2), &[Tv(268), Tv(276)]),
    ((285, false, INV), &[Tv(284)]),
    ((42, false, NAND2), &[Tv(39), Tv(41)]),
    ((44, false, NAND2), &[Tv(37), Tv(41)]),
    ((127, false, NAND2), &[Tv(124), Tv(126)]),
    ((129, false, NAND2), &[Tv(122), Tv(125)]),
    ((188, false, NAND2), &[Tv(185), Tv(187)]),
    ((190, false, NAND2), &[Tv(183), Tv(187)]),
];

static LEVEL_11: [((usize, bool, CellType), &[GateInput]); 11] = [
    ((286, false, NAND2), &[Tv(278), Tv(285)]),
    ((287, false, AND2), &[Tv(271), Tv(282)]),
    ((30, false, INV), &[Tv(29)]),
    ((43, false, NAND2), &[Tv(22), Tv(42)]),
    ((45, false, OR2), &[Tv(22), Tv(44)]),
    ((116, false, INV), &[Tv(115)]),
    ((128, false, NAND2), &[Tv(106), Tv(127)]),
    ((130, false, OR2), &[Tv(106), Tv(129)]),
    ((178, false, INV), &[Tv(177)]),
    ((189, false, NAND2), &[Tv(169), Tv(188)]),
    ((191, false, OR2), &[Tv(169), Tv(190)]),
];

static LEVEL_12: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((288, false, NAND2), &[Tv(286), Tv(287)]),
    ((290, false, NAND2), &[Tv(284), Tv(287)]),
    ((46, false, AND2), &[Tv(43), Tv(45)]),
    ((48, false, XNOR2), &[Arg(0, 3), Tv(17)]),
    ((49, false, NAND2), &[Tv(30), Tv(43)]),
    ((50, false, OR2), &[Tv(31), Tv(41)]),
    ((131, false, AND2), &[Tv(128), Tv(130)]),
    ((133, false, XNOR2), &[Arg(0, 11), Tv(102)]),
    ((134, false, NAND2), &[Tv(116), Tv(128)]),
    ((135, false, OR2), &[Tv(117), Tv(126)]),
    ((192, false, AND2), &[Tv(189), Tv(191)]),
    ((194, false, XNOR2), &[Arg(0, 11), Tv(164)]),
    ((195, false, OR2), &[Tv(179), Tv(187)]),
    ((196, false, NAND2), &[Tv(178), Tv(189)]),
];

static LEVEL_13: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((277, false, INV), &[Tv(276)]),
    ((289, false, NAND2), &[Tv(268), Tv(288)]),
    ((291, false, OR2), &[Tv(268), Tv(290)]),
    ((51, false, AND2), &[Tv(49), Tv(50)]),
    ((52, false, AND2), &[Tv(46), Tv(48)]),
    ((54, false, AND2), &[Tv(39), Tv(50)]),
    ((136, false, AND2), &[Tv(134), Tv(135)]),
    ((137, false, AND2), &[Tv(131), Tv(133)]),
    ((139, false, AND2), &[Tv(124), Tv(129)]),
    ((197, false, AND2), &[Tv(195), Tv(196)]),
    ((198, false, AND2), &[Tv(192), Tv(194)]),
    ((200, false, AND2), &[Tv(185), Tv(195)]),
];

static LEVEL_14: [((usize, bool, CellType), &[GateInput]); 11] = [
    ((292, false, AND2), &[Tv(289), Tv(291)]),
    ((294, false, XNOR2), &[Arg(0, 3), Tv(264)]),
    ((295, false, NAND2), &[Tv(277), Tv(289)]),
    ((296, false, OR2), &[Tv(278), Tv(287)]),
    ((53, false, OR2), &[Tv(51), Tv(52)]),
    ((55, false, NAND2), &[Tv(44), Tv(54)]),
    ((138, false, OR2), &[Tv(136), Tv(137)]),
    ((140, false, NAND2), &[Tv(124), Tv(129)]),
    ((142, false, NAND2), &[Tv(135), Tv(139)]),
    ((199, false, OR2), &[Tv(197), Tv(198)]),
    ((201, false, NAND2), &[Tv(190), Tv(200)]),
];

static LEVEL_15: [((usize, bool, CellType), &[GateInput]); 9] = [
    ((297, false, AND2), &[Tv(295), Tv(296)]),
    ((298, false, AND2), &[Tv(292), Tv(294)]),
    ((300, false, AND2), &[Tv(286), Tv(290)]),
    ((57, false, NAND2), &[Tv(53), Tv(55)]),
    ((62, false, NAND2), &[Tv(51), Tv(55)]),
    ((143, false, NAND2), &[Tv(138), Tv(142)]),
    ((148, false, NAND2), &[Tv(136), Tv(140)]),
    ((203, false, NAND2), &[Tv(199), Tv(201)]),
    ((208, false, NAND2), &[Tv(197), Tv(201)]),
];

static LEVEL_16: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((299, false, OR2), &[Tv(297), Tv(298)]),
    ((301, false, NAND2), &[Tv(286), Tv(290)]),
    ((303, false, NAND2), &[Tv(296), Tv(300)]),
    ((47, false, INV), &[Tv(46)]),
    ((56, false, INV), &[Tv(55)]),
    ((58, false, NAND2), &[Tv(48), Tv(57)]),
    ((68, false, OR2), &[Tv(48), Tv(62)]),
    ((132, false, INV), &[Tv(131)]),
    ((141, false, AND2), &[Tv(135), Tv(139)]),
    ((144, false, NAND2), &[Tv(133), Tv(143)]),
    ((154, false, OR2), &[Tv(133), Tv(148)]),
    ((193, false, INV), &[Tv(192)]),
    ((202, false, INV), &[Tv(201)]),
    ((204, false, NAND2), &[Tv(194), Tv(203)]),
    ((214, false, OR2), &[Tv(194), Tv(208)]),
];

static LEVEL_17: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((304, false, NAND2), &[Tv(299), Tv(303)]),
    ((3, false, NAND2), &[Tv(295), Tv(301)]),
    ((18, false, XNOR2), &[Tv(249), Tv(16)]),
    ((59, false, NAND2), &[Tv(47), Tv(58)]),
    ((60, false, NAND2), &[Tv(52), Tv(56)]),
    ((69, false, AND2), &[Tv(58), Tv(68)]),
    ((103, false, XNOR2), &[Tv(252), Tv(101)]),
    ((145, false, NAND2), &[Tv(132), Tv(144)]),
    ((146, false, NAND2), &[Tv(137), Tv(141)]),
    ((155, false, AND2), &[Tv(144), Tv(154)]),
    ((165, false, XNOR2), &[Arg(0, 10), Tv(163)]),
    ((205, false, NAND2), &[Tv(193), Tv(204)]),
    ((206, false, NAND2), &[Tv(198), Tv(202)]),
    ((215, false, AND2), &[Tv(204), Tv(214)]),
];

static LEVEL_18: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((293, false, INV), &[Tv(292)]),
    ((302, false, AND2), &[Tv(296), Tv(300)]),
    ((305, false, NAND2), &[Tv(294), Tv(304)]),
    ((8, false, OR2), &[Tv(294), Tv(3)]),
    ((61, false, AND2), &[Tv(59), Tv(60)]),
    ((63, false, AND2), &[Tv(53), Tv(62)]),
    ((71, false, AND2), &[Tv(18), Tv(69)]),
    ((147, false, AND2), &[Tv(145), Tv(146)]),
    ((149, false, AND2), &[Tv(138), Tv(148)]),
    ((157, false, AND2), &[Tv(103), Tv(155)]),
    ((207, false, AND2), &[Tv(205), Tv(206)]),
    ((209, false, AND2), &[Tv(199), Tv(208)]),
    ((217, false, AND2), &[Tv(165), Tv(215)]),
];

static LEVEL_19: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((265, false, XNOR2), &[Arg(0, 2), Tv(263)]),
    ((0, false, NAND2), &[Tv(293), Tv(305)]),
    ((1, false, NAND2), &[Tv(298), Tv(302)]),
    ((9, false, AND2), &[Tv(305), Tv(8)]),
    ((64, false, NAND2), &[Tv(60), Tv(63)]),
    ((72, false, OR2), &[Tv(61), Tv(71)]),
    ((150, false, NAND2), &[Tv(146), Tv(149)]),
    ((158, false, OR2), &[Tv(147), Tv(157)]),
    ((210, false, NAND2), &[Tv(206), Tv(209)]),
    ((218, false, OR2), &[Tv(207), Tv(217)]),
];

static LEVEL_20: [((usize, bool, CellType), &[GateInput]); 6] = [
    ((2, false, AND2), &[Tv(0), Tv(1)]),
    ((4, false, AND2), &[Tv(299), Tv(3)]),
    ((10, false, AND2), &[Tv(265), Tv(9)]),
    ((73, false, NAND2), &[Tv(64), Tv(72)]),
    ((159, false, NAND2), &[Tv(150), Tv(158)]),
    ((219, false, NAND2), &[Tv(210), Tv(218)]),
];

static LEVEL_21: [((usize, bool, CellType), &[GateInput]); 18] = [
    ((248, false, INV), &[Arg(1, 0)]),
    ((258, false, NOR2), &[Arg(1, 1), Arg(1, 0)]),
    ((5, false, NAND2), &[Tv(1), Tv(4)]),
    ((11, false, OR2), &[Tv(2), Tv(10)]),
    ((19, false, INV), &[Tv(18)]),
    ((65, false, INV), &[Tv(64)]),
    ((66, false, AND2), &[Tv(61), Tv(64)]),
    ((70, false, INV), &[Tv(69)]),
    ((74, false, NAND2), &[Tv(18), Tv(73)]),
    ((151, false, INV), &[Tv(150)]),
    ((152, false, NAND2), &[Tv(147), Tv(150)]),
    ((156, false, INV), &[Tv(155)]),
    ((160, false, NAND2), &[Tv(103), Tv(159)]),
    ((166, false, INV), &[Tv(165)]),
    ((211, false, INV), &[Tv(210)]),
    ((212, false, AND2), &[Tv(207), Tv(210)]),
    ((216, false, INV), &[Tv(215)]),
    ((220, false, NAND2), &[Tv(165), Tv(219)]),
];

static LEVEL_22: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((245, false, INV), &[Arg(1, 1)]),
    ((259, false, INV), &[Tv(258)]),
    ((6, false, NAND2), &[Tv(2), Tv(5)]),
    ((12, false, NAND2), &[Tv(5), Tv(11)]),
    ((67, false, NAND2), &[Tv(19), Tv(66)]),
    ((83, false, NAND2), &[Tv(65), Tv(71)]),
    ((84, false, NAND2), &[Tv(70), Tv(74)]),
    ((98, false, AND2), &[Arg(1, 1), Tv(248)]),
    ((153, false, OR2), &[Tv(103), Tv(152)]),
    ((213, false, NAND2), &[Tv(166), Tv(212)]),
    ((225, false, NAND2), &[Tv(156), Tv(160)]),
    ((226, false, NAND2), &[Tv(151), Tv(157)]),
    ((229, false, NAND2), &[Tv(216), Tv(220)]),
    ((230, false, NAND2), &[Tv(211), Tv(217)]),
];

static LEVEL_23: [((usize, bool, CellType), &[GateInput]); 18] = [
    ((247, false, INV), &[Arg(0, 8)]),
    ((256, false, NAND2), &[Tv(245), Arg(0, 0)]),
    ((262, false, AND2), &[Tv(245), Arg(1, 0)]),
    ((7, false, NOR2), &[Tv(265), Tv(6)]),
    ((13, false, AND2), &[Tv(265), Tv(12)]),
    ((75, false, NAND2), &[Tv(67), Tv(74)]),
    ((85, false, NAND2), &[Tv(83), Tv(84)]),
    ((88, false, AND2), &[Tv(6), Tv(11)]),
    ((92, false, NOR2), &[Tv(259), Tv(66)]),
    ((93, false, AND2), &[Tv(72), Tv(83)]),
    ((96, false, AND2), &[Arg(1, 1), Arg(1, 0)]),
    ((161, false, NAND2), &[Tv(153), Tv(160)]),
    ((221, false, NAND2), &[Tv(213), Tv(220)]),
    ((227, false, NAND2), &[Tv(225), Tv(226)]),
    ((231, false, NAND2), &[Tv(229), Tv(230)]),
    ((235, false, AND2), &[Tv(98), Tv(152)]),
    ((236, false, AND2), &[Tv(158), Tv(226)]),
    ((238, false, AND2), &[Tv(218), Tv(230)]),
];

static LEVEL_24: [((usize, bool, CellType), &[GateInput]); 21] = [
    ((257, false, NAND2), &[Tv(248), Arg(0, 0)]),
    ((260, false, NAND2), &[Tv(256), Tv(259)]),
    ((14, false, OR2), &[Tv(7), Tv(13)]),
    ((76, false, NAND2), &[Tv(258), Tv(75)]),
    ((77, false, NAND2), &[Arg(1, 1), Arg(0, 2)]),
    ((79, false, NAND2), &[Tv(9), Tv(13)]),
    ((80, false, XNOR2), &[Tv(9), Tv(13)]),
    ((82, false, NAND2), &[Arg(1, 1), Arg(0, 3)]),
    ((86, false, NAND2), &[Tv(258), Tv(85)]),
    ((89, false, AND2), &[Tv(262), Tv(88)]),
    ((91, false, NAND2), &[Arg(0, 4), Arg(1, 1)]),
    ((94, false, NAND2), &[Tv(92), Tv(93)]),
    ((97, false, NAND2), &[Arg(0, 8), Tv(96)]),
    ((99, false, NAND2), &[Tv(247), Tv(98)]),
    ((162, false, NAND2), &[Tv(98), Tv(161)]),
    ((222, false, NAND2), &[Tv(96), Tv(221)]),
    ((228, false, NAND2), &[Tv(98), Tv(227)]),
    ((232, false, NAND2), &[Tv(96), Tv(231)]),
    ((237, false, NAND2), &[Tv(235), Tv(236)]),
    ((239, false, NAND2), &[Tv(96), Tv(238)]),
    ((241, false, NAND2), &[Tv(245), Arg(0, 12)]),
];

static LEVEL_25: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((244, false, INV), &[Arg(0, 1)]),
    ((261, false, AND2), &[Tv(257), Tv(260)]),
    ((15, false, NAND2), &[Tv(262), Tv(14)]),
    ((78, false, AND2), &[Tv(76), Tv(77)]),
    ((81, false, NAND2), &[Tv(262), Tv(80)]),
    ((87, false, AND2), &[Tv(82), Tv(86)]),
    ((90, false, NAND2), &[Tv(79), Tv(89)]),
    ((95, false, AND2), &[Tv(91), Tv(94)]),
    ((100, false, AND2), &[Tv(97), Tv(99)]),
    ((223, false, NAND2), &[Tv(245), Arg(0, 10)]),
    ((224, false, AND2), &[Tv(162), Tv(222)]),
    ((233, false, NAND2), &[Tv(245), Arg(0, 11)]),
    ((234, false, AND2), &[Tv(228), Tv(232)]),
    ((240, false, OR2), &[Tv(212), Tv(239)]),
    ((242, false, AND2), &[Tv(237), Tv(241)]),
];

static LEVEL_26: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((0, true, XNOR2), &[Arg(1, 1), Arg(0, 0)]),
    ((1, true, XNOR2), &[Tv(244), Tv(261)]),
    ((2, true, NAND2), &[Tv(15), Tv(78)]),
    ((3, true, NAND2), &[Tv(81), Tv(87)]),
    ((4, true, NAND2), &[Tv(90), Tv(95)]),
    ((5, true, AND2), &[Arg(1, 1), Arg(0, 5)]),
    ((6, true, AND2), &[Arg(1, 1), Arg(0, 6)]),
    ((7, true, AND2), &[Arg(1, 1), Arg(0, 7)]),
    ((8, true, XOR2), &[Arg(1, 1), Arg(0, 8)]),
    ((9, true, XNOR2), &[Arg(0, 9), Tv(100)]),
    ((10, true, NAND2), &[Tv(223), Tv(224)]),
    ((11, true, NAND2), &[Tv(233), Tv(234)]),
    ((12, true, NAND2), &[Tv(240), Tv(242)]),
    ((13, true, AND2), &[Tv(245), Arg(0, 13)]),
    ((14, true, AND2), &[Tv(245), Arg(0, 14)]),
    ((15, true, AND2), &[Tv(245), Arg(0, 15)]),
];

static PRUNE_10: [usize; 4] = [
  122,
  37,
  125,
  183,
];

static PRUNE_6: [usize; 9] = [
  27,
  111,
  175,
  170,
  113,
  110,
  172,
  251,
  25,
];

static PRUNE_22: [usize; 14] = [
  71,
  151,
  156,
  5,
  157,
  103,
  19,
  70,
  216,
  2,
  217,
  65,
  166,
  211,
];

static PRUNE_16: [usize; 16] = [
  139,
  201,
  133,
  55,
  286,
  297,
  203,
  135,
  57,
  46,
  192,
  131,
  143,
  194,
  48,
  290,
];

static PRUNE_17: [usize; 23] = [
  303,
  252,
  101,
  163,
  202,
  16,
  141,
  56,
  214,
  198,
  249,
  68,
  193,
  58,
  204,
  52,
  132,
  301,
  154,
  137,
  295,
  47,
  144,
];

static PRUNE_11: [usize; 12] = [
  285,
  22,
  106,
  282,
  169,
  29,
  271,
  115,
  177,
  188,
  127,
  42,
];

static PRUNE_4: [usize; 1] = [
  250,
];

static PRUNE_23: [usize; 21] = [
  229,
  213,
  72,
  218,
  83,
  66,
  67,
  225,
  84,
  230,
  265,
  158,
  220,
  11,
  152,
  74,
  12,
  153,
  6,
  226,
  160,
];

static PRUNE_21: [usize; 17] = [
  150,
  10,
  207,
  4,
  61,
  219,
  73,
  215,
  147,
  210,
  69,
  165,
  18,
  1,
  159,
  64,
  155,
];

static PRUNE_9: [usize; 14] = [
  184,
  38,
  275,
  123,
  281,
  33,
  186,
  118,
  180,
  119,
  40,
  120,
  283,
  266,
];

static PRUNE_3: [usize; 2] = [
  246,
  243,
];

static PRUNE_7: [usize; 17] = [
  167,
  26,
  20,
  274,
  173,
  32,
  269,
  112,
  28,
  254,
  182,
  114,
  176,
  36,
  109,
  279,
  104,
];

static PRUNE_15: [usize; 5] = [
  140,
  197,
  136,
  51,
  142,
];

static PRUNE_19: [usize; 12] = [
  263,
  60,
  302,
  298,
  146,
  293,
  209,
  305,
  63,
  206,
  8,
  149,
];

static PRUNE_13: [usize; 10] = [
  195,
  134,
  196,
  49,
  50,
  39,
  185,
  276,
  288,
  268,
];

static PRUNE_14: [usize; 12] = [
  291,
  54,
  190,
  129,
  287,
  44,
  264,
  124,
  277,
  289,
  278,
  200,
];

static PRUNE_8: [usize; 10] = [
  280,
  174,
  270,
  34,
  181,
  272,
  35,
  267,
  273,
  121,
];

static PRUNE_26: [usize; 16] = [
  245,
  15,
  240,
  223,
  100,
  224,
  95,
  78,
  242,
  90,
  244,
  81,
  261,
  233,
  234,
  87,
];

static PRUNE_20: [usize; 3] = [
  0,
  299,
  3,
];

static PRUNE_24: [usize; 22] = [
  88,
  161,
  9,
  258,
  235,
  236,
  247,
  248,
  96,
  259,
  231,
  85,
  7,
  227,
  238,
  221,
  98,
  256,
  92,
  75,
  13,
  93,
];

static PRUNE_18: [usize; 14] = [
  145,
  208,
  292,
  62,
  304,
  294,
  148,
  300,
  205,
  53,
  199,
  296,
  59,
  138,
];

static PRUNE_5: [usize; 10] = [
  105,
  21,
  253,
  168,
  107,
  108,
  23,
  24,
  255,
  171,
];

static PRUNE_25: [usize; 23] = [
  77,
  99,
  82,
  212,
  257,
  241,
  162,
  89,
  94,
  237,
  79,
  260,
  232,
  91,
  86,
  80,
  97,
  222,
  239,
  76,
  14,
  228,
  262,
];

static PRUNE_12: [usize; 19] = [
  43,
  178,
  116,
  117,
  179,
  128,
  191,
  45,
  17,
  130,
  102,
  164,
  41,
  187,
  284,
  30,
  126,
  31,
  189,
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
    run_level(&mut temp_nodes, &LEVEL_11);
    prune(&mut temp_nodes, &PRUNE_11);
    run_level(&mut temp_nodes, &LEVEL_12);
    prune(&mut temp_nodes, &PRUNE_12);
    run_level(&mut temp_nodes, &LEVEL_13);
    prune(&mut temp_nodes, &PRUNE_13);
    run_level(&mut temp_nodes, &LEVEL_14);
    prune(&mut temp_nodes, &PRUNE_14);
    run_level(&mut temp_nodes, &LEVEL_15);
    prune(&mut temp_nodes, &PRUNE_15);
    run_level(&mut temp_nodes, &LEVEL_16);
    prune(&mut temp_nodes, &PRUNE_16);
    run_level(&mut temp_nodes, &LEVEL_17);
    prune(&mut temp_nodes, &PRUNE_17);
    run_level(&mut temp_nodes, &LEVEL_18);
    prune(&mut temp_nodes, &PRUNE_18);
    run_level(&mut temp_nodes, &LEVEL_19);
    prune(&mut temp_nodes, &PRUNE_19);
    run_level(&mut temp_nodes, &LEVEL_20);
    prune(&mut temp_nodes, &PRUNE_20);
    run_level(&mut temp_nodes, &LEVEL_21);
    prune(&mut temp_nodes, &PRUNE_21);
    run_level(&mut temp_nodes, &LEVEL_22);
    prune(&mut temp_nodes, &PRUNE_22);
    run_level(&mut temp_nodes, &LEVEL_23);
    prune(&mut temp_nodes, &PRUNE_23);
    run_level(&mut temp_nodes, &LEVEL_24);
    prune(&mut temp_nodes, &PRUNE_24);
    run_level(&mut temp_nodes, &LEVEL_25);
    prune(&mut temp_nodes, &PRUNE_25);
    run_level(&mut temp_nodes, &LEVEL_26);
    prune(&mut temp_nodes, &PRUNE_26);

            

                out.into_iter().map(|c| c.unwrap()).collect()
            }),
        )
        .unwrap()
}

