
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
    ((354, false, INV), &[Arg(0, 2)]),
    ((1976, false, NOR2), &[Arg(0, 1), Arg(0, 0)]),
    ((320, false, OR2), &[Arg(0, 33), Arg(0, 32)]),
    ((940, false, AND2), &[Arg(0, 33), Arg(0, 32)]),
];

static LEVEL_1: [((usize, bool, CellType), &[GateInput]); 6] = [
    ((307, false, INV), &[Arg(0, 1)]),
    ((332, false, INV), &[Arg(0, 33)]),
    ((1111, false, AND2), &[Arg(0, 1), Arg(0, 0)]),
    ((1977, false, NOR2), &[Tv(354), Tv(1976)]),
    ((324, false, AND2), &[Arg(0, 34), Tv(320)]),
    ((942, false, AND2), &[Arg(0, 34), Tv(940)]),
];

static LEVEL_2: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((301, false, INV), &[Arg(0, 4)]),
    ((321, false, INV), &[Arg(0, 36)]),
    ((991, false, AND2), &[Tv(307), Arg(0, 0)]),
    ((1133, false, AND2), &[Arg(0, 2), Tv(1111)]),
    ((1980, false, NAND2), &[Arg(0, 3), Tv(1977)]),
    ((314, false, AND2), &[Tv(332), Arg(0, 32)]),
    ((325, false, NAND2), &[Arg(0, 35), Tv(324)]),
    ((944, false, AND2), &[Arg(0, 35), Tv(942)]),
];

static LEVEL_3: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((376, false, INV), &[Arg(0, 5)]),
    ((684, false, INV), &[Arg(0, 37)]),
    ((1155, false, AND2), &[Arg(0, 3), Tv(1133)]),
    ((1981, false, AND2), &[Tv(301), Tv(1980)]),
    ((1984, false, AND2), &[Arg(0, 2), Tv(991)]),
    ((326, false, AND2), &[Tv(321), Tv(325)]),
    ((366, false, AND2), &[Arg(0, 34), Tv(314)]),
    ((946, false, AND2), &[Arg(0, 36), Tv(944)]),
];

static LEVEL_4: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((1177, false, AND2), &[Arg(0, 4), Tv(1155)]),
    ((1982, false, XNOR2), &[Arg(0, 4), Tv(1980)]),
    ((1986, false, AND2), &[Arg(0, 3), Tv(1984)]),
    ((1988, false, NOR2), &[Tv(376), Tv(1981)]),
    ((327, false, NOR2), &[Tv(684), Tv(326)]),
    ((363, false, XNOR2), &[Arg(0, 36), Tv(325)]),
    ((368, false, AND2), &[Arg(0, 35), Tv(366)]),
    ((949, false, AND2), &[Arg(0, 37), Tv(946)]),
];

static LEVEL_5: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((1199, false, AND2), &[Arg(0, 5), Tv(1177)]),
    ((1989, false, AND2), &[Arg(0, 6), Tv(1988)]),
    ((2016, false, XNOR2), &[Tv(376), Tv(1981)]),
    ((2018, false, AND2), &[Tv(1982), Tv(1986)]),
    ((328, false, AND2), &[Arg(0, 38), Tv(327)]),
    ((374, false, AND2), &[Tv(363), Tv(368)]),
    ((377, false, XNOR2), &[Tv(684), Tv(326)]),
    ((951, false, AND2), &[Arg(0, 38), Tv(949)]),
];

static LEVEL_6: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((1221, false, AND2), &[Arg(0, 6), Tv(1199)]),
    ((1990, false, AND2), &[Arg(0, 7), Tv(1989)]),
    ((2025, false, XNOR2), &[Arg(0, 6), Tv(1988)]),
    ((2027, false, AND2), &[Tv(2016), Tv(2018)]),
    ((329, false, AND2), &[Arg(0, 39), Tv(328)]),
    ((372, false, XNOR2), &[Arg(0, 38), Tv(327)]),
    ((379, false, AND2), &[Tv(374), Tv(377)]),
    ((953, false, AND2), &[Arg(0, 39), Tv(951)]),
];

static LEVEL_7: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((1243, false, AND2), &[Arg(0, 7), Tv(1221)]),
    ((1991, false, AND2), &[Arg(0, 8), Tv(1990)]),
    ((2023, false, XNOR2), &[Arg(0, 7), Tv(1989)]),
    ((2029, false, AND2), &[Tv(2025), Tv(2027)]),
    ((330, false, AND2), &[Arg(0, 40), Tv(329)]),
    ((384, false, XNOR2), &[Arg(0, 39), Tv(328)]),
    ((386, false, AND2), &[Tv(372), Tv(379)]),
    ((955, false, AND2), &[Arg(0, 40), Tv(953)]),
];

static LEVEL_8: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((1265, false, AND2), &[Arg(0, 8), Tv(1243)]),
    ((1992, false, AND2), &[Arg(0, 9), Tv(1991)]),
    ((2035, false, XNOR2), &[Arg(0, 8), Tv(1990)]),
    ((2037, false, AND2), &[Tv(2023), Tv(2029)]),
    ((331, false, AND2), &[Arg(0, 41), Tv(330)]),
    ((394, false, XNOR2), &[Arg(0, 40), Tv(329)]),
    ((396, false, AND2), &[Tv(384), Tv(386)]),
    ((957, false, AND2), &[Arg(0, 41), Tv(955)]),
];

static LEVEL_9: [((usize, bool, CellType), &[GateInput]); 9] = [
    ((1287, false, AND2), &[Arg(0, 9), Tv(1265)]),
    ((1993, false, AND2), &[Arg(0, 10), Tv(1992)]),
    ((2033, false, XNOR2), &[Arg(0, 9), Tv(1991)]),
    ((2039, false, AND2), &[Tv(2035), Tv(2037)]),
    ((323, false, AND2), &[Arg(0, 45), Arg(0, 46)]),
    ((333, false, AND2), &[Arg(0, 42), Tv(331)]),
    ((392, false, XNOR2), &[Arg(0, 41), Tv(330)]),
    ((399, false, AND2), &[Tv(394), Tv(396)]),
    ((960, false, AND2), &[Arg(0, 42), Tv(957)]),
];

static LEVEL_10: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((1309, false, AND2), &[Arg(0, 10), Tv(1287)]),
    ((1557, false, AND2), &[Arg(0, 13), Arg(0, 14)]),
    ((1994, false, AND2), &[Arg(0, 11), Tv(1993)]),
    ((2046, false, XNOR2), &[Arg(0, 10), Tv(1992)]),
    ((2048, false, AND2), &[Tv(2033), Tv(2039)]),
    ((334, false, AND2), &[Arg(0, 43), Tv(333)]),
    ((337, false, AND2), &[Arg(0, 44), Tv(323)]),
    ((406, false, XNOR2), &[Arg(0, 42), Tv(331)]),
    ((408, false, AND2), &[Tv(392), Tv(399)]),
    ((962, false, AND2), &[Arg(0, 43), Tv(960)]),
];

static LEVEL_11: [((usize, bool, CellType), &[GateInput]); 9] = [
    ((1331, false, AND2), &[Arg(0, 11), Tv(1309)]),
    ((1558, false, AND2), &[Arg(0, 12), Tv(1557)]),
    ((2044, false, XNOR2), &[Arg(0, 11), Tv(1993)]),
    ((2050, false, AND2), &[Tv(2046), Tv(2048)]),
    ((2054, false, AND2), &[Arg(0, 12), Tv(1994)]),
    ((335, false, AND2), &[Arg(0, 44), Tv(334)]),
    ((411, false, AND2), &[Tv(406), Tv(408)]),
    ((413, false, XNOR2), &[Arg(0, 43), Tv(333)]),
    ((964, false, AND2), &[Tv(337), Tv(962)]),
];

static LEVEL_12: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((475, false, INV), &[Arg(0, 14)]),
    ((783, false, INV), &[Arg(0, 46)]),
    ((1353, false, AND2), &[Arg(0, 12), Tv(1331)]),
    ((1995, false, AND2), &[Tv(1558), Tv(1994)]),
    ((2055, false, NAND2), &[Arg(0, 13), Tv(2054)]),
    ((2058, false, AND2), &[Tv(2044), Tv(2050)]),
    ((2060, false, XNOR2), &[Arg(0, 12), Tv(1994)]),
    ((338, false, AND2), &[Tv(334), Tv(337)]),
    ((404, false, XNOR2), &[Arg(0, 44), Tv(334)]),
    ((415, false, AND2), &[Tv(411), Tv(413)]),
    ((419, false, NAND2), &[Arg(0, 45), Tv(335)]),
    ((966, false, AND2), &[Arg(0, 47), Tv(964)]),
];

static LEVEL_13: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((1375, false, AND2), &[Arg(0, 13), Tv(1353)]),
    ((1996, false, INV), &[Tv(1995)]),
    ((2056, false, XNOR2), &[Arg(0, 13), Tv(2054)]),
    ((2062, false, AND2), &[Tv(2058), Tv(2060)]),
    ((2069, false, NAND2), &[Tv(475), Tv(2055)]),
    ((339, false, INV), &[Tv(338)]),
    ((421, false, XNOR2), &[Arg(0, 45), Tv(335)]),
    ((423, false, AND2), &[Tv(404), Tv(415)]),
    ((430, false, NAND2), &[Tv(783), Tv(419)]),
    ((968, false, AND2), &[Arg(0, 48), Tv(966)]),
];

static LEVEL_14: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((1397, false, AND2), &[Arg(0, 14), Tv(1375)]),
    ((1997, false, AND2), &[Arg(0, 15), Tv(1995)]),
    ((2070, false, NAND2), &[Tv(1996), Tv(2069)]),
    ((2072, false, AND2), &[Tv(2056), Tv(2062)]),
    ((340, false, AND2), &[Arg(0, 47), Tv(338)]),
    ((432, false, NAND2), &[Tv(339), Tv(430)]),
    ((434, false, AND2), &[Tv(421), Tv(423)]),
    ((970, false, AND2), &[Arg(0, 49), Tv(968)]),
];

static LEVEL_15: [((usize, bool, CellType), &[GateInput]); 9] = [
    ((1419, false, AND2), &[Arg(0, 15), Tv(1397)]),
    ((1998, false, AND2), &[Arg(0, 16), Tv(1997)]),
    ((2067, false, XNOR2), &[Arg(0, 15), Tv(1995)]),
    ((2074, false, AND2), &[Tv(2070), Tv(2072)]),
    ((322, false, AND2), &[Arg(0, 53), Arg(0, 54)]),
    ((341, false, AND2), &[Arg(0, 48), Tv(340)]),
    ((428, false, XNOR2), &[Arg(0, 47), Tv(338)]),
    ((436, false, AND2), &[Tv(432), Tv(434)]),
    ((972, false, AND2), &[Arg(0, 50), Tv(970)]),
];

static LEVEL_16: [((usize, bool, CellType), &[GateInput]); 9] = [
    ((1441, false, AND2), &[Arg(0, 16), Tv(1419)]),
    ((1999, false, AND2), &[Arg(0, 17), Tv(1998)]),
    ((2079, false, XNOR2), &[Arg(0, 16), Tv(1997)]),
    ((2081, false, AND2), &[Tv(2067), Tv(2074)]),
    ((336, false, AND2), &[Arg(0, 52), Tv(322)]),
    ((342, false, AND2), &[Arg(0, 49), Tv(341)]),
    ((444, false, AND2), &[Tv(428), Tv(436)]),
    ((446, false, XNOR2), &[Arg(0, 48), Tv(340)]),
    ((974, false, AND2), &[Arg(0, 51), Tv(972)]),
];

static LEVEL_17: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((1463, false, AND2), &[Arg(0, 17), Tv(1441)]),
    ((2000, false, AND2), &[Arg(0, 18), Tv(1999)]),
    ((2100, false, AND2), &[Tv(2079), Tv(2081)]),
    ((2102, false, XNOR2), &[Arg(0, 17), Tv(1998)]),
    ((344, false, AND2), &[Arg(0, 50), Tv(342)]),
    ((448, false, AND2), &[Tv(444), Tv(446)]),
    ((450, false, XNOR2), &[Arg(0, 49), Tv(341)]),
    ((976, false, AND2), &[Tv(336), Tv(974)]),
];

static LEVEL_18: [((usize, bool, CellType), &[GateInput]); 9] = [
    ((1483, false, AND2), &[Arg(0, 18), Tv(1463)]),
    ((1555, false, AND2), &[Arg(0, 21), Arg(0, 22)]),
    ((2001, false, AND2), &[Arg(0, 19), Tv(2000)]),
    ((2098, false, XNOR2), &[Arg(0, 18), Tv(1999)]),
    ((2104, false, AND2), &[Tv(2100), Tv(2102)]),
    ((345, false, AND2), &[Arg(0, 51), Tv(344)]),
    ((441, false, XNOR2), &[Arg(0, 50), Tv(342)]),
    ((452, false, AND2), &[Tv(448), Tv(450)]),
    ((978, false, AND2), &[Arg(0, 55), Tv(976)]),
];

static LEVEL_19: [((usize, bool, CellType), &[GateInput]); 9] = [
    ((1504, false, AND2), &[Arg(0, 19), Tv(1483)]),
    ((1556, false, AND2), &[Arg(0, 20), Tv(1555)]),
    ((2089, false, AND2), &[Arg(0, 20), Tv(2001)]),
    ((2096, false, XNOR2), &[Arg(0, 19), Tv(2000)]),
    ((2106, false, AND2), &[Tv(2098), Tv(2104)]),
    ((458, false, AND2), &[Arg(0, 52), Tv(345)]),
    ((466, false, AND2), &[Tv(441), Tv(452)]),
    ((468, false, XNOR2), &[Arg(0, 51), Tv(344)]),
    ((981, false, AND2), &[Arg(0, 56), Tv(978)]),
];

static LEVEL_20: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((563, false, INV), &[Arg(0, 22)]),
    ((871, false, INV), &[Arg(0, 54)]),
    ((1519, false, AND2), &[Arg(0, 20), Tv(1504)]),
    ((2002, false, AND2), &[Tv(1556), Tv(2001)]),
    ((2090, false, NAND2), &[Arg(0, 21), Tv(2089)]),
    ((2108, false, AND2), &[Tv(2096), Tv(2106)]),
    ((2110, false, XNOR2), &[Arg(0, 20), Tv(2001)]),
    ((346, false, AND2), &[Tv(336), Tv(345)]),
    ((459, false, NAND2), &[Arg(0, 53), Tv(458)]),
    ((463, false, XNOR2), &[Arg(0, 52), Tv(345)]),
    ((470, false, AND2), &[Tv(466), Tv(468)]),
    ((983, false, AND2), &[Arg(0, 57), Tv(981)]),
];

static LEVEL_21: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1525, false, AND2), &[Arg(0, 21), Tv(1519)]),
    ((2003, false, INV), &[Tv(2002)]),
    ((2004, false, AND2), &[Arg(0, 23), Tv(2002)]),
    ((2091, false, NAND2), &[Tv(563), Tv(2090)]),
    ((2094, false, XNOR2), &[Arg(0, 21), Tv(2089)]),
    ((2112, false, AND2), &[Tv(2108), Tv(2110)]),
    ((347, false, INV), &[Tv(346)]),
    ((348, false, AND2), &[Arg(0, 55), Tv(346)]),
    ((460, false, NAND2), &[Tv(871), Tv(459)]),
    ((472, false, AND2), &[Tv(463), Tv(470)]),
    ((474, false, XNOR2), &[Arg(0, 53), Tv(458)]),
    ((985, false, AND2), &[Arg(0, 58), Tv(983)]),
];

static LEVEL_22: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((1537, false, AND2), &[Arg(0, 22), Tv(1525)]),
    ((2005, false, AND2), &[Arg(0, 24), Tv(2004)]),
    ((2092, false, NAND2), &[Tv(2003), Tv(2091)]),
    ((2114, false, AND2), &[Tv(2094), Tv(2112)]),
    ((349, false, AND2), &[Arg(0, 56), Tv(348)]),
    ((461, false, NAND2), &[Tv(347), Tv(460)]),
    ((477, false, AND2), &[Tv(472), Tv(474)]),
    ((986, false, AND2), &[Arg(0, 59), Tv(985)]),
];

static LEVEL_23: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((1543, false, AND2), &[Arg(0, 23), Tv(1537)]),
    ((2006, false, AND2), &[Arg(0, 25), Tv(2005)]),
    ((2087, false, XNOR2), &[Arg(0, 23), Tv(2002)]),
    ((2116, false, AND2), &[Tv(2092), Tv(2114)]),
    ((350, false, AND2), &[Arg(0, 57), Tv(349)]),
    ((487, false, AND2), &[Tv(461), Tv(477)]),
    ((489, false, XNOR2), &[Arg(0, 55), Tv(346)]),
    ((987, false, AND2), &[Arg(0, 60), Tv(986)]),
];

static LEVEL_24: [((usize, bool, CellType), &[GateInput]); 11] = [
    ((937, false, INV), &[Arg(0, 60)]),
    ((948, false, INV), &[Arg(0, 61)]),
    ((1545, false, AND2), &[Arg(0, 24), Tv(1543)]),
    ((2007, false, AND2), &[Arg(0, 26), Tv(2006)]),
    ((2085, false, XNOR2), &[Arg(0, 24), Tv(2004)]),
    ((2118, false, AND2), &[Tv(2087), Tv(2116)]),
    ((351, false, AND2), &[Arg(0, 58), Tv(350)]),
    ((484, false, XNOR2), &[Arg(0, 56), Tv(348)]),
    ((491, false, AND2), &[Tv(487), Tv(489)]),
    ((1026, false, NOR2), &[Arg(0, 63), Tv(987)]),
    ((1029, false, NOR2), &[Arg(0, 63), Tv(986)]),
];

static LEVEL_25: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((1547, false, AND2), &[Arg(0, 25), Tv(1545)]),
    ((2008, false, AND2), &[Arg(0, 27), Tv(2007)]),
    ((2124, false, AND2), &[Tv(2085), Tv(2118)]),
    ((2126, false, XNOR2), &[Arg(0, 25), Tv(2005)]),
    ((352, false, AND2), &[Arg(0, 59), Tv(351)]),
    ((493, false, AND2), &[Tv(484), Tv(491)]),
    ((495, false, XNOR2), &[Arg(0, 57), Tv(349)]),
    ((988, false, AND2), &[Arg(0, 61), Tv(987)]),
    ((1027, false, XNOR2), &[Tv(948), Tv(1026)]),
    ((1030, false, XNOR2), &[Tv(937), Tv(1029)]),
];

static LEVEL_26: [((usize, bool, CellType), &[GateInput]); 9] = [
    ((1549, false, AND2), &[Arg(0, 26), Tv(1547)]),
    ((2009, false, AND2), &[Arg(0, 28), Tv(2008)]),
    ((2122, false, XNOR2), &[Arg(0, 26), Tv(2006)]),
    ((2128, false, AND2), &[Tv(2124), Tv(2126)]),
    ((353, false, AND2), &[Arg(0, 60), Tv(352)]),
    ((482, false, XNOR2), &[Arg(0, 58), Tv(350)]),
    ((498, false, AND2), &[Tv(493), Tv(495)]),
    ((1023, false, NOR2), &[Arg(0, 63), Tv(988)]),
    ((1031, false, AND2), &[Tv(1027), Tv(1030)]),
];

static LEVEL_27: [((usize, bool, CellType), &[GateInput]); 11] = [
    ((959, false, INV), &[Arg(0, 63)]),
    ((1551, false, AND2), &[Arg(0, 27), Tv(1549)]),
    ((2010, false, AND2), &[Arg(0, 29), Tv(2009)]),
    ((2133, false, AND2), &[Tv(2122), Tv(2128)]),
    ((2134, false, XNOR2), &[Arg(0, 27), Tv(2007)]),
    ((355, false, AND2), &[Arg(0, 61), Tv(353)]),
    ((502, false, XNOR2), &[Arg(0, 59), Tv(351)]),
    ((503, false, AND2), &[Tv(482), Tv(498)]),
    ((989, false, AND2), &[Arg(0, 62), Tv(988)]),
    ((1025, false, XNOR2), &[Arg(0, 62), Tv(1023)]),
    ((1032, false, INV), &[Tv(1031)]),
];

static LEVEL_28: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((651, false, INV), &[Arg(0, 31)]),
    ((1552, false, AND2), &[Arg(0, 28), Tv(1551)]),
    ((2011, false, AND2), &[Arg(0, 30), Tv(2010)]),
    ((2135, false, AND2), &[Tv(2133), Tv(2134)]),
    ((2136, false, XNOR2), &[Arg(0, 28), Tv(2008)]),
    ((356, false, AND2), &[Arg(0, 62), Tv(355)]),
    ((506, false, XNOR2), &[Arg(0, 60), Tv(352)]),
    ((507, false, AND2), &[Tv(502), Tv(503)]),
    ((990, false, NAND2), &[Tv(959), Tv(989)]),
    ((1037, false, NAND2), &[Tv(1025), Tv(1032)]),
];

static LEVEL_29: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((629, false, INV), &[Arg(0, 28)]),
    ((640, false, INV), &[Arg(0, 29)]),
    ((926, false, INV), &[Arg(0, 59)]),
    ((1588, false, NOR2), &[Arg(0, 31), Tv(1552)]),
    ((1591, false, NOR2), &[Arg(0, 31), Tv(1551)]),
    ((2012, false, XNOR2), &[Tv(651), Tv(2011)]),
    ((2137, false, AND2), &[Tv(2135), Tv(2136)]),
    ((2138, false, XNOR2), &[Arg(0, 29), Tv(2009)]),
    ((357, false, XNOR2), &[Tv(959), Tv(356)]),
    ((511, false, XNOR2), &[Arg(0, 61), Tv(353)]),
    ((512, false, AND2), &[Tv(506), Tv(507)]),
    ((1028, false, INV), &[Tv(1027)]),
    ((1033, false, NAND2), &[Tv(1025), Tv(1030)]),
    ((1038, false, AND2), &[Tv(990), Tv(1037)]),
    ((1041, false, NOR2), &[Arg(0, 63), Tv(985)]),
];

static LEVEL_30: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1553, false, AND2), &[Arg(0, 29), Tv(1552)]),
    ((1589, false, XNOR2), &[Tv(640), Tv(1588)]),
    ((1592, false, XNOR2), &[Tv(629), Tv(1591)]),
    ((2013, false, INV), &[Tv(2012)]),
    ((2139, false, NAND2), &[Tv(2137), Tv(2138)]),
    ((2141, false, XNOR2), &[Arg(0, 30), Tv(2010)]),
    ((358, false, INV), &[Tv(357)]),
    ((513, false, NAND2), &[Tv(511), Tv(512)]),
    ((515, false, XNOR2), &[Arg(0, 62), Tv(355)]),
    ((1039, false, XNOR2), &[Tv(1030), Tv(1038)]),
    ((1042, false, XNOR2), &[Tv(926), Tv(1041)]),
    ((1045, false, NAND2), &[Tv(1028), Tv(1033)]),
];

static LEVEL_31: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1586, false, NOR2), &[Arg(0, 31), Tv(1553)]),
    ((1593, false, AND2), &[Tv(1589), Tv(1592)]),
    ((2142, false, AND2), &[Tv(2012), Tv(2141)]),
    ((2144, false, AND2), &[Arg(0, 31), Tv(2139)]),
    ((2147, false, OR2), &[Tv(2013), Tv(2135)]),
    ((2149, false, OR2), &[Tv(2013), Tv(2137)]),
    ((509, false, OR2), &[Tv(358), Tv(507)]),
    ((516, false, AND2), &[Tv(357), Tv(515)]),
    ((520, false, AND2), &[Arg(0, 63), Tv(513)]),
    ((523, false, OR2), &[Tv(358), Tv(512)]),
    ((1043, false, AND2), &[Tv(1039), Tv(1042)]),
    ((1047, false, AND2), &[Tv(990), Tv(1045)]),
];

static LEVEL_32: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((1554, false, AND2), &[Arg(0, 30), Tv(1553)]),
    ((1587, false, XNOR2), &[Arg(0, 30), Tv(1586)]),
    ((1594, false, INV), &[Tv(1593)]),
    ((2143, false, NAND2), &[Tv(2139), Tv(2142)]),
    ((2145, false, OR2), &[Tv(2141), Tv(2144)]),
    ((2148, false, XNOR2), &[Tv(2136), Tv(2147)]),
    ((2150, false, XNOR2), &[Tv(2138), Tv(2149)]),
    ((510, false, XNOR2), &[Tv(506), Tv(509)]),
    ((518, false, NAND2), &[Tv(513), Tv(516)]),
    ((521, false, OR2), &[Tv(515), Tv(520)]),
    ((524, false, XNOR2), &[Tv(511), Tv(523)]),
    ((1034, false, NAND2), &[Tv(1025), Tv(1031)]),
    ((1049, false, OR2), &[Tv(1043), Tv(1047)]),
];

static LEVEL_33: [((usize, bool, CellType), &[GateInput]); 11] = [
    ((1550, false, INV), &[Tv(1549)]),
    ((1559, false, NAND2), &[Tv(651), Tv(1554)]),
    ((1560, false, XNOR2), &[Arg(0, 31), Tv(1554)]),
    ((1600, false, NAND2), &[Tv(1587), Tv(1594)]),
    ((2140, false, INV), &[Tv(2139)]),
    ((2146, false, NAND2), &[Tv(2143), Tv(2145)]),
    ((2151, false, NAND2), &[Tv(2148), Tv(2150)]),
    ((514, false, INV), &[Tv(513)]),
    ((522, false, NAND2), &[Tv(518), Tv(521)]),
    ((525, false, NAND2), &[Tv(510), Tv(524)]),
    ((1052, false, AND2), &[Tv(1034), Tv(1049)]),
];

static LEVEL_34: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((618, false, INV), &[Arg(0, 27)]),
    ((1590, false, INV), &[Tv(1589)]),
    ((1595, false, NAND2), &[Tv(1587), Tv(1592)]),
    ((1598, false, AND2), &[Tv(1550), Tv(1560)]),
    ((1601, false, AND2), &[Tv(1559), Tv(1600)]),
    ((2152, false, NAND2), &[Tv(2146), Tv(2151)]),
    ((2153, false, NAND2), &[Tv(2140), Tv(2142)]),
    ((517, false, NAND2), &[Tv(514), Tv(516)]),
    ((526, false, NAND2), &[Tv(522), Tv(525)]),
    ((984, false, INV), &[Tv(983)]),
    ((992, false, XNOR2), &[Arg(0, 63), Tv(989)]),
    ((1053, false, INV), &[Tv(1052)]),
];

static LEVEL_35: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((915, false, INV), &[Arg(0, 58)]),
    ((1599, false, XNOR2), &[Tv(618), Tv(1598)]),
    ((1602, false, XNOR2), &[Tv(1592), Tv(1601)]),
    ((1606, false, NAND2), &[Tv(1590), Tv(1595)]),
    ((2154, false, AND2), &[Tv(2152), Tv(2153)]),
    ((2157, false, OR2), &[Tv(2013), Tv(2133)]),
    ((2164, false, AND2), &[Tv(2146), Tv(2148)]),
    ((504, false, OR2), &[Tv(358), Tv(503)]),
    ((527, false, AND2), &[Tv(517), Tv(526)]),
    ((535, false, AND2), &[Tv(510), Tv(522)]),
    ((1036, false, INV), &[Tv(1034)]),
    ((1040, false, INV), &[Tv(1039)]),
    ((1048, false, NAND2), &[Tv(1034), Tv(1047)]),
    ((1054, false, NAND2), &[Tv(1042), Tv(1053)]),
    ((1059, false, AND2), &[Tv(984), Tv(992)]),
];

static LEVEL_36: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((1604, false, AND2), &[Tv(1599), Tv(1602)]),
    ((1607, false, AND2), &[Tv(1559), Tv(1606)]),
    ((2158, false, XNOR2), &[Tv(2134), Tv(2157)]),
    ((2160, false, XNOR2), &[Tv(2148), Tv(2154)]),
    ((2165, false, OR2), &[Tv(2150), Tv(2164)]),
    ((505, false, XNOR2), &[Tv(502), Tv(504)]),
    ((528, false, XNOR2), &[Tv(510), Tv(527)]),
    ((536, false, OR2), &[Tv(524), Tv(535)]),
    ((1044, false, NAND2), &[Tv(1036), Tv(1043)]),
    ((1050, false, AND2), &[Tv(1048), Tv(1049)]),
    ((1055, false, NAND2), &[Tv(1040), Tv(1054)]),
    ((1060, false, XNOR2), &[Tv(915), Tv(1059)]),
    ((1061, false, XNOR2), &[Tv(1042), Tv(1052)]),
];

static LEVEL_37: [((usize, bool, CellType), &[GateInput]); 9] = [
    ((1596, false, NAND2), &[Tv(1587), Tv(1593)]),
    ((1609, false, OR2), &[Tv(1604), Tv(1607)]),
    ((2162, false, AND2), &[Tv(2158), Tv(2160)]),
    ((2166, false, AND2), &[Tv(2153), Tv(2165)]),
    ((531, false, AND2), &[Tv(505), Tv(528)]),
    ((537, false, AND2), &[Tv(517), Tv(536)]),
    ((1051, false, NAND2), &[Tv(1044), Tv(1050)]),
    ((1056, false, AND2), &[Tv(1044), Tv(1055)]),
    ((1062, false, AND2), &[Tv(1060), Tv(1061)]),
];

static LEVEL_38: [((usize, bool, CellType), &[GateInput]); 7] = [
    ((1615, false, AND2), &[Tv(1596), Tv(1609)]),
    ((2155, false, NAND2), &[Tv(2146), Tv(2154)]),
    ((2167, false, OR2), &[Tv(2162), Tv(2166)]),
    ((532, false, NAND2), &[Tv(522), Tv(527)]),
    ((538, false, OR2), &[Tv(531), Tv(537)]),
    ((1058, false, NAND2), &[Tv(1051), Tv(1056)]),
    ((1064, false, NAND2), &[Tv(1051), Tv(1062)]),
];

static LEVEL_39: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1548, false, INV), &[Tv(1547)]),
    ((1597, false, INV), &[Tv(1596)]),
    ((1616, false, INV), &[Tv(1615)]),
    ((2129, false, INV), &[Tv(2128)]),
    ((2159, false, INV), &[Tv(2158)]),
    ((2169, false, AND2), &[Tv(2155), Tv(2167)]),
    ((499, false, INV), &[Tv(498)]),
    ((539, false, NAND2), &[Tv(532), Tv(537)]),
    ((543, false, NAND2), &[Tv(532), Tv(538)]),
    ((982, false, INV), &[Tv(981)]),
    ((1065, false, AND2), &[Tv(1060), Tv(1064)]),
    ((1067, false, OR2), &[Tv(1058), Tv(1060)]),
];

static LEVEL_40: [((usize, bool, CellType), &[GateInput]); 21] = [
    ((607, false, INV), &[Arg(0, 26)]),
    ((904, false, INV), &[Arg(0, 57)]),
    ((1603, false, INV), &[Tv(1602)]),
    ((1605, false, NAND2), &[Tv(1597), Tv(1604)]),
    ((1612, false, AND2), &[Tv(1548), Tv(1560)]),
    ((1617, false, NAND2), &[Tv(1599), Tv(1616)]),
    ((2123, false, INV), &[Tv(2122)]),
    ((2130, false, AND2), &[Tv(2012), Tv(2129)]),
    ((2156, false, INV), &[Tv(2155)]),
    ((2161, false, INV), &[Tv(2160)]),
    ((2170, false, OR2), &[Tv(2159), Tv(2169)]),
    ((483, false, INV), &[Tv(482)]),
    ((500, false, AND2), &[Tv(357), Tv(499)]),
    ((529, false, INV), &[Tv(528)]),
    ((533, false, INV), &[Tv(532)]),
    ((544, false, NAND2), &[Tv(505), Tv(543)]),
    ((547, false, OR2), &[Tv(505), Tv(539)]),
    ((1021, false, AND2), &[Tv(982), Tv(992)]),
    ((1063, false, NAND2), &[Tv(1060), Tv(1061)]),
    ((1066, false, AND2), &[Tv(1058), Tv(1065)]),
    ((1069, false, INV), &[Tv(1067)]),
];

static LEVEL_41: [((usize, bool, CellType), &[GateInput]); 18] = [
    ((1608, false, NAND2), &[Tv(1596), Tv(1607)]),
    ((1610, false, AND2), &[Tv(1605), Tv(1609)]),
    ((1613, false, XNOR2), &[Tv(607), Tv(1612)]),
    ((1618, false, XNOR2), &[Tv(1599), Tv(1615)]),
    ((1620, false, NAND2), &[Tv(1603), Tv(1617)]),
    ((2131, false, XNOR2), &[Tv(2123), Tv(2130)]),
    ((2163, false, NAND2), &[Tv(2156), Tv(2162)]),
    ((2168, false, NAND2), &[Tv(2155), Tv(2166)]),
    ((2171, false, NAND2), &[Tv(2161), Tv(2170)]),
    ((2173, false, XNOR2), &[Tv(2158), Tv(2169)]),
    ((501, false, XNOR2), &[Tv(483), Tv(500)]),
    ((534, false, NAND2), &[Tv(531), Tv(533)]),
    ((540, false, AND2), &[Tv(538), Tv(539)]),
    ((545, false, NAND2), &[Tv(529), Tv(544)]),
    ((548, false, AND2), &[Tv(544), Tv(547)]),
    ((1022, false, XNOR2), &[Tv(904), Tv(1021)]),
    ((1070, false, NOR2), &[Tv(1066), Tv(1069)]),
    ((1075, false, NAND2), &[Tv(1056), Tv(1063)]),
];

static LEVEL_42: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1611, false, NAND2), &[Tv(1608), Tv(1610)]),
    ((1619, false, AND2), &[Tv(1613), Tv(1618)]),
    ((1621, false, AND2), &[Tv(1605), Tv(1620)]),
    ((2172, false, NAND2), &[Tv(2163), Tv(2171)]),
    ((2175, false, NAND2), &[Tv(2131), Tv(2173)]),
    ((2177, false, AND2), &[Tv(2167), Tv(2168)]),
    ((542, false, NAND2), &[Tv(534), Tv(540)]),
    ((546, false, AND2), &[Tv(534), Tv(545)]),
    ((549, false, AND2), &[Tv(501), Tv(548)]),
    ((1072, false, NAND2), &[Tv(1022), Tv(1070)]),
    ((1073, false, XNOR2), &[Tv(1061), Tv(1066)]),
    ((1076, false, NAND2), &[Tv(1064), Tv(1075)]),
];

static LEVEL_43: [((usize, bool, CellType), &[GateInput]); 9] = [
    ((1546, false, INV), &[Tv(1545)]),
    ((1622, false, NAND2), &[Tv(1611), Tv(1621)]),
    ((1623, false, NAND2), &[Tv(1611), Tv(1619)]),
    ((2176, false, NAND2), &[Tv(2172), Tv(2175)]),
    ((2178, false, NAND2), &[Tv(2163), Tv(2177)]),
    ((551, false, NAND2), &[Tv(542), Tv(546)]),
    ((553, false, NAND2), &[Tv(542), Tv(549)]),
    ((1074, false, NAND2), &[Tv(1072), Tv(1073)]),
    ((1077, false, NAND2), &[Tv(1058), Tv(1076)]),
];

static LEVEL_44: [((usize, bool, CellType), &[GateInput]); 11] = [
    ((596, false, INV), &[Arg(0, 25)]),
    ((1614, false, INV), &[Tv(1613)]),
    ((1624, false, AND2), &[Tv(1622), Tv(1623)]),
    ((1632, false, AND2), &[Tv(1546), Tv(1560)]),
    ((2125, false, INV), &[Tv(2124)]),
    ((2132, false, INV), &[Tv(2131)]),
    ((2180, false, AND2), &[Tv(2176), Tv(2178)]),
    ((494, false, INV), &[Tv(493)]),
    ((550, false, INV), &[Tv(549)]),
    ((554, false, AND2), &[Tv(551), Tv(553)]),
    ((1080, false, AND2), &[Tv(1074), Tv(1077)]),
];

static LEVEL_45: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((1625, false, AND2), &[Tv(1613), Tv(1624)]),
    ((1628, false, OR2), &[Tv(1619), Tv(1621)]),
    ((1633, false, XNOR2), &[Tv(596), Tv(1632)]),
    ((1635, false, XNOR2), &[Tv(1614), Tv(1624)]),
    ((2127, false, INV), &[Tv(2126)]),
    ((2174, false, INV), &[Tv(2173)]),
    ((2181, false, OR2), &[Tv(2132), Tv(2180)]),
    ((2185, false, AND2), &[Tv(2012), Tv(2125)]),
    ((496, false, INV), &[Tv(495)]),
    ((555, false, NAND2), &[Tv(501), Tv(554)]),
    ((556, false, OR2), &[Tv(501), Tv(551)]),
    ((559, false, AND2), &[Tv(357), Tv(494)]),
    ((567, false, NAND2), &[Tv(546), Tv(550)]),
    ((979, false, INV), &[Tv(978)]),
    ((1081, false, INV), &[Tv(1080)]),
];

static LEVEL_46: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((893, false, INV), &[Arg(0, 56)]),
    ((1626, false, NAND2), &[Tv(1618), Tv(1625)]),
    ((1627, false, XNOR2), &[Tv(1618), Tv(1625)]),
    ((1629, false, AND2), &[Tv(1622), Tv(1628)]),
    ((1637, false, NAND2), &[Tv(1633), Tv(1635)]),
    ((2179, false, INV), &[Tv(2178)]),
    ((2183, false, XNOR2), &[Tv(2131), Tv(2180)]),
    ((2186, false, XNOR2), &[Tv(2127), Tv(2185)]),
    ((2188, false, OR2), &[Tv(2175), Tv(2178)]),
    ((2193, false, NAND2), &[Tv(2174), Tv(2181)]),
    ((557, false, AND2), &[Tv(555), Tv(556)]),
    ((560, false, XNOR2), &[Tv(496), Tv(559)]),
    ((568, false, NAND2), &[Tv(553), Tv(567)]),
    ((1019, false, AND2), &[Tv(979), Tv(992)]),
    ((1071, false, INV), &[Tv(1070)]),
    ((1078, false, INV), &[Tv(1077)]),
    ((1082, false, NAND2), &[Tv(1022), Tv(1081)]),
];

static LEVEL_47: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1630, false, NAND2), &[Tv(1626), Tv(1629)]),
    ((1638, false, NAND2), &[Tv(1627), Tv(1637)]),
    ((2182, false, OR2), &[Tv(2172), Tv(2179)]),
    ((2187, false, AND2), &[Tv(2183), Tv(2186)]),
    ((2189, false, AND2), &[Tv(2176), Tv(2188)]),
    ((2194, false, AND2), &[Tv(2188), Tv(2193)]),
    ((562, false, AND2), &[Tv(557), Tv(560)]),
    ((565, false, XNOR2), &[Tv(548), Tv(555)]),
    ((569, false, NAND2), &[Tv(551), Tv(568)]),
    ((1020, false, XNOR2), &[Tv(893), Tv(1019)]),
    ((1083, false, XNOR2), &[Tv(1022), Tv(1080)]),
    ((1086, false, NAND2), &[Tv(1071), Tv(1082)]),
    ((1087, false, OR2), &[Tv(1072), Tv(1077)]),
    ((1091, false, OR2), &[Tv(1073), Tv(1078)]),
];

static LEVEL_48: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1544, false, INV), &[Tv(1543)]),
    ((1634, false, INV), &[Tv(1633)]),
    ((1642, false, AND2), &[Tv(1630), Tv(1638)]),
    ((2190, false, NAND2), &[Tv(2182), Tv(2189)]),
    ((2195, false, OR2), &[Tv(2187), Tv(2194)]),
    ((492, false, INV), &[Tv(491)]),
    ((564, false, NAND2), &[Tv(557), Tv(560)]),
    ((570, false, NAND2), &[Tv(562), Tv(569)]),
    ((572, false, NAND2), &[Tv(565), Tv(569)]),
    ((1085, false, NAND2), &[Tv(1020), Tv(1083)]),
    ((1088, false, NAND2), &[Tv(1086), Tv(1087)]),
    ((1092, false, AND2), &[Tv(1074), Tv(1091)]),
];

static LEVEL_49: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((585, false, INV), &[Arg(0, 24)]),
    ((1631, false, OR2), &[Tv(1627), Tv(1629)]),
    ((1636, false, INV), &[Tv(1635)]),
    ((1639, false, OR2), &[Tv(1630), Tv(1637)]),
    ((1643, false, OR2), &[Tv(1634), Tv(1642)]),
    ((1646, false, AND2), &[Tv(1544), Tv(1560)]),
    ((2196, false, NAND2), &[Tv(2190), Tv(2195)]),
    ((485, false, INV), &[Tv(484)]),
    ((561, false, INV), &[Tv(560)]),
    ((566, false, NAND2), &[Tv(564), Tv(565)]),
    ((575, false, AND2), &[Tv(570), Tv(572)]),
    ((579, false, AND2), &[Tv(357), Tv(492)]),
    ((1089, false, NAND2), &[Tv(1085), Tv(1088)]),
    ((1093, false, NAND2), &[Tv(1087), Tv(1092)]),
];

static LEVEL_50: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((1640, false, AND2), &[Tv(1631), Tv(1639)]),
    ((1644, false, XNOR2), &[Tv(1633), Tv(1642)]),
    ((1647, false, XNOR2), &[Tv(585), Tv(1646)]),
    ((1649, false, NAND2), &[Tv(1636), Tv(1643)]),
    ((2119, false, INV), &[Tv(2118)]),
    ((2184, false, INV), &[Tv(2183)]),
    ((2191, false, INV), &[Tv(2190)]),
    ((2197, false, NAND2), &[Tv(2186), Tv(2196)]),
    ((2200, false, NAND2), &[Tv(2190), Tv(2194)]),
    ((558, false, INV), &[Tv(557)]),
    ((571, false, NAND2), &[Tv(566), Tv(570)]),
    ((576, false, AND2), &[Tv(560), Tv(575)]),
    ((580, false, XNOR2), &[Tv(485), Tv(579)]),
    ((581, false, XNOR2), &[Tv(561), Tv(575)]),
    ((1095, false, AND2), &[Tv(1089), Tv(1093)]),
];

static LEVEL_51: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1641, false, NAND2), &[Tv(1638), Tv(1640)]),
    ((1648, false, AND2), &[Tv(1644), Tv(1647)]),
    ((1650, false, AND2), &[Tv(1639), Tv(1649)]),
    ((2086, false, INV), &[Tv(2085)]),
    ((2120, false, AND2), &[Tv(2012), Tv(2119)]),
    ((2192, false, NAND2), &[Tv(2187), Tv(2191)]),
    ((2198, false, NAND2), &[Tv(2184), Tv(2197)]),
    ((2201, false, AND2), &[Tv(2195), Tv(2200)]),
    ((2206, false, OR2), &[Tv(2186), Tv(2200)]),
    ((573, false, NAND2), &[Tv(571), Tv(572)]),
    ((577, false, XNOR2), &[Tv(558), Tv(576)]),
    ((582, false, AND2), &[Tv(580), Tv(581)]),
    ((977, false, INV), &[Tv(976)]),
    ((1096, false, INV), &[Tv(1095)]),
];

static LEVEL_52: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((882, false, INV), &[Arg(0, 55)]),
    ((1651, false, OR2), &[Tv(1648), Tv(1650)]),
    ((1655, false, NAND2), &[Tv(1641), Tv(1650)]),
    ((2121, false, XNOR2), &[Tv(2086), Tv(2120)]),
    ((2199, false, AND2), &[Tv(2192), Tv(2198)]),
    ((2202, false, NAND2), &[Tv(2192), Tv(2201)]),
    ((2207, false, AND2), &[Tv(2197), Tv(2206)]),
    ((578, false, NAND2), &[Tv(573), Tv(577)]),
    ((584, false, NAND2), &[Tv(573), Tv(582)]),
    ((1017, false, AND2), &[Tv(977), Tv(992)]),
    ((1084, false, INV), &[Tv(1083)]),
    ((1094, false, INV), &[Tv(1093)]),
    ((1097, false, NAND2), &[Tv(1020), Tv(1096)]),
];

static LEVEL_53: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((1542, false, INV), &[Tv(1537)]),
    ((1652, false, NAND2), &[Tv(1641), Tv(1651)]),
    ((1661, false, OR2), &[Tv(1647), Tv(1655)]),
    ((2204, false, NAND2), &[Tv(2199), Tv(2202)]),
    ((2208, false, AND2), &[Tv(2121), Tv(2207)]),
    ((488, false, INV), &[Tv(487)]),
    ((586, false, AND2), &[Tv(578), Tv(584)]),
    ((588, false, OR2), &[Tv(578), Tv(580)]),
    ((1018, false, XNOR2), &[Tv(882), Tv(1017)]),
    ((1098, false, OR2), &[Tv(1088), Tv(1094)]),
    ((1099, false, XNOR2), &[Tv(1020), Tv(1095)]),
    ((1102, false, NAND2), &[Tv(1084), Tv(1097)]),
    ((1103, false, OR2), &[Tv(1085), Tv(1093)]),
];

static LEVEL_54: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((574, false, INV), &[Arg(0, 23)]),
    ((1645, false, INV), &[Tv(1644)]),
    ((1653, false, AND2), &[Tv(1647), Tv(1652)]),
    ((1662, false, INV), &[Tv(1661)]),
    ((1665, false, AND2), &[Tv(1542), Tv(1560)]),
    ((2117, false, INV), &[Tv(2116)]),
    ((2203, false, INV), &[Tv(2202)]),
    ((2205, false, AND2), &[Tv(2121), Tv(2204)]),
    ((2209, false, NAND2), &[Tv(2202), Tv(2208)]),
    ((490, false, INV), &[Tv(489)]),
    ((583, false, NAND2), &[Tv(580), Tv(581)]),
    ((587, false, AND2), &[Tv(580), Tv(586)]),
    ((589, false, INV), &[Tv(588)]),
    ((592, false, AND2), &[Tv(357), Tv(488)]),
    ((1101, false, AND2), &[Tv(1018), Tv(1099)]),
    ((1104, false, AND2), &[Tv(1102), Tv(1103)]),
    ((1106, false, AND2), &[Tv(1089), Tv(1098)]),
];

static LEVEL_55: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((860, false, INV), &[Arg(0, 53)]),
    ((1658, false, XNOR2), &[Tv(1645), Tv(1653)]),
    ((1663, false, NOR2), &[Tv(1653), Tv(1662)]),
    ((1666, false, XNOR2), &[Tv(574), Tv(1665)]),
    ((2088, false, INV), &[Tv(2087)]),
    ((2210, false, NAND2), &[Tv(2205), Tv(2209)]),
    ((2211, false, OR2), &[Tv(2121), Tv(2204)]),
    ((2214, false, AND2), &[Tv(2012), Tv(2117)]),
    ((2217, false, NAND2), &[Tv(2203), Tv(2208)]),
    ((2221, false, OR2), &[Tv(2205), Tv(2207)]),
    ((590, false, NOR2), &[Tv(587), Tv(589)]),
    ((593, false, XNOR2), &[Tv(490), Tv(592)]),
    ((595, false, NAND2), &[Tv(577), Tv(583)]),
    ((1012, false, NAND2), &[Arg(0, 52), Tv(974)]),
    ((1105, false, OR2), &[Tv(1101), Tv(1104)]),
    ((1107, false, NAND2), &[Tv(1103), Tv(1106)]),
];

static LEVEL_56: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1654, false, NAND2), &[Tv(1644), Tv(1653)]),
    ((1656, false, AND2), &[Tv(1651), Tv(1655)]),
    ((1659, false, INV), &[Tv(1658)]),
    ((1667, false, NAND2), &[Tv(1663), Tv(1666)]),
    ((2212, false, AND2), &[Tv(2210), Tv(2211)]),
    ((2215, false, XNOR2), &[Tv(2088), Tv(2214)]),
    ((2218, false, OR2), &[Tv(2199), Tv(2208)]),
    ((2222, false, AND2), &[Tv(2217), Tv(2221)]),
    ((597, false, NAND2), &[Tv(584), Tv(595)]),
    ((600, false, NAND2), &[Tv(590), Tv(593)]),
    ((601, false, XNOR2), &[Tv(581), Tv(587)]),
    ((1109, false, NAND2), &[Tv(1105), Tv(1107)]),
    ((1112, false, NAND2), &[Tv(1104), Tv(1107)]),
    ((1116, false, OR2), &[Tv(860), Tv(1012)]),
];

static LEVEL_57: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1657, false, NAND2), &[Tv(1654), Tv(1656)]),
    ((1668, false, NAND2), &[Tv(1659), Tv(1667)]),
    ((2219, false, AND2), &[Tv(2204), Tv(2218)]),
    ((2223, false, INV), &[Tv(2222)]),
    ((2224, false, NAND2), &[Tv(2212), Tv(2215)]),
    ((598, false, NAND2), &[Tv(578), Tv(597)]),
    ((602, false, NAND2), &[Tv(600), Tv(601)]),
    ((1100, false, INV), &[Tv(1099)]),
    ((1108, false, INV), &[Tv(1107)]),
    ((1110, false, NAND2), &[Tv(1018), Tv(1109)]),
    ((1113, false, OR2), &[Tv(1018), Tv(1112)]),
    ((1117, false, AND2), &[Tv(992), Tv(1116)]),
];

static LEVEL_58: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((1531, false, INV), &[Tv(1525)]),
    ((1660, false, NAND2), &[Tv(1657), Tv(1658)]),
    ((1672, false, NAND2), &[Tv(1657), Tv(1668)]),
    ((2220, false, NAND2), &[Tv(2217), Tv(2219)]),
    ((2225, false, NAND2), &[Tv(2223), Tv(2224)]),
    ((478, false, INV), &[Tv(477)]),
    ((594, false, INV), &[Tv(593)]),
    ((599, false, INV), &[Tv(598)]),
    ((603, false, AND2), &[Tv(598), Tv(602)]),
    ((1114, false, AND2), &[Tv(1110), Tv(1113)]),
    ((1118, false, XNOR2), &[Tv(871), Tv(1117)]),
    ((1119, false, NAND2), &[Tv(1100), Tv(1110)]),
    ((1120, false, NAND2), &[Tv(1101), Tv(1108)]),
];

static LEVEL_59: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1664, false, INV), &[Tv(1663)]),
    ((1673, false, NAND2), &[Tv(1666), Tv(1672)]),
    ((1679, false, AND2), &[Tv(1531), Tv(1560)]),
    ((1681, false, OR2), &[Tv(1660), Tv(1666)]),
    ((2115, false, INV), &[Tv(2114)]),
    ((2216, false, INV), &[Tv(2215)]),
    ((2226, false, NAND2), &[Tv(2220), Tv(2225)]),
    ((2233, false, AND2), &[Tv(2220), Tv(2222)]),
    ((462, false, INV), &[Tv(461)]),
    ((479, false, AND2), &[Tv(357), Tv(478)]),
    ((591, false, INV), &[Tv(590)]),
    ((604, false, OR2), &[Tv(594), Tv(603)]),
    ((609, false, OR2), &[Tv(599), Tv(601)]),
    ((1121, false, AND2), &[Tv(1119), Tv(1120)]),
    ((1123, false, AND2), &[Tv(1114), Tv(1118)]),
    ((1125, false, AND2), &[Tv(1105), Tv(1112)]),
];

static LEVEL_60: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((1671, false, OR2), &[Tv(1657), Tv(1667)]),
    ((1674, false, NAND2), &[Tv(1664), Tv(1673)]),
    ((1680, false, XNOR2), &[Tv(563), Tv(1679)]),
    ((1682, false, AND2), &[Tv(1673), Tv(1681)]),
    ((2093, false, INV), &[Tv(2092)]),
    ((2213, false, INV), &[Tv(2212)]),
    ((2227, false, NAND2), &[Tv(2215), Tv(2226)]),
    ((2231, false, AND2), &[Tv(2012), Tv(2115)]),
    ((2234, false, NAND2), &[Tv(2216), Tv(2233)]),
    ((480, false, XNOR2), &[Tv(462), Tv(479)]),
    ((605, false, NAND2), &[Tv(591), Tv(604)]),
    ((606, false, OR2), &[Tv(598), Tv(600)]),
    ((610, false, XNOR2), &[Tv(593), Tv(603)]),
    ((613, false, AND2), &[Tv(602), Tv(609)]),
    ((1124, false, OR2), &[Tv(1121), Tv(1123)]),
    ((1126, false, NAND2), &[Tv(1105), Tv(1112)]),
    ((1128, false, NAND2), &[Tv(1120), Tv(1125)]),
];

static LEVEL_61: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1669, false, AND2), &[Tv(1660), Tv(1668)]),
    ((1675, false, AND2), &[Tv(1671), Tv(1674)]),
    ((1684, false, AND2), &[Tv(1680), Tv(1682)]),
    ((2228, false, NAND2), &[Tv(2213), Tv(2227)]),
    ((2229, false, OR2), &[Tv(2220), Tv(2224)]),
    ((2232, false, XNOR2), &[Tv(2093), Tv(2231)]),
    ((2235, false, AND2), &[Tv(2227), Tv(2234)]),
    ((608, false, NAND2), &[Tv(605), Tv(606)]),
    ((611, false, NAND2), &[Tv(480), Tv(610)]),
    ((614, false, NAND2), &[Tv(606), Tv(613)]),
    ((1129, false, NAND2), &[Tv(1124), Tv(1128)]),
    ((1135, false, NAND2), &[Tv(1121), Tv(1126)]),
];

static LEVEL_62: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1670, false, NAND2), &[Tv(1660), Tv(1668)]),
    ((1678, false, NAND2), &[Tv(1669), Tv(1671)]),
    ((1686, false, OR2), &[Tv(1675), Tv(1684)]),
    ((2230, false, AND2), &[Tv(2228), Tv(2229)]),
    ((2237, false, AND2), &[Tv(2232), Tv(2235)]),
    ((2239, false, NAND2), &[Tv(2225), Tv(2229)]),
    ((473, false, INV), &[Tv(472)]),
    ((612, false, NAND2), &[Tv(608), Tv(611)]),
    ((615, false, INV), &[Tv(614)]),
    ((1014, false, AND2), &[Tv(992), Tv(1012)]),
    ((1115, false, INV), &[Tv(1114)]),
    ((1127, false, AND2), &[Tv(1120), Tv(1125)]),
    ((1130, false, NAND2), &[Tv(1118), Tv(1129)]),
    ((1136, false, OR2), &[Tv(1118), Tv(1135)]),
];

static LEVEL_63: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1561, false, INV), &[Tv(1560)]),
    ((1676, false, NAND2), &[Tv(1670), Tv(1675)]),
    ((1692, false, NAND2), &[Tv(1678), Tv(1686)]),
    ((2238, false, OR2), &[Tv(2230), Tv(2237)]),
    ((2240, false, OR2), &[Tv(2233), Tv(2239)]),
    ((476, false, INV), &[Tv(474)]),
    ((481, false, INV), &[Tv(480)]),
    ((616, false, AND2), &[Tv(612), Tv(614)]),
    ((621, false, AND2), &[Tv(357), Tv(473)]),
    ((625, false, AND2), &[Tv(611), Tv(615)]),
    ((1015, false, XNOR2), &[Tv(860), Tv(1014)]),
    ((1131, false, NAND2), &[Tv(1115), Tv(1130)]),
    ((1132, false, NAND2), &[Tv(1123), Tv(1127)]),
    ((1137, false, AND2), &[Tv(1130), Tv(1136)]),
];

static LEVEL_64: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((552, false, INV), &[Arg(0, 21)]),
    ((1677, false, AND2), &[Tv(1669), Tv(1671)]),
    ((1683, false, INV), &[Tv(1682)]),
    ((1689, false, NOR2), &[Tv(1519), Tv(1561)]),
    ((1693, false, NAND2), &[Tv(1680), Tv(1692)]),
    ((1694, false, OR2), &[Tv(1676), Tv(1680)]),
    ((2113, false, INV), &[Tv(2112)]),
    ((2242, false, NAND2), &[Tv(2230), Tv(2240)]),
    ((2246, false, NAND2), &[Tv(2238), Tv(2240)]),
    ((617, false, NOR2), &[Tv(481), Tv(616)]),
    ((619, false, XNOR2), &[Tv(480), Tv(616)]),
    ((622, false, XNOR2), &[Tv(476), Tv(621)]),
    ((626, false, OR2), &[Tv(608), Tv(625)]),
    ((1134, false, NAND2), &[Tv(1131), Tv(1132)]),
    ((1138, false, NAND2), &[Tv(1015), Tv(1137)]),
    ((1140, false, AND2), &[Tv(1124), Tv(1135)]),
];

static LEVEL_65: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1685, false, NAND2), &[Tv(1677), Tv(1684)]),
    ((1690, false, XNOR2), &[Tv(552), Tv(1689)]),
    ((1695, false, AND2), &[Tv(1693), Tv(1694)]),
    ((1698, false, NAND2), &[Tv(1683), Tv(1693)]),
    ((2095, false, INV), &[Tv(2094)]),
    ((2236, false, INV), &[Tv(2235)]),
    ((2241, false, INV), &[Tv(2240)]),
    ((2247, false, NAND2), &[Tv(2232), Tv(2246)]),
    ((2251, false, AND2), &[Tv(2012), Tv(2113)]),
    ((2253, false, OR2), &[Tv(2232), Tv(2242)]),
    ((624, false, NAND2), &[Tv(619), Tv(622)]),
    ((627, false, NAND2), &[Tv(612), Tv(626)]),
    ((631, false, XNOR2), &[Tv(610), Tv(617)]),
    ((975, false, INV), &[Tv(974)]),
    ((1139, false, NAND2), &[Tv(1134), Tv(1138)]),
    ((1141, false, NAND2), &[Tv(1132), Tv(1140)]),
];

static LEVEL_66: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((849, false, INV), &[Arg(0, 52)]),
    ((1687, false, AND2), &[Tv(1685), Tv(1686)]),
    ((1697, false, NAND2), &[Tv(1690), Tv(1695)]),
    ((1700, false, NAND2), &[Tv(1685), Tv(1698)]),
    ((2243, false, AND2), &[Tv(2238), Tv(2242)]),
    ((2245, false, NAND2), &[Tv(2237), Tv(2241)]),
    ((2248, false, NAND2), &[Tv(2236), Tv(2247)]),
    ((2252, false, XNOR2), &[Tv(2095), Tv(2251)]),
    ((2254, false, AND2), &[Tv(2247), Tv(2253)]),
    ((628, false, INV), &[Tv(627)]),
    ((632, false, NAND2), &[Tv(624), Tv(631)]),
    ((1009, false, AND2), &[Tv(975), Tv(992)]),
    ((1016, false, INV), &[Tv(1015)]),
    ((1142, false, AND2), &[Tv(1139), Tv(1141)]),
];

static LEVEL_67: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1688, false, NAND2), &[Tv(1676), Tv(1687)]),
    ((1699, false, AND2), &[Tv(1685), Tv(1698)]),
    ((1701, false, NAND2), &[Tv(1697), Tv(1700)]),
    ((2244, false, INV), &[Tv(2243)]),
    ((2249, false, AND2), &[Tv(2245), Tv(2248)]),
    ((2256, false, AND2), &[Tv(2252), Tv(2254)]),
    ((2258, false, NAND2), &[Tv(2243), Tv(2245)]),
    ((623, false, INV), &[Tv(622)]),
    ((633, false, AND2), &[Tv(627), Tv(632)]),
    ((637, false, OR2), &[Tv(628), Tv(631)]),
    ((1010, false, XNOR2), &[Tv(849), Tv(1009)]),
    ((1143, false, NOR2), &[Tv(1016), Tv(1142)]),
    ((1145, false, OR2), &[Tv(1134), Tv(1140)]),
    ((1146, false, XNOR2), &[Tv(1015), Tv(1142)]),
];

static LEVEL_68: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1691, false, INV), &[Tv(1690)]),
    ((1702, false, AND2), &[Tv(1688), Tv(1701)]),
    ((1705, false, NAND2), &[Tv(1688), Tv(1699)]),
    ((2250, false, NAND2), &[Tv(2244), Tv(2249)]),
    ((2259, false, NAND2), &[Tv(2256), Tv(2258)]),
    ((471, false, INV), &[Tv(470)]),
    ((620, false, INV), &[Tv(619)]),
    ((630, false, OR2), &[Tv(624), Tv(627)]),
    ((634, false, OR2), &[Tv(623), Tv(633)]),
    ((638, false, AND2), &[Tv(632), Tv(637)]),
    ((1148, false, NAND2), &[Tv(1010), Tv(1146)]),
    ((1149, false, OR2), &[Tv(1138), Tv(1141)]),
    ((1150, false, XNOR2), &[Tv(1137), Tv(1143)]),
    ((1152, false, AND2), &[Tv(1139), Tv(1145)]),
];

static LEVEL_69: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((1513, false, INV), &[Tv(1504)]),
    ((1696, false, INV), &[Tv(1695)]),
    ((1703, false, NOR2), &[Tv(1691), Tv(1702)]),
    ((1704, false, OR2), &[Tv(1688), Tv(1697)]),
    ((1706, false, AND2), &[Tv(1701), Tv(1705)]),
    ((2260, false, AND2), &[Tv(2250), Tv(2259)]),
    ((465, false, INV), &[Tv(463)]),
    ((635, false, NAND2), &[Tv(620), Tv(634)]),
    ((642, false, AND2), &[Tv(357), Tv(471)]),
    ((644, false, NAND2), &[Tv(630), Tv(638)]),
    ((645, false, XNOR2), &[Tv(622), Tv(633)]),
    ((1151, false, NAND2), &[Tv(1148), Tv(1150)]),
    ((1153, false, NAND2), &[Tv(1149), Tv(1152)]),
];

static LEVEL_70: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((541, false, INV), &[Arg(0, 20)]),
    ((1707, false, NAND2), &[Tv(1704), Tv(1706)]),
    ((1711, false, AND2), &[Tv(1513), Tv(1560)]),
    ((1713, false, XNOR2), &[Tv(1696), Tv(1703)]),
    ((2109, false, INV), &[Tv(2108)]),
    ((2255, false, INV), &[Tv(2254)]),
    ((2257, false, INV), &[Tv(2256)]),
    ((2261, false, NAND2), &[Tv(2252), Tv(2260)]),
    ((2272, false, OR2), &[Tv(2249), Tv(2256)]),
    ((636, false, AND2), &[Tv(630), Tv(635)]),
    ((639, false, INV), &[Tv(638)]),
    ((643, false, XNOR2), &[Tv(465), Tv(642)]),
    ((647, false, NAND2), &[Tv(644), Tv(645)]),
    ((973, false, INV), &[Tv(972)]),
    ((1011, false, INV), &[Tv(1010)]),
    ((1154, false, INV), &[Tv(1153)]),
    ((1156, false, AND2), &[Tv(1151), Tv(1153)]),
];

static LEVEL_71: [((usize, bool, CellType), &[GateInput]); 18] = [
    ((838, false, INV), &[Arg(0, 51)]),
    ((1709, false, XNOR2), &[Tv(1690), Tv(1702)]),
    ((1712, false, XNOR2), &[Tv(541), Tv(1711)]),
    ((1714, false, NAND2), &[Tv(1707), Tv(1713)]),
    ((2111, false, INV), &[Tv(2110)]),
    ((2262, false, OR2), &[Tv(2250), Tv(2252)]),
    ((2265, false, AND2), &[Tv(2012), Tv(2109)]),
    ((2268, false, NAND2), &[Tv(2255), Tv(2261)]),
    ((2269, false, OR2), &[Tv(2257), Tv(2258)]),
    ((2273, false, NAND2), &[Tv(2250), Tv(2272)]),
    ((467, false, INV), &[Tv(466)]),
    ((641, false, NAND2), &[Tv(636), Tv(639)]),
    ((648, false, AND2), &[Tv(643), Tv(645)]),
    ((650, false, AND2), &[Tv(643), Tv(647)]),
    ((1006, false, AND2), &[Tv(973), Tv(992)]),
    ((1147, false, INV), &[Tv(1146)]),
    ((1157, false, OR2), &[Tv(1011), Tv(1156)]),
    ((1158, false, OR2), &[Tv(1150), Tv(1154)]),
];

static LEVEL_72: [((usize, bool, CellType), &[GateInput]); 18] = [
    ((1494, false, INV), &[Tv(1483)]),
    ((1710, false, NAND2), &[Tv(1707), Tv(1709)]),
    ((1715, false, AND2), &[Tv(1712), Tv(1714)]),
    ((2263, false, AND2), &[Tv(2261), Tv(2262)]),
    ((2266, false, XNOR2), &[Tv(2111), Tv(2265)]),
    ((2270, false, AND2), &[Tv(2268), Tv(2269)]),
    ((2274, false, INV), &[Tv(2273)]),
    ((469, false, INV), &[Tv(468)]),
    ((646, false, INV), &[Tv(645)]),
    ((649, false, INV), &[Tv(648)]),
    ((652, false, NAND2), &[Tv(641), Tv(650)]),
    ((653, false, OR2), &[Tv(641), Tv(643)]),
    ((656, false, AND2), &[Tv(357), Tv(467)]),
    ((1007, false, XNOR2), &[Tv(838), Tv(1006)]),
    ((1159, false, XNOR2), &[Tv(1010), Tv(1156)]),
    ((1161, false, NAND2), &[Tv(1147), Tv(1157)]),
    ((1162, false, OR2), &[Tv(1148), Tv(1153)]),
    ((1165, false, AND2), &[Tv(1151), Tv(1158)]),
];

static LEVEL_73: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((530, false, INV), &[Arg(0, 19)]),
    ((1708, false, INV), &[Tv(1707)]),
    ((1716, false, NAND2), &[Tv(1710), Tv(1715)]),
    ((1717, false, AND2), &[Tv(1709), Tv(1712)]),
    ((1719, false, OR2), &[Tv(1712), Tv(1714)]),
    ((1722, false, AND2), &[Tv(1494), Tv(1560)]),
    ((2267, false, NAND2), &[Tv(2263), Tv(2266)]),
    ((2275, false, AND2), &[Tv(2269), Tv(2274)]),
    ((2278, false, NAND2), &[Tv(2270), Tv(2273)]),
    ((654, false, AND2), &[Tv(652), Tv(653)]),
    ((657, false, XNOR2), &[Tv(469), Tv(656)]),
    ((659, false, OR2), &[Tv(644), Tv(649)]),
    ((660, false, OR2), &[Tv(636), Tv(648)]),
    ((666, false, NAND2), &[Tv(646), Tv(652)]),
    ((1160, false, AND2), &[Tv(1007), Tv(1159)]),
    ((1163, false, AND2), &[Tv(1161), Tv(1162)]),
    ((1167, false, NAND2), &[Tv(1162), Tv(1165)]),
];

static LEVEL_74: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1718, false, OR2), &[Tv(1713), Tv(1717)]),
    ((1720, false, AND2), &[Tv(1716), Tv(1719)]),
    ((1723, false, XNOR2), &[Tv(530), Tv(1722)]),
    ((1726, false, OR2), &[Tv(1709), Tv(1715)]),
    ((1727, false, NAND2), &[Tv(1708), Tv(1717)]),
    ((2107, false, INV), &[Tv(2106)]),
    ((2276, false, OR2), &[Tv(2267), Tv(2275)]),
    ((2281, false, AND2), &[Tv(2266), Tv(2278)]),
    ((661, false, AND2), &[Tv(641), Tv(660)]),
    ((665, false, AND2), &[Tv(654), Tv(657)]),
    ((667, false, AND2), &[Tv(659), Tv(666)]),
    ((1008, false, INV), &[Tv(1007)]),
    ((1164, false, OR2), &[Tv(1160), Tv(1163)]),
    ((1170, false, AND2), &[Tv(1163), Tv(1167)]),
];

static LEVEL_75: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1725, false, NAND2), &[Tv(1720), Tv(1723)]),
    ((1728, false, NAND2), &[Tv(1726), Tv(1727)]),
    ((1730, false, AND2), &[Tv(1714), Tv(1718)]),
    ((2097, false, INV), &[Tv(2096)]),
    ((2282, false, AND2), &[Tv(2276), Tv(2281)]),
    ((2285, false, NOR2), &[Tv(2266), Tv(2278)]),
    ((2288, false, AND2), &[Tv(2012), Tv(2107)]),
    ((663, false, NAND2), &[Tv(659), Tv(661)]),
    ((668, false, OR2), &[Tv(665), Tv(667)]),
    ((971, false, INV), &[Tv(970)]),
    ((1168, false, NAND2), &[Tv(1164), Tv(1167)]),
    ((1171, false, NAND2), &[Tv(1008), Tv(1170)]),
];

static LEVEL_76: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((827, false, INV), &[Arg(0, 50)]),
    ((1729, false, NAND2), &[Tv(1725), Tv(1728)]),
    ((1731, false, NAND2), &[Tv(1727), Tv(1730)]),
    ((2264, false, INV), &[Tv(2263)]),
    ((2271, false, NAND2), &[Tv(2267), Tv(2270)]),
    ((2286, false, NOR2), &[Tv(2282), Tv(2285)]),
    ((2289, false, XNOR2), &[Tv(2097), Tv(2288)]),
    ((454, false, INV), &[Tv(452)]),
    ((658, false, INV), &[Tv(657)]),
    ((664, false, INV), &[Tv(663)]),
    ((669, false, NAND2), &[Tv(663), Tv(668)]),
    ((675, false, AND2), &[Tv(663), Tv(667)]),
    ((1003, false, AND2), &[Tv(971), Tv(992)]),
    ((1169, false, AND2), &[Tv(1007), Tv(1168)]),
    ((1172, false, INV), &[Tv(1171)]),
];

static LEVEL_77: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((1473, false, INV), &[Tv(1463)]),
    ((1724, false, INV), &[Tv(1723)]),
    ((1733, false, AND2), &[Tv(1729), Tv(1731)]),
    ((2277, false, NAND2), &[Tv(2271), Tv(2276)]),
    ((2283, false, XNOR2), &[Tv(2264), Tv(2282)]),
    ((2290, false, AND2), &[Tv(2286), Tv(2289)]),
    ((443, false, INV), &[Tv(441)]),
    ((455, false, AND2), &[Tv(357), Tv(454)]),
    ((655, false, INV), &[Tv(654)]),
    ((670, false, NAND2), &[Tv(657), Tv(669)]),
    ((672, false, NAND2), &[Tv(664), Tv(665)]),
    ((676, false, NAND2), &[Tv(658), Tv(675)]),
    ((1004, false, XNOR2), &[Tv(827), Tv(1003)]),
    ((1173, false, NOR2), &[Tv(1169), Tv(1172)]),
    ((1176, false, NAND2), &[Tv(1159), Tv(1169)]),
];

static LEVEL_78: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((519, false, INV), &[Arg(0, 18)]),
    ((1584, false, AND2), &[Tv(1473), Tv(1560)]),
    ((1721, false, INV), &[Tv(1720)]),
    ((1732, false, INV), &[Tv(1731)]),
    ((1734, false, OR2), &[Tv(1724), Tv(1733)]),
    ((2279, false, NAND2), &[Tv(2277), Tv(2278)]),
    ((2292, false, OR2), &[Tv(2283), Tv(2290)]),
    ((456, false, XNOR2), &[Tv(443), Tv(455)]),
    ((671, false, NAND2), &[Tv(655), Tv(670)]),
    ((677, false, AND2), &[Tv(670), Tv(676)]),
    ((681, false, NAND2), &[Tv(668), Tv(672)]),
    ((1175, false, NAND2), &[Tv(1004), Tv(1173)]),
    ((1178, false, XNOR2), &[Tv(1159), Tv(1169)]),
    ((1180, false, NAND2), &[Tv(1164), Tv(1176)]),
];

static LEVEL_79: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((1585, false, XNOR2), &[Tv(519), Tv(1584)]),
    ((1735, false, NAND2), &[Tv(1721), Tv(1734)]),
    ((1736, false, OR2), &[Tv(1725), Tv(1731)]),
    ((1738, false, OR2), &[Tv(1728), Tv(1732)]),
    ((1739, false, XNOR2), &[Tv(1723), Tv(1733)]),
    ((2105, false, INV), &[Tv(2104)]),
    ((2284, false, NAND2), &[Tv(2279), Tv(2283)]),
    ((4, false, NAND2), &[Tv(2279), Tv(2292)]),
    ((674, false, NAND2), &[Tv(671), Tv(672)]),
    ((679, false, NAND2), &[Tv(456), Tv(677)]),
    ((682, false, OR2), &[Tv(675), Tv(681)]),
    ((1179, false, NAND2), &[Tv(1175), Tv(1178)]),
    ((1181, false, OR2), &[Tv(1170), Tv(1180)]),
];

static LEVEL_80: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1737, false, NAND2), &[Tv(1735), Tv(1736)]),
    ((1741, false, NAND2), &[Tv(1585), Tv(1739)]),
    ((1743, false, AND2), &[Tv(1729), Tv(1738)]),
    ((2099, false, INV), &[Tv(2098)]),
    ((2280, false, INV), &[Tv(2279)]),
    ((2287, false, INV), &[Tv(2286)]),
    ((2, false, AND2), &[Tv(2012), Tv(2105)]),
    ((5, false, NAND2), &[Tv(2289), Tv(4)]),
    ((6, false, OR2), &[Tv(2284), Tv(2289)]),
    ((680, false, NAND2), &[Tv(674), Tv(679)]),
    ((683, false, INV), &[Tv(682)]),
    ((969, false, INV), &[Tv(968)]),
    ((1005, false, INV), &[Tv(1004)]),
    ((1183, false, AND2), &[Tv(1179), Tv(1181)]),
];

static LEVEL_81: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((816, false, INV), &[Arg(0, 49)]),
    ((1742, false, NAND2), &[Tv(1737), Tv(1741)]),
    ((1744, false, INV), &[Tv(1743)]),
    ((1745, false, NAND2), &[Tv(1736), Tv(1743)]),
    ((2291, false, NAND2), &[Tv(2280), Tv(2290)]),
    ((3, false, XNOR2), &[Tv(2099), Tv(2)]),
    ((7, false, AND2), &[Tv(5), Tv(6)]),
    ((11, false, NAND2), &[Tv(2287), Tv(5)]),
    ((449, false, INV), &[Tv(448)]),
    ((457, false, INV), &[Tv(456)]),
    ((685, false, AND2), &[Tv(680), Tv(682)]),
    ((687, false, OR2), &[Tv(674), Tv(683)]),
    ((1174, false, INV), &[Tv(1173)]),
    ((1182, false, INV), &[Tv(1181)]),
    ((1184, false, OR2), &[Tv(1005), Tv(1183)]),
    ((1189, false, AND2), &[Tv(969), Tv(992)]),
];

static LEVEL_82: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((1452, false, INV), &[Tv(1441)]),
    ((1746, false, NAND2), &[Tv(1742), Tv(1745)]),
    ((1748, false, NAND2), &[Tv(1735), Tv(1744)]),
    ((2293, false, AND2), &[Tv(2291), Tv(2292)]),
    ((9, false, AND2), &[Tv(3), Tv(7)]),
    ((12, false, AND2), &[Tv(2291), Tv(11)]),
    ((451, false, INV), &[Tv(450)]),
    ((678, false, INV), &[Tv(677)]),
    ((686, false, NOR2), &[Tv(457), Tv(685)]),
    ((689, false, AND2), &[Tv(357), Tv(449)]),
    ((693, false, OR2), &[Tv(679), Tv(682)]),
    ((697, false, AND2), &[Tv(680), Tv(687)]),
    ((1185, false, OR2), &[Tv(1178), Tv(1182)]),
    ((1186, false, XNOR2), &[Tv(1004), Tv(1183)]),
    ((1190, false, XNOR2), &[Tv(816), Tv(1189)]),
    ((1194, false, OR2), &[Tv(1175), Tv(1181)]),
    ((1198, false, NAND2), &[Tv(1174), Tv(1184)]),
];

static LEVEL_83: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((508, false, INV), &[Arg(0, 17)]),
    ((1740, false, INV), &[Tv(1739)]),
    ((1747, false, NAND2), &[Tv(1585), Tv(1746)]),
    ((1749, false, OR2), &[Tv(1585), Tv(1748)]),
    ((1752, false, AND2), &[Tv(1452), Tv(1560)]),
    ((0, false, NAND2), &[Tv(2284), Tv(2293)]),
    ((13, false, OR2), &[Tv(9), Tv(12)]),
    ((688, false, XNOR2), &[Tv(456), Tv(685)]),
    ((690, false, XNOR2), &[Tv(451), Tv(689)]),
    ((694, false, XNOR2), &[Tv(678), Tv(686)]),
    ((698, false, NAND2), &[Tv(693), Tv(697)]),
    ((1191, false, AND2), &[Tv(1186), Tv(1190)]),
    ((1192, false, AND2), &[Tv(1179), Tv(1185)]),
    ((1200, false, AND2), &[Tv(1194), Tv(1198)]),
];

static LEVEL_84: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((1750, false, AND2), &[Tv(1747), Tv(1749)]),
    ((1753, false, XNOR2), &[Tv(508), Tv(1752)]),
    ((1756, false, OR2), &[Tv(1741), Tv(1745)]),
    ((1760, false, NAND2), &[Tv(1740), Tv(1747)]),
    ((14, false, NAND2), &[Tv(0), Tv(13)]),
    ((691, false, AND2), &[Tv(688), Tv(690)]),
    ((692, false, NAND2), &[Tv(688), Tv(690)]),
    ((701, false, NAND2), &[Tv(694), Tv(698)]),
    ((1196, false, NAND2), &[Tv(1192), Tv(1194)]),
    ((1202, false, OR2), &[Tv(1191), Tv(1200)]),
];

static LEVEL_85: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1755, false, NAND2), &[Tv(1750), Tv(1753)]),
    ((1757, false, AND2), &[Tv(1742), Tv(1748)]),
    ((1761, false, NAND2), &[Tv(1756), Tv(1760)]),
    ((2101, false, INV), &[Tv(2100)]),
    ((1, false, INV), &[Tv(0)]),
    ((8, false, INV), &[Tv(7)]),
    ((15, false, NAND2), &[Tv(3), Tv(14)]),
    ((18, false, NAND2), &[Tv(0), Tv(12)]),
    ((445, false, INV), &[Tv(444)]),
    ((696, false, NAND2), &[Tv(692), Tv(694)]),
    ((699, false, NAND2), &[Tv(691), Tv(698)]),
    ((703, false, AND2), &[Tv(690), Tv(701)]),
    ((1193, false, NAND2), &[Tv(1179), Tv(1185)]),
    ((1206, false, NAND2), &[Tv(1196), Tv(1202)]),
];

static LEVEL_86: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((1758, false, NAND2), &[Tv(1756), Tv(1757)]),
    ((1762, false, NAND2), &[Tv(1755), Tv(1761)]),
    ((2103, false, INV), &[Tv(2102)]),
    ((10, false, NAND2), &[Tv(1), Tv(9)]),
    ((16, false, NAND2), &[Tv(8), Tv(15)]),
    ((19, false, OR2), &[Tv(3), Tv(18)]),
    ((21, false, AND2), &[Tv(2012), Tv(2101)]),
    ((25, false, AND2), &[Tv(13), Tv(18)]),
    ((447, false, INV), &[Tv(446)]),
    ((700, false, NAND2), &[Tv(696), Tv(699)]),
    ((704, false, NAND2), &[Tv(699), Tv(703)]),
    ((708, false, OR2), &[Tv(690), Tv(701)]),
    ((711, false, AND2), &[Tv(357), Tv(445)]),
    ((1187, false, INV), &[Tv(1186)]),
    ((1195, false, AND2), &[Tv(1192), Tv(1194)]),
    ((1201, false, NAND2), &[Tv(1193), Tv(1200)]),
    ((1207, false, NAND2), &[Tv(1190), Tv(1206)]),
];

static LEVEL_87: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((1430, false, INV), &[Tv(1419)]),
    ((1754, false, INV), &[Tv(1753)]),
    ((1768, false, AND2), &[Tv(1758), Tv(1762)]),
    ((17, false, AND2), &[Tv(10), Tv(16)]),
    ((20, false, AND2), &[Tv(15), Tv(19)]),
    ((22, false, XNOR2), &[Tv(2103), Tv(21)]),
    ((26, false, NAND2), &[Tv(10), Tv(25)]),
    ((702, false, NAND2), &[Tv(700), Tv(701)]),
    ((705, false, XNOR2), &[Tv(688), Tv(704)]),
    ((709, false, AND2), &[Tv(704), Tv(708)]),
    ((712, false, XNOR2), &[Tv(447), Tv(711)]),
    ((967, false, INV), &[Tv(966)]),
    ((1197, false, NAND2), &[Tv(1191), Tv(1195)]),
    ((1203, false, AND2), &[Tv(1201), Tv(1202)]),
    ((1215, false, NAND2), &[Tv(1187), Tv(1207)]),
];

static LEVEL_88: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((497, false, INV), &[Arg(0, 16)]),
    ((805, false, INV), &[Arg(0, 48)]),
    ((1581, false, AND2), &[Tv(1430), Tv(1560)]),
    ((1751, false, INV), &[Tv(1750)]),
    ((1769, false, OR2), &[Tv(1754), Tv(1768)]),
    ((23, false, AND2), &[Tv(20), Tv(22)]),
    ((28, false, NAND2), &[Tv(17), Tv(26)]),
    ((707, false, NAND2), &[Tv(702), Tv(705)]),
    ((713, false, AND2), &[Tv(709), Tv(712)]),
    ((1204, false, NAND2), &[Tv(1197), Tv(1203)]),
    ((1208, false, OR2), &[Tv(1190), Tv(1201)]),
    ((1211, false, AND2), &[Tv(967), Tv(992)]),
    ((1216, false, AND2), &[Tv(1197), Tv(1215)]),
];

static LEVEL_89: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1582, false, XNOR2), &[Tv(497), Tv(1581)]),
    ((1759, false, OR2), &[Tv(1755), Tv(1758)]),
    ((1763, false, OR2), &[Tv(1757), Tv(1761)]),
    ((1770, false, NAND2), &[Tv(1751), Tv(1769)]),
    ((1772, false, XNOR2), &[Tv(1753), Tv(1768)]),
    ((2082, false, INV), &[Tv(2081)]),
    ((27, false, INV), &[Tv(26)]),
    ((31, false, NAND2), &[Tv(23), Tv(26)]),
    ((33, false, AND2), &[Tv(22), Tv(28)]),
    ((437, false, INV), &[Tv(436)]),
    ((716, false, NAND2), &[Tv(702), Tv(713)]),
    ((720, false, AND2), &[Tv(707), Tv(712)]),
    ((722, false, OR2), &[Tv(707), Tv(712)]),
    ((1209, false, AND2), &[Tv(1207), Tv(1208)]),
    ((1212, false, XNOR2), &[Tv(805), Tv(1211)]),
    ((1217, false, NAND2), &[Tv(1204), Tv(1216)]),
];

static LEVEL_90: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((1764, false, AND2), &[Tv(1762), Tv(1763)]),
    ((1771, false, AND2), &[Tv(1759), Tv(1770)]),
    ((1774, false, AND2), &[Tv(1582), Tv(1772)]),
    ((2080, false, INV), &[Tv(2079)]),
    ((2083, false, AND2), &[Tv(2012), Tv(2082)]),
    ((29, false, NAND2), &[Tv(23), Tv(27)]),
    ((34, false, NAND2), &[Tv(31), Tv(33)]),
    ((35, false, OR2), &[Tv(22), Tv(28)]),
    ((39, false, OR2), &[Tv(20), Tv(33)]),
    ((429, false, INV), &[Tv(428)]),
    ((438, false, AND2), &[Tv(357), Tv(437)]),
    ((714, false, NAND2), &[Tv(709), Tv(712)]),
    ((721, false, AND2), &[Tv(716), Tv(720)]),
    ((723, false, INV), &[Tv(722)]),
    ((965, false, INV), &[Tv(964)]),
    ((1228, false, NAND2), &[Tv(1204), Tv(1209)]),
    ((1229, false, AND2), &[Tv(1212), Tv(1217)]),
];

static LEVEL_91: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((794, false, INV), &[Arg(0, 47)]),
    ((1765, false, NAND2), &[Tv(1762), Tv(1763)]),
    ((1767, false, NAND2), &[Tv(1759), Tv(1764)]),
    ((1775, false, OR2), &[Tv(1771), Tv(1774)]),
    ((2084, false, XNOR2), &[Tv(2080), Tv(2083)]),
    ((36, false, AND2), &[Tv(34), Tv(35)]),
    ((40, false, AND2), &[Tv(29), Tv(39)]),
    ((439, false, XNOR2), &[Tv(429), Tv(438)]),
    ((715, false, NAND2), &[Tv(705), Tv(714)]),
    ((724, false, NOR2), &[Tv(721), Tv(723)]),
    ((1205, false, INV), &[Tv(1204)]),
    ((1213, false, AND2), &[Tv(1209), Tv(1212)]),
    ((1225, false, AND2), &[Tv(965), Tv(992)]),
    ((1230, false, NAND2), &[Tv(1228), Tv(1229)]),
    ((1231, false, OR2), &[Tv(1212), Tv(1217)]),
];

static LEVEL_92: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1408, false, INV), &[Tv(1397)]),
    ((1583, false, INV), &[Tv(1582)]),
    ((1776, false, NAND2), &[Tv(1767), Tv(1775)]),
    ((1778, false, AND2), &[Tv(1765), Tv(1771)]),
    ((24, false, OR2), &[Tv(17), Tv(23)]),
    ((30, false, AND2), &[Tv(28), Tv(29)]),
    ((38, false, NAND2), &[Tv(2084), Tv(36)]),
    ((41, false, INV), &[Tv(40)]),
    ((718, false, NAND2), &[Tv(715), Tv(716)]),
    ((725, false, NAND2), &[Tv(439), Tv(724)]),
    ((727, false, XNOR2), &[Tv(709), Tv(721)]),
    ((1214, false, NAND2), &[Tv(1205), Tv(1213)]),
    ((1218, false, OR2), &[Tv(1213), Tv(1216)]),
    ((1226, false, XNOR2), &[Tv(794), Tv(1225)]),
    ((1233, false, AND2), &[Tv(1230), Tv(1231)]),
    ((1236, false, OR2), &[Tv(1209), Tv(1229)]),
];

static LEVEL_93: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((486, false, INV), &[Arg(0, 15)]),
    ((1766, false, AND2), &[Tv(1759), Tv(1764)]),
    ((1773, false, INV), &[Tv(1772)]),
    ((1777, false, NAND2), &[Tv(1582), Tv(1776)]),
    ((1779, false, NAND2), &[Tv(1583), Tv(1778)]),
    ((1782, false, AND2), &[Tv(1408), Tv(1560)]),
    ((32, false, NAND2), &[Tv(24), Tv(30)]),
    ((42, false, NAND2), &[Tv(38), Tv(41)]),
    ((435, false, INV), &[Tv(434)]),
    ((710, false, INV), &[Tv(709)]),
    ((719, false, NAND2), &[Tv(707), Tv(718)]),
    ((729, false, NAND2), &[Tv(725), Tv(727)]),
    ((1219, false, AND2), &[Tv(1217), Tv(1218)]),
    ((1223, false, AND2), &[Arg(0, 44), Tv(962)]),
    ((1235, false, NAND2), &[Tv(1226), Tv(1233)]),
    ((1237, false, NAND2), &[Tv(1214), Tv(1236)]),
];

static LEVEL_94: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((1780, false, AND2), &[Tv(1777), Tv(1779)]),
    ((1783, false, XNOR2), &[Tv(486), Tv(1782)]),
    ((1785, false, NAND2), &[Tv(1766), Tv(1774)]),
    ((1790, false, NAND2), &[Tv(1773), Tv(1777)]),
    ((2075, false, INV), &[Tv(2074)]),
    ((43, false, NAND2), &[Tv(32), Tv(42)]),
    ((45, false, NAND2), &[Tv(32), Tv(40)]),
    ((433, false, INV), &[Tv(432)]),
    ((440, false, INV), &[Tv(439)]),
    ((726, false, XNOR2), &[Tv(710), Tv(721)]),
    ((730, false, AND2), &[Tv(719), Tv(729)]),
    ((735, false, AND2), &[Tv(357), Tv(435)]),
    ((1220, false, NAND2), &[Tv(1214), Tv(1219)]),
    ((1224, false, NAND2), &[Arg(0, 45), Tv(1223)]),
    ((1238, false, NAND2), &[Tv(1235), Tv(1237)]),
];

static LEVEL_95: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1784, false, AND2), &[Tv(1780), Tv(1783)]),
    ((1786, false, NAND2), &[Tv(1775), Tv(1785)]),
    ((1791, false, AND2), &[Tv(1785), Tv(1790)]),
    ((2068, false, INV), &[Tv(2067)]),
    ((2076, false, AND2), &[Tv(2012), Tv(2075)]),
    ((37, false, INV), &[Tv(36)]),
    ((44, false, NAND2), &[Tv(2084), Tv(43)]),
    ((46, false, OR2), &[Tv(2084), Tv(45)]),
    ((731, false, NOR2), &[Tv(440), Tv(730)]),
    ((732, false, NAND2), &[Tv(719), Tv(726)]),
    ((733, false, XNOR2), &[Tv(439), Tv(730)]),
    ((736, false, XNOR2), &[Tv(433), Tv(735)]),
    ((1222, false, INV), &[Tv(1220)]),
    ((1227, false, INV), &[Tv(1226)]),
    ((1239, false, AND2), &[Tv(1220), Tv(1238)]),
    ((1247, false, AND2), &[Tv(992), Tv(1224)]),
];

static LEVEL_96: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1787, false, OR2), &[Tv(1778), Tv(1786)]),
    ((1792, false, OR2), &[Tv(1784), Tv(1791)]),
    ((2077, false, XNOR2), &[Tv(2068), Tv(2076)]),
    ((47, false, AND2), &[Tv(44), Tv(46)]),
    ((50, false, OR2), &[Tv(32), Tv(38)]),
    ((53, false, NAND2), &[Tv(37), Tv(44)]),
    ((738, false, NAND2), &[Tv(733), Tv(736)]),
    ((740, false, OR2), &[Tv(719), Tv(725)]),
    ((741, false, XNOR2), &[Tv(724), Tv(731)]),
    ((743, false, AND2), &[Tv(729), Tv(732)]),
    ((1240, false, NOR2), &[Tv(1227), Tv(1239)]),
    ((1241, false, AND2), &[Tv(1222), Tv(1235)]),
    ((1245, false, XNOR2), &[Tv(1226), Tv(1239)]),
    ((1248, false, XNOR2), &[Tv(783), Tv(1247)]),
];

static LEVEL_97: [((usize, bool, CellType), &[GateInput]); 11] = [
    ((1386, false, INV), &[Tv(1375)]),
    ((1793, false, NAND2), &[Tv(1787), Tv(1792)]),
    ((1797, false, NAND2), &[Tv(1787), Tv(1791)]),
    ((49, false, NAND2), &[Tv(2077), Tv(47)]),
    ((51, false, AND2), &[Tv(42), Tv(45)]),
    ((54, false, NAND2), &[Tv(50), Tv(53)]),
    ((742, false, NAND2), &[Tv(738), Tv(741)]),
    ((744, false, NAND2), &[Tv(740), Tv(743)]),
    ((1242, false, OR2), &[Tv(1237), Tv(1241)]),
    ((1250, false, NAND2), &[Tv(1245), Tv(1248)]),
    ((1252, false, XNOR2), &[Tv(1233), Tv(1240)]),
];

static LEVEL_98: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((1578, false, AND2), &[Tv(1386), Tv(1560)]),
    ((1781, false, INV), &[Tv(1780)]),
    ((1788, false, INV), &[Tv(1787)]),
    ((1794, false, NAND2), &[Tv(1783), Tv(1793)]),
    ((1798, false, OR2), &[Tv(1783), Tv(1797)]),
    ((2073, false, INV), &[Tv(2072)]),
    ((52, false, NAND2), &[Tv(50), Tv(51)]),
    ((55, false, NAND2), &[Tv(49), Tv(54)]),
    ((424, false, INV), &[Tv(423)]),
    ((737, false, INV), &[Tv(736)]),
    ((745, false, AND2), &[Tv(742), Tv(744)]),
    ((993, false, INV), &[Tv(992)]),
    ((1234, false, INV), &[Tv(1233)]),
    ((1244, false, NAND2), &[Tv(1238), Tv(1242)]),
    ((1253, false, NAND2), &[Tv(1250), Tv(1252)]),
];

static LEVEL_99: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((772, false, INV), &[Arg(0, 45)]),
    ((1579, false, XNOR2), &[Tv(475), Tv(1578)]),
    ((1789, false, NAND2), &[Tv(1784), Tv(1788)]),
    ((1795, false, NAND2), &[Tv(1781), Tv(1794)]),
    ((1799, false, AND2), &[Tv(1794), Tv(1798)]),
    ((2071, false, INV), &[Tv(2070)]),
    ((2078, false, INV), &[Tv(2077)]),
    ((56, false, AND2), &[Tv(52), Tv(55)]),
    ((63, false, AND2), &[Tv(2012), Tv(2073)]),
    ((422, false, INV), &[Tv(421)]),
    ((425, false, AND2), &[Tv(357), Tv(424)]),
    ((734, false, INV), &[Tv(733)]),
    ((746, false, OR2), &[Tv(737), Tv(745)]),
    ((1249, false, INV), &[Tv(1248)]),
    ((1251, false, XNOR2), &[Tv(1234), Tv(1240)]),
    ((1255, false, AND2), &[Tv(1244), Tv(1253)]),
    ((1267, false, NOR2), &[Tv(993), Tv(1223)]),
];

static LEVEL_100: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1796, false, NAND2), &[Tv(1789), Tv(1795)]),
    ((1801, false, NAND2), &[Tv(1579), Tv(1799)]),
    ((1803, false, AND2), &[Tv(1792), Tv(1797)]),
    ((57, false, NOR2), &[Tv(2078), Tv(56)]),
    ((59, false, OR2), &[Tv(51), Tv(54)]),
    ((62, false, XNOR2), &[Tv(2077), Tv(56)]),
    ((64, false, XNOR2), &[Tv(2071), Tv(63)]),
    ((426, false, XNOR2), &[Tv(422), Tv(425)]),
    ((747, false, NAND2), &[Tv(734), Tv(746)]),
    ((748, false, OR2), &[Tv(738), Tv(744)]),
    ((751, false, OR2), &[Tv(741), Tv(743)]),
    ((752, false, XNOR2), &[Tv(736), Tv(745)]),
    ((1256, false, NOR2), &[Tv(1249), Tv(1255)]),
    ((1260, false, NAND2), &[Tv(1244), Tv(1251)]),
    ((1264, false, XNOR2), &[Tv(1248), Tv(1255)]),
    ((1268, false, XNOR2), &[Tv(772), Tv(1267)]),
];

static LEVEL_101: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1364, false, INV), &[Tv(1353)]),
    ((1802, false, NAND2), &[Tv(1796), Tv(1801)]),
    ((1804, false, NAND2), &[Tv(1789), Tv(1803)]),
    ((58, false, OR2), &[Tv(49), Tv(52)]),
    ((60, false, AND2), &[Tv(55), Tv(59)]),
    ((66, false, NAND2), &[Tv(62), Tv(64)]),
    ((68, false, XNOR2), &[Tv(47), Tv(57)]),
    ((749, false, NAND2), &[Tv(747), Tv(748)]),
    ((754, false, NAND2), &[Tv(426), Tv(752)]),
    ((756, false, AND2), &[Tv(742), Tv(751)]),
    ((1257, false, OR2), &[Tv(1244), Tv(1250)]),
    ((1259, false, XNOR2), &[Tv(1245), Tv(1256)]),
    ((1261, false, AND2), &[Tv(1253), Tv(1260)]),
    ((1270, false, NAND2), &[Tv(1264), Tv(1268)]),
];

static LEVEL_102: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((464, false, INV), &[Arg(0, 13)]),
    ((1575, false, AND2), &[Tv(1364), Tv(1560)]),
    ((1580, false, INV), &[Tv(1579)]),
    ((1805, false, AND2), &[Tv(1802), Tv(1804)]),
    ((2063, false, INV), &[Tv(2062)]),
    ((48, false, INV), &[Tv(47)]),
    ((61, false, NAND2), &[Tv(58), Tv(60)]),
    ((69, false, NAND2), &[Tv(66), Tv(68)]),
    ((416, false, INV), &[Tv(415)]),
    ((755, false, NAND2), &[Tv(749), Tv(754)]),
    ((757, false, NAND2), &[Tv(748), Tv(756)]),
    ((1262, false, NAND2), &[Tv(1257), Tv(1261)]),
    ((1271, false, NAND2), &[Tv(1259), Tv(1270)]),
];

static LEVEL_103: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1576, false, XNOR2), &[Tv(464), Tv(1575)]),
    ((1800, false, AND2), &[Tv(1579), Tv(1799)]),
    ((1806, false, NOR2), &[Tv(1580), Tv(1805)]),
    ((1807, false, XNOR2), &[Tv(1579), Tv(1805)]),
    ((2057, false, INV), &[Tv(2056)]),
    ((2064, false, AND2), &[Tv(2012), Tv(2063)]),
    ((65, false, INV), &[Tv(64)]),
    ((67, false, XNOR2), &[Tv(48), Tv(57)]),
    ((70, false, AND2), &[Tv(61), Tv(69)]),
    ((405, false, INV), &[Tv(404)]),
    ((417, false, AND2), &[Tv(357), Tv(416)]),
    ((427, false, INV), &[Tv(426)]),
    ((758, false, AND2), &[Tv(755), Tv(757)]),
    ((963, false, INV), &[Tv(962)]),
    ((1269, false, INV), &[Tv(1268)]),
    ((1272, false, AND2), &[Tv(1262), Tv(1271)]),
];

static LEVEL_104: [((usize, bool, CellType), &[GateInput]); 18] = [
    ((761, false, INV), &[Arg(0, 44)]),
    ((1810, false, NAND2), &[Tv(1576), Tv(1807)]),
    ((1811, false, XNOR2), &[Tv(1799), Tv(1806)]),
    ((1813, false, NAND2), &[Tv(1796), Tv(1800)]),
    ((1814, false, OR2), &[Tv(1800), Tv(1804)]),
    ((2065, false, XNOR2), &[Tv(2057), Tv(2064)]),
    ((71, false, NOR2), &[Tv(65), Tv(70)]),
    ((74, false, NAND2), &[Tv(61), Tv(67)]),
    ((78, false, XNOR2), &[Tv(64), Tv(70)]),
    ((418, false, XNOR2), &[Tv(405), Tv(417)]),
    ((753, false, INV), &[Tv(752)]),
    ((759, false, NOR2), &[Tv(427), Tv(758)]),
    ((760, false, XNOR2), &[Tv(426), Tv(758)]),
    ((764, false, AND2), &[Tv(754), Tv(756)]),
    ((1246, false, INV), &[Tv(1245)]),
    ((1266, false, INV), &[Tv(1264)]),
    ((1273, false, NOR2), &[Tv(1269), Tv(1272)]),
    ((1281, false, AND2), &[Tv(963), Tv(992)]),
];

static LEVEL_105: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1812, false, NAND2), &[Tv(1810), Tv(1811)]),
    ((1815, false, AND2), &[Tv(1813), Tv(1814)]),
    ((72, false, OR2), &[Tv(61), Tv(66)]),
    ((73, false, XNOR2), &[Tv(62), Tv(71)]),
    ((75, false, AND2), &[Tv(69), Tv(74)]),
    ((79, false, NAND2), &[Tv(2065), Tv(78)]),
    ((763, false, AND2), &[Tv(418), Tv(760)]),
    ((765, false, OR2), &[Tv(749), Tv(764)]),
    ((769, false, XNOR2), &[Tv(753), Tv(759)]),
    ((1258, false, XNOR2), &[Tv(1246), Tv(1256)]),
    ((1274, false, OR2), &[Tv(1262), Tv(1270)]),
    ((1278, false, XNOR2), &[Tv(1266), Tv(1273)]),
    ((1282, false, XNOR2), &[Tv(761), Tv(1281)]),
    ((1284, false, XNOR2), &[Tv(1268), Tv(1272)]),
];

static LEVEL_106: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1342, false, INV), &[Tv(1331)]),
    ((1577, false, INV), &[Tv(1576)]),
    ((1817, false, AND2), &[Tv(1812), Tv(1815)]),
    ((2059, false, INV), &[Tv(2058)]),
    ((76, false, NAND2), &[Tv(72), Tv(75)]),
    ((80, false, NAND2), &[Tv(73), Tv(79)]),
    ((766, false, NAND2), &[Tv(755), Tv(765)]),
    ((770, false, OR2), &[Tv(763), Tv(769)]),
    ((1263, false, NAND2), &[Tv(1258), Tv(1262)]),
    ((1275, false, AND2), &[Tv(1271), Tv(1274)]),
    ((1279, false, INV), &[Tv(1278)]),
    ((1286, false, NAND2), &[Tv(1282), Tv(1284)]),
];

static LEVEL_107: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((453, false, INV), &[Arg(0, 12)]),
    ((1808, false, INV), &[Tv(1807)]),
    ((1809, false, AND2), &[Tv(1576), Tv(1807)]),
    ((1816, false, INV), &[Tv(1815)]),
    ((1818, false, OR2), &[Tv(1577), Tv(1817)]),
    ((1821, false, AND2), &[Tv(1342), Tv(1560)]),
    ((2061, false, INV), &[Tv(2060)]),
    ((2066, false, INV), &[Tv(2065)]),
    ((81, false, AND2), &[Tv(76), Tv(80)]),
    ((85, false, AND2), &[Tv(2012), Tv(2059)]),
    ((412, false, INV), &[Tv(411)]),
    ((771, false, NAND2), &[Tv(766), Tv(770)]),
    ((776, false, NAND2), &[Tv(766), Tv(769)]),
    ((1277, false, NAND2), &[Tv(1263), Tv(1275)]),
    ((1288, false, NAND2), &[Tv(1279), Tv(1286)]),
];

static LEVEL_108: [((usize, bool, CellType), &[GateInput]); 18] = [
    ((1819, false, XNOR2), &[Tv(1576), Tv(1817)]),
    ((1822, false, XNOR2), &[Tv(453), Tv(1821)]),
    ((1824, false, NAND2), &[Tv(1808), Tv(1818)]),
    ((1825, false, NAND2), &[Tv(1809), Tv(1816)]),
    ((1829, false, NAND2), &[Tv(1809), Tv(1811)]),
    ((77, false, OR2), &[Tv(73), Tv(75)]),
    ((82, false, NOR2), &[Tv(2066), Tv(81)]),
    ((83, false, XNOR2), &[Tv(2065), Tv(81)]),
    ((86, false, XNOR2), &[Tv(2061), Tv(85)]),
    ((414, false, INV), &[Tv(413)]),
    ((762, false, INV), &[Tv(760)]),
    ((767, false, INV), &[Tv(766)]),
    ((773, false, NAND2), &[Tv(418), Tv(771)]),
    ((780, false, OR2), &[Tv(418), Tv(776)]),
    ((784, false, AND2), &[Tv(357), Tv(412)]),
    ((961, false, INV), &[Tv(960)]),
    ((1280, false, NAND2), &[Tv(1277), Tv(1278)]),
    ((1289, false, NAND2), &[Tv(1277), Tv(1288)]),
];

static LEVEL_109: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((750, false, INV), &[Arg(0, 43)]),
    ((1826, false, NAND2), &[Tv(1824), Tv(1825)]),
    ((1827, false, NAND2), &[Tv(1819), Tv(1822)]),
    ((1830, false, NAND2), &[Tv(1815), Tv(1829)]),
    ((89, false, NAND2), &[Tv(83), Tv(86)]),
    ((90, false, OR2), &[Tv(76), Tv(79)]),
    ((91, false, XNOR2), &[Tv(78), Tv(82)]),
    ((93, false, AND2), &[Tv(77), Tv(80)]),
    ((768, false, NAND2), &[Tv(763), Tv(767)]),
    ((774, false, NAND2), &[Tv(762), Tv(773)]),
    ((781, false, AND2), &[Tv(773), Tv(780)]),
    ((785, false, XNOR2), &[Tv(414), Tv(784)]),
    ((1283, false, OR2), &[Tv(1280), Tv(1282)]),
    ((1285, false, INV), &[Tv(1284)]),
    ((1290, false, NAND2), &[Tv(1282), Tv(1289)]),
    ((1293, false, AND2), &[Tv(961), Tv(992)]),
];

static LEVEL_110: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1320, false, INV), &[Tv(1309)]),
    ((1828, false, NAND2), &[Tv(1826), Tv(1827)]),
    ((1831, false, NAND2), &[Tv(1825), Tv(1830)]),
    ((92, false, NAND2), &[Tv(89), Tv(91)]),
    ((94, false, NAND2), &[Tv(90), Tv(93)]),
    ((775, false, AND2), &[Tv(768), Tv(774)]),
    ((777, false, AND2), &[Tv(770), Tv(776)]),
    ((786, false, AND2), &[Tv(781), Tv(785)]),
    ((1291, false, AND2), &[Tv(1283), Tv(1290)]),
    ((1294, false, XNOR2), &[Tv(750), Tv(1293)]),
    ((1296, false, OR2), &[Tv(1277), Tv(1286)]),
    ((1302, false, NAND2), &[Tv(1285), Tv(1290)]),
];

static LEVEL_111: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((442, false, INV), &[Arg(0, 11)]),
    ((1573, false, AND2), &[Tv(1320), Tv(1560)]),
    ((1823, false, INV), &[Tv(1822)]),
    ((1833, false, AND2), &[Tv(1828), Tv(1831)]),
    ((2051, false, INV), &[Tv(2050)]),
    ((87, false, INV), &[Tv(86)]),
    ((95, false, AND2), &[Tv(92), Tv(94)]),
    ((778, false, INV), &[Tv(777)]),
    ((787, false, OR2), &[Tv(775), Tv(786)]),
    ((789, false, NAND2), &[Tv(768), Tv(777)]),
    ((1295, false, AND2), &[Tv(1291), Tv(1294)]),
    ((1297, false, AND2), &[Tv(1280), Tv(1296)]),
    ((1303, false, AND2), &[Tv(1296), Tv(1302)]),
];

static LEVEL_112: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((1574, false, XNOR2), &[Tv(442), Tv(1573)]),
    ((1820, false, INV), &[Tv(1819)]),
    ((1832, false, INV), &[Tv(1831)]),
    ((1834, false, NOR2), &[Tv(1823), Tv(1833)]),
    ((1836, false, XNOR2), &[Tv(1822), Tv(1833)]),
    ((2045, false, INV), &[Tv(2044)]),
    ((2052, false, AND2), &[Tv(2012), Tv(2051)]),
    ((84, false, INV), &[Tv(83)]),
    ((88, false, AND2), &[Tv(83), Tv(86)]),
    ((96, false, OR2), &[Tv(87), Tv(95)]),
    ((410, false, INV), &[Tv(408)]),
    ((779, false, NAND2), &[Tv(775), Tv(778)]),
    ((795, false, NAND2), &[Tv(787), Tv(789)]),
    ((1299, false, NAND2), &[Tv(1288), Tv(1297)]),
    ((1304, false, OR2), &[Tv(1295), Tv(1303)]),
];

static LEVEL_113: [((usize, bool, CellType), &[GateInput]); 18] = [
    ((1835, false, XNOR2), &[Tv(1820), Tv(1834)]),
    ((1838, false, AND2), &[Tv(1574), Tv(1836)]),
    ((1840, false, NAND2), &[Tv(1826), Tv(1833)]),
    ((1841, false, NAND2), &[Tv(1827), Tv(1832)]),
    ((2053, false, XNOR2), &[Tv(2045), Tv(2052)]),
    ((97, false, NAND2), &[Tv(84), Tv(96)]),
    ((98, false, OR2), &[Tv(89), Tv(94)]),
    ((100, false, XNOR2), &[Tv(86), Tv(95)]),
    ((104, false, NAND2), &[Tv(88), Tv(91)]),
    ((407, false, INV), &[Tv(406)]),
    ((782, false, INV), &[Tv(781)]),
    ((788, false, AND2), &[Tv(768), Tv(777)]),
    ((796, false, NAND2), &[Tv(785), Tv(795)]),
    ((797, false, OR2), &[Tv(779), Tv(785)]),
    ((800, false, AND2), &[Tv(357), Tv(410)]),
    ((958, false, INV), &[Tv(957)]),
    ((1305, false, NAND2), &[Tv(1299), Tv(1304)]),
    ((1310, false, NAND2), &[Tv(1299), Tv(1303)]),
];

static LEVEL_114: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((739, false, INV), &[Arg(0, 42)]),
    ((1839, false, OR2), &[Tv(1835), Tv(1838)]),
    ((1842, false, AND2), &[Tv(1840), Tv(1841)]),
    ((99, false, NAND2), &[Tv(97), Tv(98)]),
    ((102, false, NAND2), &[Tv(2053), Tv(100)]),
    ((105, false, NAND2), &[Tv(94), Tv(104)]),
    ((790, false, NAND2), &[Tv(786), Tv(788)]),
    ((798, false, AND2), &[Tv(796), Tv(797)]),
    ((801, false, XNOR2), &[Tv(407), Tv(800)]),
    ((804, false, NAND2), &[Tv(782), Tv(796)]),
    ((1292, false, INV), &[Tv(1291)]),
    ((1300, false, INV), &[Tv(1299)]),
    ((1306, false, NAND2), &[Tv(1294), Tv(1305)]),
    ((1315, false, AND2), &[Tv(958), Tv(992)]),
    ((1318, false, OR2), &[Tv(1294), Tv(1310)]),
];

static LEVEL_115: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((1298, false, INV), &[Tv(1287)]),
    ((1844, false, NAND2), &[Tv(1839), Tv(1842)]),
    ((1846, false, NAND2), &[Tv(1835), Tv(1842)]),
    ((103, false, NAND2), &[Tv(99), Tv(102)]),
    ((106, false, INV), &[Tv(105)]),
    ((107, false, NAND2), &[Tv(98), Tv(105)]),
    ((791, false, AND2), &[Tv(787), Tv(790)]),
    ((802, false, AND2), &[Tv(798), Tv(801)]),
    ((806, false, AND2), &[Tv(790), Tv(804)]),
    ((1301, false, NAND2), &[Tv(1295), Tv(1300)]),
    ((1307, false, NAND2), &[Tv(1292), Tv(1306)]),
    ((1316, false, XNOR2), &[Tv(739), Tv(1315)]),
    ((1319, false, AND2), &[Tv(1306), Tv(1318)]),
];

static LEVEL_116: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((431, false, INV), &[Arg(0, 10)]),
    ((1837, false, INV), &[Tv(1836)]),
    ((1843, false, INV), &[Tv(1842)]),
    ((1845, false, NAND2), &[Tv(1574), Tv(1844)]),
    ((1847, false, OR2), &[Tv(1574), Tv(1846)]),
    ((1850, false, AND2), &[Tv(1298), Tv(1560)]),
    ((2049, false, INV), &[Tv(2048)]),
    ((108, false, NAND2), &[Tv(103), Tv(107)]),
    ((110, false, NAND2), &[Tv(97), Tv(106)]),
    ((792, false, NAND2), &[Tv(779), Tv(791)]),
    ((807, false, OR2), &[Tv(802), Tv(806)]),
    ((1308, false, AND2), &[Tv(1301), Tv(1307)]),
    ((1311, false, AND2), &[Tv(1304), Tv(1310)]),
    ((1322, false, AND2), &[Tv(1316), Tv(1319)]),
];

static LEVEL_117: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1848, false, AND2), &[Tv(1845), Tv(1847)]),
    ((1851, false, XNOR2), &[Tv(431), Tv(1850)]),
    ((1853, false, NAND2), &[Tv(1838), Tv(1843)]),
    ((1858, false, NAND2), &[Tv(1837), Tv(1845)]),
    ((2047, false, INV), &[Tv(2046)]),
    ((101, false, INV), &[Tv(100)]),
    ((109, false, NAND2), &[Tv(2053), Tv(108)]),
    ((111, false, OR2), &[Tv(2053), Tv(110)]),
    ((114, false, AND2), &[Tv(2012), Tv(2049)]),
    ((400, false, INV), &[Tv(399)]),
    ((808, false, NAND2), &[Tv(792), Tv(807)]),
    ((812, false, NAND2), &[Tv(792), Tv(806)]),
    ((1312, false, NAND2), &[Tv(1301), Tv(1311)]),
    ((1323, false, OR2), &[Tv(1308), Tv(1322)]),
];

static LEVEL_118: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((1852, false, AND2), &[Tv(1848), Tv(1851)]),
    ((1854, false, AND2), &[Tv(1839), Tv(1846)]),
    ((1859, false, AND2), &[Tv(1853), Tv(1858)]),
    ((112, false, AND2), &[Tv(109), Tv(111)]),
    ((115, false, XNOR2), &[Tv(2047), Tv(114)]),
    ((117, false, OR2), &[Tv(102), Tv(107)]),
    ((122, false, NAND2), &[Tv(101), Tv(109)]),
    ((393, false, INV), &[Tv(392)]),
    ((401, false, AND2), &[Tv(357), Tv(400)]),
    ((793, false, INV), &[Tv(792)]),
    ((799, false, INV), &[Tv(798)]),
    ((809, false, NAND2), &[Tv(801), Tv(808)]),
    ((813, false, OR2), &[Tv(801), Tv(812)]),
    ((956, false, INV), &[Tv(955)]),
    ((1313, false, INV), &[Tv(1312)]),
    ((1314, false, NAND2), &[Tv(1308), Tv(1312)]),
    ((1324, false, NAND2), &[Tv(1312), Tv(1323)]),
];

static LEVEL_119: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((728, false, INV), &[Arg(0, 41)]),
    ((1855, false, NAND2), &[Tv(1853), Tv(1854)]),
    ((1860, false, OR2), &[Tv(1852), Tv(1859)]),
    ((116, false, AND2), &[Tv(112), Tv(115)]),
    ((118, false, AND2), &[Tv(103), Tv(117)]),
    ((123, false, AND2), &[Tv(117), Tv(122)]),
    ((402, false, XNOR2), &[Tv(393), Tv(401)]),
    ((803, false, NAND2), &[Tv(793), Tv(802)]),
    ((810, false, NAND2), &[Tv(799), Tv(809)]),
    ((814, false, AND2), &[Tv(809), Tv(813)]),
    ((1317, false, OR2), &[Tv(1314), Tv(1316)]),
    ((1321, false, INV), &[Tv(1319)]),
    ((1325, false, NAND2), &[Tv(1316), Tv(1324)]),
    ((1328, false, AND2), &[Tv(956), Tv(992)]),
    ((1333, false, NAND2), &[Tv(1313), Tv(1322)]),
];

static LEVEL_120: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1276, false, INV), &[Tv(1265)]),
    ((1861, false, NAND2), &[Tv(1855), Tv(1860)]),
    ((1865, false, NAND2), &[Tv(1855), Tv(1859)]),
    ((119, false, NAND2), &[Tv(110), Tv(118)]),
    ((124, false, OR2), &[Tv(116), Tv(123)]),
    ((811, false, NAND2), &[Tv(803), Tv(810)]),
    ((815, false, NAND2), &[Tv(402), Tv(814)]),
    ((818, false, AND2), &[Tv(807), Tv(812)]),
    ((1326, false, AND2), &[Tv(1317), Tv(1325)]),
    ((1329, false, XNOR2), &[Tv(728), Tv(1328)]),
    ((1334, false, AND2), &[Tv(1323), Tv(1333)]),
    ((1338, false, NAND2), &[Tv(1321), Tv(1325)]),
];

static LEVEL_121: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((420, false, INV), &[Arg(0, 9)]),
    ((1849, false, INV), &[Tv(1848)]),
    ((1856, false, INV), &[Tv(1855)]),
    ((1862, false, NAND2), &[Tv(1851), Tv(1861)]),
    ((1869, false, AND2), &[Tv(1276), Tv(1560)]),
    ((1872, false, OR2), &[Tv(1851), Tv(1865)]),
    ((2040, false, INV), &[Tv(2039)]),
    ((125, false, NAND2), &[Tv(119), Tv(124)]),
    ((129, false, NAND2), &[Tv(119), Tv(123)]),
    ((397, false, INV), &[Tv(396)]),
    ((817, false, NAND2), &[Tv(811), Tv(815)]),
    ((819, false, NAND2), &[Tv(803), Tv(818)]),
    ((1330, false, AND2), &[Tv(1326), Tv(1329)]),
    ((1335, false, NAND2), &[Tv(1314), Tv(1334)]),
    ((1339, false, AND2), &[Tv(1333), Tv(1338)]),
];

static LEVEL_122: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((1857, false, NAND2), &[Tv(1852), Tv(1856)]),
    ((1863, false, NAND2), &[Tv(1849), Tv(1862)]),
    ((1866, false, AND2), &[Tv(1860), Tv(1865)]),
    ((1870, false, XNOR2), &[Tv(420), Tv(1869)]),
    ((1873, false, AND2), &[Tv(1862), Tv(1872)]),
    ((2034, false, INV), &[Tv(2033)]),
    ((2041, false, AND2), &[Tv(2012), Tv(2040)]),
    ((113, false, INV), &[Tv(112)]),
    ((120, false, INV), &[Tv(119)]),
    ((126, false, NAND2), &[Tv(115), Tv(125)]),
    ((133, false, OR2), &[Tv(115), Tv(129)]),
    ((395, false, INV), &[Tv(394)]),
    ((403, false, INV), &[Tv(402)]),
    ((820, false, AND2), &[Tv(817), Tv(819)]),
    ((825, false, AND2), &[Tv(357), Tv(397)]),
    ((1340, false, NAND2), &[Tv(1335), Tv(1339)]),
    ((1341, false, NAND2), &[Tv(1330), Tv(1335)]),
];

static LEVEL_123: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1864, false, AND2), &[Tv(1857), Tv(1863)]),
    ((1867, false, NAND2), &[Tv(1857), Tv(1866)]),
    ((1875, false, AND2), &[Tv(1870), Tv(1873)]),
    ((2042, false, XNOR2), &[Tv(2034), Tv(2041)]),
    ((121, false, NAND2), &[Tv(116), Tv(120)]),
    ((127, false, NAND2), &[Tv(113), Tv(126)]),
    ((130, false, AND2), &[Tv(124), Tv(129)]),
    ((134, false, AND2), &[Tv(126), Tv(133)]),
    ((821, false, NOR2), &[Tv(403), Tv(820)]),
    ((822, false, OR2), &[Tv(811), Tv(818)]),
    ((823, false, XNOR2), &[Tv(402), Tv(820)]),
    ((826, false, XNOR2), &[Tv(395), Tv(825)]),
    ((954, false, INV), &[Tv(953)]),
    ((1343, false, AND2), &[Tv(1340), Tv(1341)]),
];

static LEVEL_124: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((717, false, INV), &[Arg(0, 40)]),
    ((1254, false, INV), &[Tv(1243)]),
    ((1868, false, NAND2), &[Tv(1864), Tv(1867)]),
    ((1877, false, NAND2), &[Tv(1867), Tv(1875)]),
    ((128, false, AND2), &[Tv(121), Tv(127)]),
    ((131, false, NAND2), &[Tv(121), Tv(130)]),
    ((135, false, AND2), &[Tv(2042), Tv(134)]),
    ((829, false, NAND2), &[Tv(823), Tv(826)]),
    ((830, false, OR2), &[Tv(815), Tv(819)]),
    ((831, false, XNOR2), &[Tv(814), Tv(821)]),
    ((833, false, AND2), &[Tv(817), Tv(822)]),
    ((1327, false, INV), &[Tv(1326)]),
    ((1332, false, INV), &[Tv(1330)]),
    ((1336, false, INV), &[Tv(1335)]),
    ((1344, false, NAND2), &[Tv(1329), Tv(1343)]),
    ((1352, false, AND2), &[Tv(954), Tv(992)]),
    ((1356, false, OR2), &[Tv(1329), Tv(1340)]),
];

static LEVEL_125: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((409, false, INV), &[Arg(0, 8)]),
    ((1570, false, AND2), &[Tv(1254), Tv(1560)]),
    ((1871, false, INV), &[Tv(1870)]),
    ((1876, false, NAND2), &[Tv(1870), Tv(1873)]),
    ((1878, false, AND2), &[Tv(1868), Tv(1877)]),
    ((2038, false, INV), &[Tv(2037)]),
    ((132, false, NAND2), &[Tv(128), Tv(131)]),
    ((137, false, NAND2), &[Tv(131), Tv(135)]),
    ((832, false, NAND2), &[Tv(829), Tv(831)]),
    ((834, false, NAND2), &[Tv(830), Tv(833)]),
    ((1337, false, NAND2), &[Tv(1330), Tv(1336)]),
    ((1345, false, NAND2), &[Tv(1327), Tv(1344)]),
    ((1347, false, NAND2), &[Tv(1332), Tv(1339)]),
    ((1354, false, XNOR2), &[Tv(717), Tv(1352)]),
    ((1357, false, AND2), &[Tv(1344), Tv(1356)]),
];

static LEVEL_126: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1571, false, XNOR2), &[Tv(409), Tv(1570)]),
    ((1874, false, INV), &[Tv(1873)]),
    ((1879, false, AND2), &[Tv(1870), Tv(1878)]),
    ((1880, false, XNOR2), &[Tv(1871), Tv(1878)]),
    ((1885, false, NAND2), &[Tv(1864), Tv(1876)]),
    ((2036, false, INV), &[Tv(2035)]),
    ((2043, false, INV), &[Tv(2042)]),
    ((136, false, NAND2), &[Tv(2042), Tv(134)]),
    ((138, false, AND2), &[Tv(132), Tv(137)]),
    ((142, false, AND2), &[Tv(2012), Tv(2038)]),
    ((388, false, INV), &[Tv(386)]),
    ((828, false, INV), &[Tv(826)]),
    ((835, false, AND2), &[Tv(832), Tv(834)]),
    ((1346, false, AND2), &[Tv(1337), Tv(1345)]),
    ((1348, false, NAND2), &[Tv(1341), Tv(1347)]),
    ((1359, false, AND2), &[Tv(1354), Tv(1357)]),
];

static LEVEL_127: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1882, false, AND2), &[Tv(1571), Tv(1880)]),
    ((1883, false, XNOR2), &[Tv(1874), Tv(1879)]),
    ((1886, false, NAND2), &[Tv(1877), Tv(1885)]),
    ((139, false, AND2), &[Tv(2042), Tv(138)]),
    ((140, false, XNOR2), &[Tv(2043), Tv(138)]),
    ((143, false, XNOR2), &[Tv(2036), Tv(142)]),
    ((148, false, NAND2), &[Tv(128), Tv(136)]),
    ((385, false, INV), &[Tv(384)]),
    ((389, false, AND2), &[Tv(357), Tv(388)]),
    ((824, false, INV), &[Tv(823)]),
    ((836, false, OR2), &[Tv(828), Tv(835)]),
    ((841, false, OR2), &[Tv(831), Tv(833)]),
    ((1349, false, NAND2), &[Tv(1340), Tv(1348)]),
    ((1360, false, OR2), &[Tv(1346), Tv(1359)]),
];

static LEVEL_128: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((1884, false, OR2), &[Tv(1882), Tv(1883)]),
    ((1887, false, NAND2), &[Tv(1868), Tv(1886)]),
    ((145, false, NAND2), &[Tv(140), Tv(143)]),
    ((146, false, XNOR2), &[Tv(134), Tv(139)]),
    ((149, false, NAND2), &[Tv(137), Tv(148)]),
    ((390, false, XNOR2), &[Tv(385), Tv(389)]),
    ((837, false, NAND2), &[Tv(824), Tv(836)]),
    ((839, false, OR2), &[Tv(829), Tv(834)]),
    ((842, false, XNOR2), &[Tv(826), Tv(835)]),
    ((845, false, NAND2), &[Tv(832), Tv(841)]),
    ((952, false, INV), &[Tv(951)]),
    ((1351, false, NAND2), &[Tv(1346), Tv(1349)]),
    ((1361, false, NAND2), &[Tv(1349), Tv(1360)]),
];

static LEVEL_129: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((706, false, INV), &[Arg(0, 39)]),
    ((1232, false, INV), &[Tv(1221)]),
    ((1572, false, INV), &[Tv(1571)]),
    ((1889, false, NAND2), &[Tv(1884), Tv(1887)]),
    ((1891, false, AND2), &[Tv(1883), Tv(1887)]),
    ((147, false, NAND2), &[Tv(145), Tv(146)]),
    ((150, false, NAND2), &[Tv(132), Tv(149)]),
    ((840, false, NAND2), &[Tv(837), Tv(839)]),
    ((843, false, NAND2), &[Tv(390), Tv(842)]),
    ((846, false, INV), &[Tv(845)]),
    ((1350, false, INV), &[Tv(1349)]),
    ((1355, false, OR2), &[Tv(1351), Tv(1354)]),
    ((1358, false, INV), &[Tv(1357)]),
    ((1362, false, NAND2), &[Tv(1354), Tv(1361)]),
    ((1366, false, AND2), &[Tv(952), Tv(992)]),
];

static LEVEL_130: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((398, false, INV), &[Arg(0, 7)]),
    ((1881, false, INV), &[Tv(1880)]),
    ((1888, false, INV), &[Tv(1887)]),
    ((1890, false, NAND2), &[Tv(1571), Tv(1889)]),
    ((1892, false, NAND2), &[Tv(1572), Tv(1891)]),
    ((1895, false, AND2), &[Tv(1232), Tv(1560)]),
    ((2030, false, INV), &[Tv(2029)]),
    ((144, false, INV), &[Tv(143)]),
    ((152, false, AND2), &[Tv(147), Tv(150)]),
    ((391, false, INV), &[Tv(390)]),
    ((844, false, NAND2), &[Tv(840), Tv(843)]),
    ((847, false, NAND2), &[Tv(839), Tv(846)]),
    ((851, false, AND2), &[Tv(837), Tv(845)]),
    ((1363, false, AND2), &[Tv(1355), Tv(1362)]),
    ((1367, false, XNOR2), &[Tv(706), Tv(1366)]),
    ((1369, false, NAND2), &[Tv(1358), Tv(1362)]),
    ((1370, false, NAND2), &[Tv(1350), Tv(1359)]),
];

static LEVEL_131: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((1893, false, AND2), &[Tv(1890), Tv(1892)]),
    ((1896, false, XNOR2), &[Tv(398), Tv(1895)]),
    ((1898, false, NAND2), &[Tv(1882), Tv(1888)]),
    ((1903, false, NAND2), &[Tv(1881), Tv(1890)]),
    ((2024, false, INV), &[Tv(2023)]),
    ((2031, false, AND2), &[Tv(2012), Tv(2030)]),
    ((141, false, INV), &[Tv(140)]),
    ((151, false, INV), &[Tv(150)]),
    ((153, false, OR2), &[Tv(144), Tv(152)]),
    ((380, false, INV), &[Tv(379)]),
    ((848, false, NAND2), &[Tv(844), Tv(847)]),
    ((852, false, NAND2), &[Tv(391), Tv(851)]),
    ((1371, false, AND2), &[Tv(1369), Tv(1370)]),
    ((1372, false, AND2), &[Tv(1363), Tv(1367)]),
    ((1374, false, AND2), &[Tv(1351), Tv(1360)]),
];

static LEVEL_132: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((1897, false, AND2), &[Tv(1893), Tv(1896)]),
    ((1899, false, NAND2), &[Tv(1884), Tv(1898)]),
    ((1904, false, AND2), &[Tv(1898), Tv(1903)]),
    ((2032, false, XNOR2), &[Tv(2024), Tv(2031)]),
    ((154, false, NAND2), &[Tv(141), Tv(153)]),
    ((155, false, OR2), &[Tv(145), Tv(150)]),
    ((157, false, OR2), &[Tv(146), Tv(151)]),
    ((158, false, XNOR2), &[Tv(143), Tv(152)]),
    ((373, false, INV), &[Tv(372)]),
    ((381, false, AND2), &[Tv(357), Tv(380)]),
    ((850, false, AND2), &[Tv(390), Tv(848)]),
    ((853, false, INV), &[Tv(852)]),
    ((1373, false, OR2), &[Tv(1371), Tv(1372)]),
    ((1376, false, NAND2), &[Tv(1351), Tv(1360)]),
    ((1378, false, NAND2), &[Tv(1370), Tv(1374)]),
];

static LEVEL_133: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1900, false, OR2), &[Tv(1891), Tv(1899)]),
    ((1905, false, OR2), &[Tv(1897), Tv(1904)]),
    ((156, false, AND2), &[Tv(154), Tv(155)]),
    ((160, false, AND2), &[Tv(2032), Tv(158)]),
    ((162, false, AND2), &[Tv(147), Tv(157)]),
    ((382, false, XNOR2), &[Tv(373), Tv(381)]),
    ((854, false, NOR2), &[Tv(850), Tv(853)]),
    ((857, false, OR2), &[Tv(843), Tv(847)]),
    ((950, false, INV), &[Tv(949)]),
    ((1368, false, INV), &[Tv(1367)]),
    ((1379, false, NAND2), &[Tv(1373), Tv(1378)]),
    ((1384, false, AND2), &[Tv(1371), Tv(1376)]),
];

static LEVEL_134: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((695, false, INV), &[Arg(0, 38)]),
    ((1210, false, INV), &[Tv(1199)]),
    ((1906, false, NAND2), &[Tv(1900), Tv(1905)]),
    ((1910, false, NAND2), &[Tv(1900), Tv(1904)]),
    ((161, false, OR2), &[Tv(156), Tv(160)]),
    ((163, false, NAND2), &[Tv(155), Tv(162)]),
    ((856, false, NAND2), &[Tv(382), Tv(854)]),
    ((858, false, XNOR2), &[Tv(842), Tv(850)]),
    ((861, false, NAND2), &[Tv(844), Tv(857)]),
    ((1000, false, AND2), &[Tv(950), Tv(992)]),
    ((1365, false, INV), &[Tv(1363)]),
    ((1377, false, AND2), &[Tv(1370), Tv(1374)]),
    ((1380, false, NAND2), &[Tv(1367), Tv(1379)]),
    ((1385, false, NAND2), &[Tv(1368), Tv(1384)]),
];

static LEVEL_135: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((387, false, INV), &[Arg(0, 6)]),
    ((1567, false, AND2), &[Tv(1210), Tv(1560)]),
    ((1894, false, INV), &[Tv(1893)]),
    ((1901, false, INV), &[Tv(1900)]),
    ((1907, false, NAND2), &[Tv(1896), Tv(1906)]),
    ((1911, false, OR2), &[Tv(1896), Tv(1910)]),
    ((2028, false, INV), &[Tv(2027)]),
    ((165, false, NAND2), &[Tv(161), Tv(163)]),
    ((167, false, NAND2), &[Tv(156), Tv(163)]),
    ((859, false, NAND2), &[Tv(856), Tv(858)]),
    ((862, false, OR2), &[Tv(851), Tv(861)]),
    ((1001, false, XNOR2), &[Tv(695), Tv(1000)]),
    ((1381, false, NAND2), &[Tv(1365), Tv(1380)]),
    ((1382, false, NAND2), &[Tv(1372), Tv(1377)]),
    ((1387, false, AND2), &[Tv(1380), Tv(1385)]),
];

static LEVEL_136: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1568, false, XNOR2), &[Tv(387), Tv(1567)]),
    ((1902, false, NAND2), &[Tv(1897), Tv(1901)]),
    ((1908, false, NAND2), &[Tv(1894), Tv(1907)]),
    ((1912, false, AND2), &[Tv(1907), Tv(1911)]),
    ((2026, false, INV), &[Tv(2025)]),
    ((159, false, INV), &[Tv(158)]),
    ((164, false, INV), &[Tv(163)]),
    ((166, false, NAND2), &[Tv(2032), Tv(165)]),
    ((168, false, OR2), &[Tv(2032), Tv(167)]),
    ((171, false, AND2), &[Tv(2012), Tv(2028)]),
    ((375, false, INV), &[Tv(374)]),
    ((383, false, INV), &[Tv(382)]),
    ((864, false, AND2), &[Tv(859), Tv(862)]),
    ((1383, false, AND2), &[Tv(1381), Tv(1382)]),
    ((1389, false, AND2), &[Tv(1001), Tv(1387)]),
    ((1391, false, NAND2), &[Tv(1373), Tv(1382)]),
];

static LEVEL_137: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1909, false, NAND2), &[Tv(1902), Tv(1908)]),
    ((1913, false, NAND2), &[Tv(1568), Tv(1912)]),
    ((1915, false, AND2), &[Tv(1905), Tv(1910)]),
    ((169, false, AND2), &[Tv(166), Tv(168)]),
    ((172, false, XNOR2), &[Tv(2026), Tv(171)]),
    ((175, false, NAND2), &[Tv(159), Tv(166)]),
    ((176, false, NAND2), &[Tv(160), Tv(164)]),
    ((378, false, INV), &[Tv(377)]),
    ((855, false, INV), &[Tv(854)]),
    ((863, false, INV), &[Tv(862)]),
    ((865, false, OR2), &[Tv(383), Tv(864)]),
    ((869, false, AND2), &[Tv(357), Tv(375)]),
    ((1390, false, OR2), &[Tv(1383), Tv(1389)]),
    ((1392, false, OR2), &[Tv(1384), Tv(1391)]),
];

static LEVEL_138: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1188, false, INV), &[Tv(1177)]),
    ((1914, false, NAND2), &[Tv(1909), Tv(1913)]),
    ((1916, false, NAND2), &[Tv(1902), Tv(1915)]),
    ((174, false, NAND2), &[Tv(169), Tv(172)]),
    ((177, false, NAND2), &[Tv(175), Tv(176)]),
    ((179, false, AND2), &[Tv(161), Tv(176)]),
    ((866, false, OR2), &[Tv(858), Tv(863)]),
    ((867, false, XNOR2), &[Tv(382), Tv(864)]),
    ((870, false, XNOR2), &[Tv(378), Tv(869)]),
    ((873, false, NAND2), &[Tv(855), Tv(865)]),
    ((874, false, OR2), &[Tv(856), Tv(862)]),
    ((947, false, INV), &[Tv(946)]),
    ((1394, false, NAND2), &[Tv(1390), Tv(1392)]),
    ((1396, false, NAND2), &[Tv(1383), Tv(1392)]),
];

static LEVEL_139: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((1569, false, INV), &[Tv(1568)]),
    ((1917, false, AND2), &[Tv(1914), Tv(1916)]),
    ((1922, false, AND2), &[Tv(1188), Tv(1560)]),
    ((178, false, NAND2), &[Tv(174), Tv(177)]),
    ((180, false, NAND2), &[Tv(167), Tv(179)]),
    ((875, false, AND2), &[Tv(873), Tv(874)]),
    ((876, false, AND2), &[Tv(867), Tv(870)]),
    ((878, false, AND2), &[Tv(859), Tv(866)]),
    ((1388, false, INV), &[Tv(1387)]),
    ((1393, false, INV), &[Tv(1392)]),
    ((1395, false, NAND2), &[Tv(1001), Tv(1394)]),
    ((1398, false, OR2), &[Tv(1001), Tv(1396)]),
    ((1401, false, AND2), &[Tv(947), Tv(992)]),
];

static LEVEL_140: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1918, false, NOR2), &[Tv(1569), Tv(1917)]),
    ((1919, false, OR2), &[Tv(1909), Tv(1915)]),
    ((1920, false, XNOR2), &[Tv(1568), Tv(1917)]),
    ((1923, false, XNOR2), &[Tv(376), Tv(1922)]),
    ((2019, false, INV), &[Tv(2018)]),
    ((173, false, INV), &[Tv(172)]),
    ((182, false, AND2), &[Tv(178), Tv(180)]),
    ((877, false, OR2), &[Tv(875), Tv(876)]),
    ((879, false, NAND2), &[Tv(859), Tv(866)]),
    ((881, false, NAND2), &[Tv(874), Tv(878)]),
    ((1399, false, AND2), &[Tv(1395), Tv(1398)]),
    ((1402, false, XNOR2), &[Tv(684), Tv(1401)]),
    ((1405, false, NAND2), &[Tv(1389), Tv(1393)]),
    ((1411, false, NAND2), &[Tv(1388), Tv(1395)]),
];

static LEVEL_141: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1925, false, NAND2), &[Tv(1920), Tv(1923)]),
    ((1926, false, OR2), &[Tv(1913), Tv(1916)]),
    ((1927, false, XNOR2), &[Tv(1912), Tv(1918)]),
    ((1929, false, AND2), &[Tv(1914), Tv(1919)]),
    ((2017, false, INV), &[Tv(2016)]),
    ((2020, false, AND2), &[Tv(2012), Tv(2019)]),
    ((170, false, INV), &[Tv(169)]),
    ((181, false, INV), &[Tv(180)]),
    ((183, false, OR2), &[Tv(173), Tv(182)]),
    ((369, false, INV), &[Tv(368)]),
    ((872, false, INV), &[Tv(870)]),
    ((883, false, NAND2), &[Tv(877), Tv(881)]),
    ((888, false, AND2), &[Tv(875), Tv(879)]),
    ((1404, false, NAND2), &[Tv(1399), Tv(1402)]),
    ((1406, false, AND2), &[Tv(1390), Tv(1405)]),
    ((1412, false, NAND2), &[Tv(1405), Tv(1411)]),
];

static LEVEL_142: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1166, false, INV), &[Tv(1155)]),
    ((1928, false, NAND2), &[Tv(1925), Tv(1927)]),
    ((1930, false, NAND2), &[Tv(1926), Tv(1929)]),
    ((2021, false, XNOR2), &[Tv(2017), Tv(2020)]),
    ((184, false, NAND2), &[Tv(170), Tv(183)]),
    ((185, false, OR2), &[Tv(174), Tv(180)]),
    ((187, false, OR2), &[Tv(177), Tv(181)]),
    ((188, false, XNOR2), &[Tv(172), Tv(182)]),
    ((364, false, INV), &[Tv(363)]),
    ((370, false, AND2), &[Tv(357), Tv(369)]),
    ((868, false, INV), &[Tv(867)]),
    ((880, false, AND2), &[Tv(874), Tv(878)]),
    ((884, false, NAND2), &[Tv(870), Tv(883)]),
    ((889, false, NAND2), &[Tv(872), Tv(888)]),
    ((1407, false, NAND2), &[Tv(1396), Tv(1406)]),
    ((1413, false, NAND2), &[Tv(1404), Tv(1412)]),
];

static LEVEL_143: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((1564, false, AND2), &[Tv(1166), Tv(1560)]),
    ((1924, false, INV), &[Tv(1923)]),
    ((1931, false, AND2), &[Tv(1928), Tv(1930)]),
    ((186, false, NAND2), &[Tv(184), Tv(185)]),
    ((190, false, NAND2), &[Tv(2021), Tv(188)]),
    ((192, false, AND2), &[Tv(178), Tv(187)]),
    ((371, false, XNOR2), &[Tv(364), Tv(370)]),
    ((885, false, NAND2), &[Tv(868), Tv(884)]),
    ((886, false, NAND2), &[Tv(876), Tv(880)]),
    ((890, false, AND2), &[Tv(884), Tv(889)]),
    ((945, false, INV), &[Tv(944)]),
    ((1403, false, INV), &[Tv(1402)]),
    ((1414, false, AND2), &[Tv(1407), Tv(1413)]),
];

static LEVEL_144: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1565, false, XNOR2), &[Tv(301), Tv(1564)]),
    ((1921, false, INV), &[Tv(1920)]),
    ((1932, false, OR2), &[Tv(1924), Tv(1931)]),
    ((1987, false, INV), &[Tv(1986)]),
    ((191, false, NAND2), &[Tv(186), Tv(190)]),
    ((193, false, NAND2), &[Tv(185), Tv(192)]),
    ((887, false, AND2), &[Tv(885), Tv(886)]),
    ((892, false, AND2), &[Tv(371), Tv(890)]),
    ((895, false, NAND2), &[Tv(877), Tv(886)]),
    ((997, false, AND2), &[Tv(945), Tv(992)]),
    ((1400, false, INV), &[Tv(1399)]),
    ((1409, false, INV), &[Tv(1407)]),
    ((1410, false, OR2), &[Tv(1404), Tv(1407)]),
    ((1415, false, OR2), &[Tv(1403), Tv(1414)]),
];

static LEVEL_145: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1566, false, INV), &[Tv(1565)]),
    ((1933, false, NAND2), &[Tv(1921), Tv(1932)]),
    ((1934, false, OR2), &[Tv(1925), Tv(1930)]),
    ((1936, false, OR2), &[Tv(1927), Tv(1929)]),
    ((1937, false, XNOR2), &[Tv(1924), Tv(1931)]),
    ((1983, false, INV), &[Tv(1982)]),
    ((2014, false, AND2), &[Tv(1987), Tv(2012)]),
    ((2022, false, INV), &[Tv(2021)]),
    ((194, false, AND2), &[Tv(191), Tv(193)]),
    ((894, false, OR2), &[Tv(887), Tv(892)]),
    ((896, false, OR2), &[Tv(888), Tv(895)]),
    ((998, false, XNOR2), &[Tv(321), Tv(997)]),
    ((1416, false, NAND2), &[Tv(1400), Tv(1415)]),
    ((1418, false, OR2), &[Tv(1409), Tv(1412)]),
    ((1420, false, XNOR2), &[Tv(1402), Tv(1414)]),
    ((1423, false, AND2), &[Tv(1410), Tv(1413)]),
];

static LEVEL_146: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((673, false, INV), &[Arg(0, 35)]),
    ((1935, false, NAND2), &[Tv(1933), Tv(1934)]),
    ((1938, false, OR2), &[Tv(1566), Tv(1937)]),
    ((1940, false, AND2), &[Tv(1928), Tv(1936)]),
    ((2015, false, XNOR2), &[Tv(1983), Tv(2014)]),
    ((189, false, INV), &[Tv(188)]),
    ((195, false, NOR2), &[Tv(2022), Tv(194)]),
    ((196, false, OR2), &[Tv(186), Tv(192)]),
    ((197, false, XNOR2), &[Tv(2021), Tv(194)]),
    ((367, false, INV), &[Tv(366)]),
    ((898, false, NAND2), &[Tv(894), Tv(896)]),
    ((900, false, NAND2), &[Tv(887), Tv(896)]),
    ((1417, false, AND2), &[Tv(1410), Tv(1416)]),
    ((1421, false, AND2), &[Tv(998), Tv(1420)]),
    ((1424, false, NAND2), &[Tv(1418), Tv(1423)]),
];

static LEVEL_147: [((usize, bool, CellType), &[GateInput]); 15] = [
    ((1939, false, NAND2), &[Tv(1935), Tv(1938)]),
    ((1941, false, NAND2), &[Tv(1934), Tv(1940)]),
    ((199, false, AND2), &[Tv(2015), Tv(197)]),
    ((200, false, OR2), &[Tv(190), Tv(193)]),
    ((201, false, XNOR2), &[Tv(189), Tv(195)]),
    ((203, false, AND2), &[Tv(191), Tv(196)]),
    ((891, false, INV), &[Tv(890)]),
    ((897, false, INV), &[Tv(896)]),
    ((899, false, NAND2), &[Tv(371), Tv(898)]),
    ((901, false, OR2), &[Tv(371), Tv(900)]),
    ((905, false, XNOR2), &[Tv(673), Tv(324)]),
    ((906, false, AND2), &[Tv(357), Tv(367)]),
    ((999, false, INV), &[Tv(998)]),
    ((1422, false, OR2), &[Tv(1417), Tv(1421)]),
    ((1427, false, AND2), &[Tv(1417), Tv(1424)]),
];

static LEVEL_148: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((1144, false, INV), &[Tv(1133)]),
    ((1942, false, NAND2), &[Tv(1939), Tv(1941)]),
    ((1944, false, NOR2), &[Tv(1935), Tv(1940)]),
    ((202, false, OR2), &[Tv(199), Tv(201)]),
    ((204, false, NAND2), &[Tv(200), Tv(203)]),
    ((902, false, AND2), &[Tv(899), Tv(901)]),
    ((907, false, XNOR2), &[Tv(905), Tv(906)]),
    ((909, false, NAND2), &[Tv(891), Tv(899)]),
    ((910, false, NAND2), &[Tv(892), Tv(897)]),
    ((943, false, INV), &[Tv(942)]),
    ((1425, false, NAND2), &[Tv(1422), Tv(1424)]),
    ((1428, false, NAND2), &[Tv(999), Tv(1427)]),
];

static LEVEL_149: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((365, false, INV), &[Arg(0, 3)]),
    ((1943, false, NAND2), &[Tv(1565), Tv(1942)]),
    ((1945, false, NAND2), &[Tv(1566), Tv(1944)]),
    ((1947, false, AND2), &[Tv(1144), Tv(1560)]),
    ((1949, false, NAND2), &[Tv(1566), Tv(1937)]),
    ((1985, false, INV), &[Tv(1984)]),
    ((206, false, NAND2), &[Tv(202), Tv(204)]),
    ((208, false, NAND2), &[Tv(201), Tv(204)]),
    ((908, false, AND2), &[Tv(902), Tv(907)]),
    ((911, false, AND2), &[Tv(909), Tv(910)]),
    ((913, false, AND2), &[Tv(894), Tv(910)]),
    ((1426, false, AND2), &[Tv(998), Tv(1425)]),
    ((1429, false, INV), &[Tv(1428)]),
    ((1433, false, AND2), &[Tv(943), Tv(992)]),
];

static LEVEL_150: [((usize, bool, CellType), &[GateInput]); 16] = [
    ((1946, false, AND2), &[Tv(1943), Tv(1945)]),
    ((1948, false, XNOR2), &[Arg(0, 3), Tv(1947)]),
    ((1950, false, NAND2), &[Tv(1944), Tv(1949)]),
    ((1952, false, OR2), &[Tv(1938), Tv(1941)]),
    ((1953, false, NAND2), &[Tv(1937), Tv(1943)]),
    ((198, false, INV), &[Tv(197)]),
    ((205, false, INV), &[Tv(204)]),
    ((207, false, NAND2), &[Tv(2015), Tv(206)]),
    ((209, false, OR2), &[Tv(2015), Tv(208)]),
    ((212, false, AND2), &[Tv(1985), Tv(2012)]),
    ((213, false, XNOR2), &[Tv(365), Tv(1977)]),
    ((912, false, OR2), &[Tv(908), Tv(911)]),
    ((914, false, NAND2), &[Tv(900), Tv(913)]),
    ((1431, false, NOR2), &[Tv(1426), Tv(1429)]),
    ((1434, false, XNOR2), &[Tv(673), Tv(1433)]),
    ((1437, false, NAND2), &[Tv(1420), Tv(1426)]),
];

static LEVEL_151: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1951, false, NAND2), &[Tv(1939), Tv(1950)]),
    ((1954, false, AND2), &[Tv(1952), Tv(1953)]),
    ((1957, false, NAND2), &[Tv(1946), Tv(1948)]),
    ((210, false, AND2), &[Tv(207), Tv(209)]),
    ((214, false, XNOR2), &[Tv(212), Tv(213)]),
    ((216, false, NAND2), &[Tv(199), Tv(205)]),
    ((221, false, NAND2), &[Tv(198), Tv(207)]),
    ((359, false, OR2), &[Tv(320), Tv(357)]),
    ((360, false, NAND2), &[Arg(0, 33), Tv(357)]),
    ((917, false, NAND2), &[Tv(912), Tv(914)]),
    ((922, false, NAND2), &[Tv(911), Tv(914)]),
    ((1436, false, NAND2), &[Tv(1431), Tv(1434)]),
    ((1438, false, NAND2), &[Tv(1422), Tv(1437)]),
    ((1443, false, XNOR2), &[Tv(1420), Tv(1426)]),
];

static LEVEL_152: [((usize, bool, CellType), &[GateInput]); 13] = [
    ((1122, false, INV), &[Tv(1111)]),
    ((1956, false, NAND2), &[Tv(1951), Tv(1954)]),
    ((1958, false, NAND2), &[Tv(1951), Tv(1957)]),
    ((215, false, NAND2), &[Tv(210), Tv(214)]),
    ((217, false, AND2), &[Tv(202), Tv(216)]),
    ((222, false, NAND2), &[Tv(216), Tv(221)]),
    ((361, false, AND2), &[Tv(359), Tv(360)]),
    ((903, false, INV), &[Tv(902)]),
    ((916, false, INV), &[Tv(914)]),
    ((918, false, NAND2), &[Tv(907), Tv(917)]),
    ((923, false, OR2), &[Tv(907), Tv(922)]),
    ((1439, false, OR2), &[Tv(1427), Tv(1438)]),
    ((1444, false, NAND2), &[Tv(1436), Tv(1443)]),
];

static LEVEL_153: [((usize, bool, CellType), &[GateInput]); 14] = [
    ((1002, false, INV), &[Tv(991)]),
    ((1562, false, AND2), &[Tv(1122), Tv(1560)]),
    ((1955, false, NAND2), &[Tv(1952), Tv(1953)]),
    ((1959, false, AND2), &[Tv(1956), Tv(1958)]),
    ((1978, false, XNOR2), &[Arg(0, 2), Tv(1976)]),
    ((218, false, NAND2), &[Tv(208), Tv(217)]),
    ((223, false, NAND2), &[Tv(215), Tv(222)]),
    ((362, false, XNOR2), &[Arg(0, 34), Tv(361)]),
    ((919, false, NAND2), &[Tv(903), Tv(918)]),
    ((920, false, NAND2), &[Tv(908), Tv(916)]),
    ((924, false, AND2), &[Tv(918), Tv(923)]),
    ((941, false, INV), &[Tv(940)]),
    ((1435, false, INV), &[Tv(1434)]),
    ((1445, false, AND2), &[Tv(1439), Tv(1444)]),
];

static LEVEL_154: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((662, false, INV), &[Arg(0, 34)]),
    ((1563, false, XNOR2), &[Tv(354), Tv(1562)]),
    ((1960, false, NOR2), &[Tv(1948), Tv(1951)]),
    ((1963, false, XNOR2), &[Tv(1948), Tv(1959)]),
    ((1967, false, NAND2), &[Tv(1955), Tv(1958)]),
    ((1979, false, INV), &[Tv(1978)]),
    ((219, false, INV), &[Tv(218)]),
    ((220, false, OR2), &[Tv(215), Tv(218)]),
    ((224, false, NAND2), &[Tv(218), Tv(223)]),
    ((229, false, NAND2), &[Tv(1002), Tv(1978)]),
    ((921, false, NAND2), &[Tv(919), Tv(920)]),
    ((927, false, NAND2), &[Tv(362), Tv(924)]),
    ((929, false, AND2), &[Tv(912), Tv(922)]),
    ((994, false, AND2), &[Tv(941), Tv(992)]),
    ((1432, false, INV), &[Tv(1431)]),
    ((1440, false, INV), &[Tv(1439)]),
    ((1446, false, OR2), &[Tv(1435), Tv(1445)]),
];

static LEVEL_155: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((1961, false, NAND2), &[Tv(1946), Tv(1960)]),
    ((1962, false, XNOR2), &[Tv(1946), Tv(1960)]),
    ((1965, false, NAND2), &[Tv(1563), Tv(1963)]),
    ((1968, false, AND2), &[Tv(1956), Tv(1967)]),
    ((211, false, INV), &[Tv(210)]),
    ((225, false, NAND2), &[Tv(214), Tv(224)]),
    ((228, false, OR2), &[Tv(1979), Tv(2012)]),
    ((230, false, NAND2), &[Tv(212), Tv(229)]),
    ((232, false, OR2), &[Tv(219), Tv(222)]),
    ((236, false, AND2), &[Tv(220), Tv(223)]),
    ((928, false, NAND2), &[Tv(921), Tv(927)]),
    ((930, false, NAND2), &[Tv(920), Tv(929)]),
    ((995, false, XNOR2), &[Tv(662), Tv(994)]),
    ((1442, false, OR2), &[Tv(1436), Tv(1439)]),
    ((1447, false, NAND2), &[Tv(1432), Tv(1446)]),
    ((1449, false, OR2), &[Tv(1440), Tv(1443)]),
    ((1454, false, XNOR2), &[Tv(1434), Tv(1445)]),
];

static LEVEL_156: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((1966, false, NAND2), &[Tv(1962), Tv(1965)]),
    ((1969, false, NAND2), &[Tv(1961), Tv(1968)]),
    ((226, false, NAND2), &[Tv(211), Tv(225)]),
    ((231, false, AND2), &[Tv(228), Tv(230)]),
    ((233, false, XOR2), &[Tv(214), Tv(224)]),
    ((237, false, NAND2), &[Tv(232), Tv(236)]),
    ((931, false, NAND2), &[Tv(928), Tv(930)]),
    ((1448, false, NAND2), &[Tv(1442), Tv(1447)]),
    ((1450, false, AND2), &[Tv(1444), Tv(1449)]),
    ((1456, false, NAND2), &[Tv(995), Tv(1454)]),
];

static LEVEL_157: [((usize, bool, CellType), &[GateInput]); 8] = [
    ((1970, false, NAND2), &[Tv(1966), Tv(1969)]),
    ((227, false, NAND2), &[Tv(220), Tv(226)]),
    ((234, false, NAND2), &[Tv(231), Tv(233)]),
    ((238, false, INV), &[Tv(237)]),
    ((925, false, INV), &[Tv(924)]),
    ((932, false, NAND2), &[Tv(362), Tv(931)]),
    ((1451, false, NAND2), &[Tv(1442), Tv(1450)]),
    ((1457, false, NAND2), &[Tv(1448), Tv(1456)]),
];

static LEVEL_158: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((1964, false, INV), &[Tv(1963)]),
    ((1971, false, NAND2), &[Tv(1563), Tv(1970)]),
    ((235, false, NAND2), &[Tv(227), Tv(234)]),
    ((246, false, OR2), &[Tv(234), Tv(237)]),
    ((247, false, OR2), &[Tv(227), Tv(238)]),
    ((933, false, XOR2), &[Tv(362), Tv(931)]),
    ((996, false, INV), &[Tv(995)]),
    ((1458, false, AND2), &[Tv(1451), Tv(1457)]),
    ((1467, false, NAND2), &[Tv(925), Tv(932)]),
    ((1468, false, OR2), &[Tv(927), Tv(930)]),
];

static LEVEL_159: [((usize, bool, CellType), &[GateInput]); 11] = [
    ((1972, false, XOR2), &[Tv(1563), Tv(1970)]),
    ((239, false, AND2), &[Tv(235), Tv(237)]),
    ((249, false, AND2), &[Tv(246), Tv(247)]),
    ((254, false, NAND2), &[Tv(1964), Tv(1971)]),
    ((255, false, OR2), &[Tv(1965), Tv(1969)]),
    ((1455, false, INV), &[Tv(1454)]),
    ((1459, false, NOR2), &[Tv(996), Tv(1458)]),
    ((1460, false, XNOR2), &[Tv(995), Tv(1458)]),
    ((1469, false, AND2), &[Tv(1467), Tv(1468)]),
    ((1470, false, AND2), &[Tv(314), Tv(933)]),
    ((1484, false, OR2), &[Tv(921), Tv(929)]),
];

static LEVEL_160: [((usize, bool, CellType), &[GateInput]); 12] = [
    ((313, false, INV), &[Arg(1, 1)]),
    ((240, false, XNOR2), &[Tv(1979), Tv(239)]),
    ((248, false, NAND2), &[Tv(233), Tv(246)]),
    ((250, false, NAND2), &[Tv(231), Tv(249)]),
    ((256, false, AND2), &[Tv(254), Tv(255)]),
    ((257, false, AND2), &[Tv(1111), Tv(1972)]),
    ((265, false, AND2), &[Tv(1965), Tv(1968)]),
    ((1453, false, OR2), &[Tv(1448), Tv(1450)]),
    ((1476, false, AND2), &[Tv(940), Tv(1460)]),
    ((1479, false, XNOR2), &[Tv(1455), Tv(1459)]),
    ((1485, false, AND2), &[Tv(928), Tv(1484)]),
    ((1488, false, NAND2), &[Tv(1469), Tv(1470)]),
];

static LEVEL_161: [((usize, bool, CellType), &[GateInput]); 10] = [
    ((1024, false, AND2), &[Tv(313), Arg(1, 0)]),
    ((244, false, AND2), &[Tv(991), Tv(240)]),
    ((251, false, NAND2), &[Tv(248), Tv(250)]),
    ((263, false, NAND2), &[Tv(256), Tv(257)]),
    ((266, false, OR2), &[Tv(1962), Tv(265)]),
    ((1478, false, OR2), &[Tv(1451), Tv(1456)]),
    ((1486, false, AND2), &[Tv(1468), Tv(1485)]),
    ((1489, false, OR2), &[Tv(1485), Tv(1488)]),
    ((1496, false, AND2), &[Tv(1453), Tv(1457)]),
    ((1498, false, AND2), &[Tv(1476), Tv(1479)]),
];

static LEVEL_162: [((usize, bool, CellType), &[GateInput]); 17] = [
    ((343, false, INV), &[Arg(1, 0)]),
    ((1035, false, INV), &[Tv(1024)]),
    ((1046, false, NOR2), &[Arg(1, 1), Arg(1, 0)]),
    ((1973, false, NOR2), &[Tv(1111), Tv(1560)]),
    ((258, false, NOR2), &[Tv(1560), Tv(257)]),
    ((264, false, NAND2), &[Tv(1561), Tv(263)]),
    ((267, false, AND2), &[Tv(1966), Tv(266)]),
    ((270, false, NAND2), &[Tv(235), Tv(249)]),
    ((271, false, AND2), &[Tv(244), Tv(251)]),
    ((315, false, INV), &[Tv(314)]),
    ((1461, false, NOR2), &[Tv(940), Tv(992)]),
    ((1462, false, AND2), &[Arg(1, 1), Arg(1, 0)]),
    ((1477, false, NOR2), &[Tv(992), Tv(1476)]),
    ((1490, false, AND2), &[Tv(357), Tv(1489)]),
    ((1491, false, NAND2), &[Tv(1486), Tv(1488)]),
    ((1497, false, NAND2), &[Tv(1478), Tv(1496)]),
    ((1499, false, NOR2), &[Tv(992), Tv(1498)]),
];

static LEVEL_163: [((usize, bool, CellType), &[GateInput]); 21] = [
    ((1013, false, XNOR2), &[Arg(0, 1), Arg(0, 0)]),
    ((1974, false, XNOR2), &[Tv(1972), Tv(1973)]),
    ((245, false, NOR2), &[Tv(2013), Tv(244)]),
    ((259, false, XNOR2), &[Tv(256), Tv(258)]),
    ((268, false, XNOR2), &[Tv(264), Tv(267)]),
    ((272, false, NOR2), &[Tv(2013), Tv(271)]),
    ((278, false, OR2), &[Tv(263), Tv(267)]),
    ((279, false, NOR2), &[Tv(1035), Tv(1560)]),
    ((281, false, NAND2), &[Tv(270), Tv(271)]),
    ((282, false, AND2), &[Tv(1046), Tv(2012)]),
    ((316, false, XNOR2), &[Arg(0, 33), Arg(0, 32)]),
    ((934, false, AND2), &[Tv(315), Tv(357)]),
    ((935, false, AND2), &[Arg(1, 1), Tv(343)]),
    ((1464, false, XNOR2), &[Tv(1460), Tv(1461)]),
    ((1471, false, NOR2), &[Tv(358), Tv(1470)]),
    ((1480, false, XNOR2), &[Tv(1477), Tv(1479)]),
    ((1487, false, NAND2), &[Tv(358), Tv(1486)]),
    ((1492, false, NAND2), &[Tv(1490), Tv(1491)]),
    ((1500, false, XNOR2), &[Tv(1497), Tv(1499)]),
    ((1507, false, NAND2), &[Tv(1497), Tv(1498)]),
    ((1508, false, AND2), &[Tv(993), Tv(1462)]),
];

static LEVEL_164: [((usize, bool, CellType), &[GateInput]); 25] = [
    ((980, false, NAND2), &[Arg(0, 1), Arg(1, 1)]),
    ((1057, false, INV), &[Tv(1046)]),
    ((1079, false, OR2), &[Tv(1013), Tv(1035)]),
    ((1975, false, NAND2), &[Tv(1024), Tv(1974)]),
    ((242, false, NAND2), &[Arg(1, 1), Arg(0, 2)]),
    ((252, false, XNOR2), &[Tv(245), Tv(251)]),
    ((260, false, NAND2), &[Tv(1024), Tv(259)]),
    ((261, false, NAND2), &[Arg(1, 1), Arg(0, 3)]),
    ((269, false, NAND2), &[Tv(1024), Tv(268)]),
    ((273, false, XNOR2), &[Tv(270), Tv(272)]),
    ((275, false, NAND2), &[Arg(0, 4), Arg(1, 1)]),
    ((280, false, NAND2), &[Tv(278), Tv(279)]),
    ((283, false, NAND2), &[Tv(281), Tv(282)]),
    ((317, false, XNOR2), &[Tv(343), Tv(316)]),
    ((936, false, XNOR2), &[Tv(933), Tv(934)]),
    ((939, false, NAND2), &[Tv(313), Arg(0, 34)]),
    ((1465, false, NAND2), &[Tv(1462), Tv(1464)]),
    ((1472, false, XNOR2), &[Tv(1469), Tv(1471)]),
    ((1475, false, NAND2), &[Tv(313), Arg(0, 35)]),
    ((1481, false, NAND2), &[Tv(1462), Tv(1480)]),
    ((1493, false, NAND2), &[Tv(1487), Tv(1492)]),
    ((1501, false, NAND2), &[Tv(1462), Tv(1500)]),
    ((1502, false, NAND2), &[Tv(313), Arg(0, 36)]),
    ((1506, false, NAND2), &[Tv(935), Tv(1490)]),
    ((1509, false, NAND2), &[Tv(1507), Tv(1508)]),
];

static LEVEL_165: [((usize, bool, CellType), &[GateInput]); 72] = [
    ((1068, false, NAND2), &[Tv(1013), Tv(1046)]),
    ((1090, false, AND2), &[Tv(980), Tv(1079)]),
    ((241, false, OR2), &[Tv(1057), Tv(240)]),
    ((243, false, AND2), &[Tv(1975), Tv(242)]),
    ((253, false, NAND2), &[Tv(1046), Tv(252)]),
    ((262, false, AND2), &[Tv(260), Tv(261)]),
    ((274, false, NAND2), &[Tv(1046), Tv(273)]),
    ((276, false, AND2), &[Tv(269), Tv(275)]),
    ((277, false, NAND2), &[Arg(1, 1), Arg(0, 5)]),
    ((284, false, AND2), &[Tv(280), Tv(283)]),
    ((285, false, NAND2), &[Arg(1, 1), Arg(0, 6)]),
    ((286, false, NAND2), &[Arg(1, 1), Arg(0, 7)]),
    ((287, false, NAND2), &[Arg(1, 1), Arg(0, 8)]),
    ((288, false, NAND2), &[Arg(1, 1), Arg(0, 9)]),
    ((289, false, NAND2), &[Arg(1, 1), Arg(0, 10)]),
    ((290, false, NAND2), &[Arg(1, 1), Arg(0, 11)]),
    ((291, false, NAND2), &[Arg(1, 1), Arg(0, 12)]),
    ((292, false, NAND2), &[Arg(1, 1), Arg(0, 13)]),
    ((293, false, NAND2), &[Arg(1, 1), Arg(0, 14)]),
    ((294, false, NAND2), &[Arg(1, 1), Arg(0, 15)]),
    ((295, false, NAND2), &[Arg(1, 1), Arg(0, 16)]),
    ((296, false, NAND2), &[Arg(1, 1), Arg(0, 17)]),
    ((297, false, NAND2), &[Arg(1, 1), Arg(0, 18)]),
    ((298, false, NAND2), &[Arg(1, 1), Arg(0, 19)]),
    ((299, false, NAND2), &[Arg(1, 1), Arg(0, 20)]),
    ((300, false, NAND2), &[Arg(1, 1), Arg(0, 21)]),
    ((302, false, NAND2), &[Arg(1, 1), Arg(0, 22)]),
    ((303, false, NAND2), &[Arg(1, 1), Arg(0, 23)]),
    ((304, false, NAND2), &[Arg(1, 1), Arg(0, 24)]),
    ((305, false, NAND2), &[Arg(1, 1), Arg(0, 25)]),
    ((306, false, NAND2), &[Arg(1, 1), Arg(0, 26)]),
    ((308, false, NAND2), &[Arg(1, 1), Arg(0, 27)]),
    ((309, false, NAND2), &[Arg(1, 1), Arg(0, 28)]),
    ((310, false, NAND2), &[Arg(1, 1), Arg(0, 29)]),
    ((311, false, NAND2), &[Arg(1, 1), Arg(0, 30)]),
    ((312, false, NAND2), &[Arg(1, 1), Arg(0, 31)]),
    ((318, false, NAND2), &[Arg(1, 1), Tv(317)]),
    ((319, false, NAND2), &[Tv(313), Arg(0, 33)]),
    ((938, false, NAND2), &[Tv(935), Tv(936)]),
    ((1466, false, AND2), &[Tv(939), Tv(1465)]),
    ((1474, false, NAND2), &[Tv(935), Tv(1472)]),
    ((1482, false, AND2), &[Tv(1475), Tv(1481)]),
    ((1495, false, NAND2), &[Tv(935), Tv(1493)]),
    ((1503, false, AND2), &[Tv(1501), Tv(1502)]),
    ((1505, false, NAND2), &[Tv(313), Arg(0, 37)]),
    ((1510, false, AND2), &[Tv(1506), Tv(1509)]),
    ((1511, false, NAND2), &[Tv(313), Arg(0, 38)]),
    ((1512, false, NAND2), &[Tv(313), Arg(0, 39)]),
    ((1514, false, NAND2), &[Tv(313), Arg(0, 40)]),
    ((1515, false, NAND2), &[Tv(313), Arg(0, 41)]),
    ((1516, false, NAND2), &[Tv(313), Arg(0, 42)]),
    ((1517, false, NAND2), &[Tv(313), Arg(0, 43)]),
    ((1518, false, NAND2), &[Tv(313), Arg(0, 44)]),
    ((1520, false, NAND2), &[Tv(313), Arg(0, 45)]),
    ((1521, false, NAND2), &[Tv(313), Arg(0, 46)]),
    ((1522, false, NAND2), &[Tv(313), Arg(0, 47)]),
    ((1523, false, NAND2), &[Tv(313), Arg(0, 48)]),
    ((1524, false, NAND2), &[Tv(313), Arg(0, 49)]),
    ((1526, false, NAND2), &[Tv(313), Arg(0, 50)]),
    ((1527, false, NAND2), &[Tv(313), Arg(0, 51)]),
    ((1528, false, NAND2), &[Tv(313), Arg(0, 52)]),
    ((1529, false, NAND2), &[Tv(313), Arg(0, 53)]),
    ((1530, false, NAND2), &[Tv(313), Arg(0, 54)]),
    ((1532, false, NAND2), &[Tv(313), Arg(0, 55)]),
    ((1533, false, NAND2), &[Tv(313), Arg(0, 56)]),
    ((1534, false, NAND2), &[Tv(313), Arg(0, 57)]),
    ((1535, false, NAND2), &[Tv(313), Arg(0, 58)]),
    ((1536, false, NAND2), &[Tv(313), Arg(0, 59)]),
    ((1538, false, NAND2), &[Tv(313), Arg(0, 60)]),
    ((1539, false, NAND2), &[Tv(313), Arg(0, 61)]),
    ((1540, false, NAND2), &[Tv(313), Arg(0, 62)]),
    ((1541, false, NAND2), &[Tv(313), Arg(0, 63)]),
];

static LEVEL_166: [((usize, bool, CellType), &[GateInput]); 64] = [
    ((0, true, XNOR2), &[Arg(0, 0), Arg(1, 1)]),
    ((1, true, NAND2), &[Tv(1068), Tv(1090)]),
    ((2, true, NAND2), &[Tv(241), Tv(243)]),
    ((3, true, NAND2), &[Tv(253), Tv(262)]),
    ((4, true, NAND2), &[Tv(274), Tv(276)]),
    ((5, true, NAND2), &[Tv(277), Tv(284)]),
    ((6, true, NAND2), &[Tv(284), Tv(285)]),
    ((7, true, NAND2), &[Tv(284), Tv(286)]),
    ((8, true, NAND2), &[Tv(284), Tv(287)]),
    ((9, true, NAND2), &[Tv(284), Tv(288)]),
    ((10, true, NAND2), &[Tv(284), Tv(289)]),
    ((11, true, NAND2), &[Tv(284), Tv(290)]),
    ((12, true, NAND2), &[Tv(284), Tv(291)]),
    ((13, true, NAND2), &[Tv(284), Tv(292)]),
    ((14, true, NAND2), &[Tv(284), Tv(293)]),
    ((15, true, NAND2), &[Tv(284), Tv(294)]),
    ((16, true, NAND2), &[Tv(284), Tv(295)]),
    ((17, true, NAND2), &[Tv(284), Tv(296)]),
    ((18, true, NAND2), &[Tv(284), Tv(297)]),
    ((19, true, NAND2), &[Tv(284), Tv(298)]),
    ((20, true, NAND2), &[Tv(284), Tv(299)]),
    ((21, true, NAND2), &[Tv(284), Tv(300)]),
    ((22, true, NAND2), &[Tv(284), Tv(302)]),
    ((23, true, NAND2), &[Tv(284), Tv(303)]),
    ((24, true, NAND2), &[Tv(284), Tv(304)]),
    ((25, true, NAND2), &[Tv(284), Tv(305)]),
    ((26, true, NAND2), &[Tv(284), Tv(306)]),
    ((27, true, NAND2), &[Tv(284), Tv(308)]),
    ((28, true, NAND2), &[Tv(284), Tv(309)]),
    ((29, true, NAND2), &[Tv(284), Tv(310)]),
    ((30, true, NAND2), &[Tv(284), Tv(311)]),
    ((31, true, NAND2), &[Tv(284), Tv(312)]),
    ((32, true, XOR2), &[Arg(1, 1), Arg(0, 32)]),
    ((33, true, NAND2), &[Tv(318), Tv(319)]),
    ((34, true, NAND2), &[Tv(938), Tv(1466)]),
    ((35, true, NAND2), &[Tv(1474), Tv(1482)]),
    ((36, true, NAND2), &[Tv(1495), Tv(1503)]),
    ((37, true, NAND2), &[Tv(1505), Tv(1510)]),
    ((38, true, NAND2), &[Tv(1510), Tv(1511)]),
    ((39, true, NAND2), &[Tv(1510), Tv(1512)]),
    ((40, true, NAND2), &[Tv(1510), Tv(1514)]),
    ((41, true, NAND2), &[Tv(1510), Tv(1515)]),
    ((42, true, NAND2), &[Tv(1510), Tv(1516)]),
    ((43, true, NAND2), &[Tv(1510), Tv(1517)]),
    ((44, true, NAND2), &[Tv(1510), Tv(1518)]),
    ((45, true, NAND2), &[Tv(1510), Tv(1520)]),
    ((46, true, NAND2), &[Tv(1510), Tv(1521)]),
    ((47, true, NAND2), &[Tv(1510), Tv(1522)]),
    ((48, true, NAND2), &[Tv(1510), Tv(1523)]),
    ((49, true, NAND2), &[Tv(1510), Tv(1524)]),
    ((50, true, NAND2), &[Tv(1510), Tv(1526)]),
    ((51, true, NAND2), &[Tv(1510), Tv(1527)]),
    ((52, true, NAND2), &[Tv(1510), Tv(1528)]),
    ((53, true, NAND2), &[Tv(1510), Tv(1529)]),
    ((54, true, NAND2), &[Tv(1510), Tv(1530)]),
    ((55, true, NAND2), &[Tv(1510), Tv(1532)]),
    ((56, true, NAND2), &[Tv(1510), Tv(1533)]),
    ((57, true, NAND2), &[Tv(1510), Tv(1534)]),
    ((58, true, NAND2), &[Tv(1510), Tv(1535)]),
    ((59, true, NAND2), &[Tv(1510), Tv(1536)]),
    ((60, true, NAND2), &[Tv(1510), Tv(1538)]),
    ((61, true, NAND2), &[Tv(1510), Tv(1539)]),
    ((62, true, NAND2), &[Tv(1510), Tv(1540)]),
    ((63, true, NAND2), &[Tv(1510), Tv(1541)]),
];

static PRUNE_69: [usize; 16] = [
  1688,
  1504,
  463,
  1695,
  1697,
  1152,
  471,
  1691,
  1149,
  1705,
  622,
  1701,
  620,
  2259,
  633,
  634,
];

static PRUNE_7: [usize; 2] = [
  328,
  1989,
];

static PRUNE_38: [usize; 6] = [
  1051,
  527,
  522,
  2146,
  1062,
  2154,
];

static PRUNE_100: [usize; 26] = [
  63,
  741,
  425,
  736,
  422,
  738,
  734,
  54,
  56,
  1248,
  745,
  746,
  744,
  743,
  2071,
  1797,
  1255,
  1795,
  1249,
  1251,
  1792,
  51,
  1267,
  2078,
  772,
  2077,
];

static PRUNE_131: [usize; 14] = [
  379,
  1369,
  391,
  1890,
  398,
  2023,
  1888,
  1881,
  1882,
  144,
  140,
  2030,
  1895,
  1892,
];

static PRUNE_162: [usize; 15] = [
  235,
  1111,
  1966,
  1561,
  249,
  266,
  940,
  1478,
  257,
  1476,
  1496,
  1489,
  314,
  992,
  1488,
];

static PRUNE_86: [usize; 21] = [
  18,
  696,
  1192,
  13,
  690,
  1186,
  9,
  703,
  2101,
  1200,
  2102,
  1193,
  699,
  1194,
  446,
  1756,
  445,
  1206,
  8,
  1,
  3,
];

static PRUNE_55: [usize; 18] = [
  2087,
  1106,
  1103,
  1645,
  2205,
  2117,
  2207,
  1665,
  583,
  2203,
  1662,
  490,
  577,
  574,
  592,
  2121,
  589,
  2209,
];

static PRUNE_24: [usize; 3] = [
  2004,
  986,
  348,
];

static PRUNE_117: [usize; 16] = [
  108,
  2049,
  2046,
  100,
  2053,
  431,
  1845,
  1301,
  399,
  806,
  1843,
  1838,
  1837,
  1311,
  1850,
  1847,
];

static PRUNE_148: [usize; 15] = [
  200,
  1424,
  203,
  1935,
  942,
  897,
  899,
  892,
  891,
  999,
  906,
  1133,
  1940,
  905,
  901,
];

static PRUNE_103: [usize; 13] = [
  962,
  57,
  464,
  416,
  2056,
  1575,
  757,
  2063,
  1805,
  1580,
  1579,
  48,
  404,
];

static PRUNE_72: [usize; 21] = [
  1010,
  1147,
  650,
  468,
  467,
  643,
  1006,
  2268,
  645,
  2265,
  1157,
  1156,
  1158,
  838,
  1151,
  1153,
  1148,
  1483,
  2111,
  2262,
  2261,
];

static PRUNE_41: [usize; 33] = [
  1596,
  1056,
  2130,
  1607,
  1021,
  1609,
  1063,
  1603,
  607,
  1599,
  538,
  2162,
  2161,
  2158,
  533,
  1617,
  531,
  2155,
  1615,
  2156,
  483,
  1069,
  1612,
  529,
  500,
  2169,
  2123,
  2170,
  547,
  2166,
  544,
  904,
  539,
];

static PRUNE_10: [usize; 3] = [
  331,
  1992,
  323,
];

static PRUNE_165: [usize; 30] = [
  1506,
  1057,
  240,
  242,
  283,
  1501,
  1502,
  280,
  1472,
  252,
  1509,
  1013,
  1465,
  313,
  1079,
  1481,
  939,
  260,
  1975,
  261,
  1475,
  936,
  980,
  935,
  1046,
  275,
  1493,
  317,
  273,
  269,
];

static PRUNE_134: [usize; 15] = [
  155,
  1368,
  1370,
  1367,
  1363,
  1904,
  842,
  1199,
  162,
  1379,
  1374,
  850,
  844,
  950,
  857,
];

static PRUNE_120: [usize; 13] = [
  110,
  1323,
  1325,
  1321,
  1859,
  1317,
  118,
  1328,
  807,
  728,
  1265,
  810,
  812,
];

static PRUNE_89: [usize; 19] = [
  1769,
  1768,
  436,
  26,
  702,
  1755,
  1751,
  1753,
  805,
  1211,
  1208,
  1207,
  713,
  2081,
  497,
  1581,
  1761,
  1757,
  1758,
];

static PRUNE_58: [usize; 13] = [
  1100,
  1101,
  871,
  2223,
  2219,
  1113,
  477,
  1110,
  1108,
  1658,
  1117,
  1525,
  2217,
];

static PRUNE_27: [usize; 4] = [
  1023,
  2007,
  988,
  351,
];

static PRUNE_151: [usize; 14] = [
  198,
  199,
  911,
  1950,
  209,
  207,
  1426,
  1422,
  205,
  1420,
  1437,
  213,
  320,
  1939,
];

static PRUNE_156: [usize; 12] = [
  236,
  211,
  1961,
  214,
  232,
  1447,
  1449,
  230,
  1444,
  228,
  224,
  225,
];

static PRUNE_32: [usize; 16] = [
  2138,
  1553,
  515,
  511,
  2136,
  509,
  506,
  2149,
  523,
  2147,
  520,
  2144,
  2141,
  1031,
  1025,
  1586,
];

static PRUNE_63: [usize; 17] = [
  1678,
  615,
  2239,
  614,
  474,
  2233,
  611,
  473,
  1014,
  1123,
  1115,
  1675,
  1136,
  860,
  1670,
  1130,
  1127,
];

static PRUNE_94: [usize; 15] = [
  1777,
  1774,
  1773,
  435,
  432,
  1782,
  1779,
  40,
  1214,
  486,
  710,
  1766,
  1219,
  721,
  2074,
];

static PRUNE_125: [usize; 14] = [
  830,
  1327,
  1339,
  1336,
  1332,
  1330,
  717,
  131,
  1254,
  1344,
  2037,
  1356,
  135,
  1352,
];

static PRUNE_18: [usize; 2] = [
  1999,
  342,
];

static PRUNE_80: [usize; 9] = [
  1729,
  2286,
  2105,
  2279,
  1738,
  2098,
  968,
  2289,
  4,
];

static PRUNE_49: [usize; 13] = [
  1637,
  1092,
  1635,
  1634,
  1544,
  565,
  564,
  492,
  484,
  1630,
  1627,
  1629,
  1087,
];

static PRUNE_111: [usize; 7] = [
  1280,
  1320,
  1828,
  2050,
  1302,
  1296,
  92,
];

static PRUNE_142: [usize; 22] = [
  872,
  874,
  870,
  867,
  883,
  1155,
  878,
  177,
  174,
  170,
  172,
  2020,
  2017,
  1926,
  369,
  1406,
  182,
  181,
  183,
  363,
  1396,
  180,
];

static PRUNE_35: [usize; 15] = [
  1598,
  1053,
  1595,
  1592,
  2133,
  1047,
  1590,
  526,
  1601,
  1034,
  984,
  2152,
  618,
  503,
  1039,
];

static PRUNE_4: [usize; 2] = [
  325,
  1980,
];

static PRUNE_66: [usize; 11] = [
  1686,
  2241,
  2236,
  975,
  2238,
  2237,
  2095,
  2251,
  2247,
  2242,
  2253,
];

static PRUNE_97: [usize; 8] = [
  740,
  1237,
  1241,
  1375,
  42,
  1791,
  53,
  45,
];

static PRUNE_128: [usize; 18] = [
  829,
  1868,
  148,
  824,
  826,
  841,
  389,
  385,
  836,
  835,
  832,
  834,
  1346,
  1886,
  951,
  139,
  137,
  134,
];

static PRUNE_159: [usize; 14] = [
  921,
  1458,
  237,
  1454,
  1969,
  1563,
  1970,
  929,
  1467,
  1964,
  247,
  1971,
  996,
  995,
];

static PRUNE_21: [usize; 4] = [
  2090,
  2089,
  458,
  459,
];

static PRUNE_52: [usize; 14] = [
  1096,
  2086,
  977,
  1650,
  2192,
  1648,
  582,
  2206,
  2201,
  2197,
  573,
  2198,
  2120,
  1083,
];

static PRUNE_83: [usize; 15] = [
  693,
  689,
  686,
  1452,
  2284,
  1739,
  1198,
  697,
  2293,
  1746,
  1585,
  685,
  456,
  678,
  451,
];

static PRUNE_114: [usize; 16] = [
  786,
  782,
  104,
  958,
  796,
  1294,
  797,
  1291,
  788,
  1305,
  1841,
  1299,
  1840,
  800,
  94,
  407,
];

static PRUNE_145: [usize; 21] = [
  1413,
  1415,
  1414,
  1412,
  1409,
  1924,
  1925,
  888,
  1921,
  1932,
  895,
  1929,
  1931,
  1930,
  1927,
  1402,
  321,
  997,
  1987,
  1400,
  1982,
];

static PRUNE_88: [usize; 9] = [
  1190,
  967,
  1203,
  1201,
  1197,
  1754,
  1215,
  1750,
  1430,
];

static PRUNE_150: [usize; 20] = [
  197,
  1949,
  913,
  1429,
  2015,
  204,
  206,
  673,
  1977,
  1433,
  1947,
  365,
  1943,
  1945,
  1944,
  1985,
  1941,
  1938,
  1937,
  900,
];

static PRUNE_57: [usize; 12] = [
  1099,
  597,
  1654,
  1109,
  1018,
  1107,
  2204,
  1659,
  578,
  1656,
  1116,
  2218,
];

static PRUNE_119: [usize; 18] = [
  1324,
  103,
  1322,
  956,
  1319,
  117,
  793,
  401,
  809,
  802,
  393,
  799,
  122,
  1316,
  1853,
  1313,
  1854,
  813,
];

static PRUNE_26: [usize; 2] = [
  2006,
  350,
];

static PRUNE_105: [usize; 22] = [
  1281,
  62,
  61,
  1814,
  1813,
  1272,
  1273,
  74,
  71,
  1246,
  749,
  69,
  66,
  764,
  759,
  761,
  1256,
  753,
  1270,
  1810,
  1266,
  1268,
];

static PRUNE_12: [usize; 4] = [
  334,
  1994,
  1558,
  337,
];

static PRUNE_136: [usize; 17] = [
  1911,
  1907,
  374,
  165,
  1382,
  387,
  1381,
  163,
  1373,
  158,
  2025,
  1567,
  1901,
  2032,
  1897,
  2028,
  1894,
];

static PRUNE_43: [usize; 7] = [
  1545,
  2177,
  1058,
  1076,
  1611,
  542,
  2163,
];

static PRUNE_74: [usize; 16] = [
  641,
  660,
  2275,
  1709,
  1708,
  666,
  1163,
  1160,
  2106,
  530,
  1722,
  1717,
  1719,
  1715,
  1716,
  1713,
];

static PRUNE_60: [usize; 25] = [
  2092,
  604,
  602,
  603,
  2226,
  1681,
  598,
  600,
  462,
  1679,
  1112,
  479,
  609,
  1105,
  563,
  2115,
  1664,
  1657,
  593,
  2216,
  1673,
  591,
  2215,
  2212,
  1667,
];

static PRUNE_29: [usize; 9] = [
  1551,
  959,
  2011,
  2009,
  356,
  985,
  353,
  1027,
  1037,
];

static PRUNE_122: [usize; 17] = [
  1869,
  1865,
  1862,
  420,
  1860,
  2040,
  119,
  1872,
  115,
  112,
  125,
  397,
  394,
  1856,
  2033,
  1852,
  1849,
];

static PRUNE_91: [usize; 16] = [
  965,
  1228,
  705,
  429,
  714,
  1212,
  39,
  35,
  438,
  1204,
  34,
  2083,
  1763,
  1762,
  2080,
  723,
];

static PRUNE_153: [usize; 12] = [
  918,
  1952,
  1953,
  916,
  208,
  923,
  1122,
  1976,
  217,
  908,
  903,
  361,
];

static PRUNE_77: [usize; 16] = [
  1463,
  827,
  2271,
  2264,
  1003,
  658,
  2282,
  657,
  2276,
  654,
  669,
  441,
  664,
  665,
  454,
  1172,
];

static PRUNE_46: [usize; 20] = [
  559,
  2180,
  556,
  2181,
  553,
  2178,
  2174,
  2131,
  2175,
  567,
  2185,
  1618,
  979,
  1070,
  2127,
  1628,
  1625,
  1622,
  1081,
  496,
];

static PRUNE_139: [usize; 9] = [
  1188,
  873,
  1001,
  1394,
  1392,
  167,
  1387,
  179,
  947,
];

static PRUNE_108: [usize; 24] = [
  1278,
  1821,
  960,
  418,
  1816,
  1818,
  1817,
  75,
  73,
  85,
  760,
  2066,
  2065,
  81,
  2061,
  413,
  1811,
  412,
  1808,
  771,
  1809,
  453,
  1576,
  766,
];

static PRUNE_15: [usize; 2] = [
  1995,
  338,
];

static PRUNE_161: [usize; 11] = [
  1456,
  1457,
  1453,
  250,
  1468,
  1962,
  248,
  1485,
  265,
  1451,
  991,
];

static PRUNE_20: [usize; 4] = [
  2001,
  345,
  336,
  1556,
];

static PRUNE_113: [usize; 24] = [
  785,
  2045,
  1820,
  781,
  957,
  777,
  1833,
  1834,
  1832,
  795,
  1827,
  2052,
  1826,
  86,
  1303,
  84,
  410,
  96,
  95,
  768,
  91,
  89,
  88,
  406,
];

static PRUNE_82: [usize; 24] = [
  1189,
  1004,
  1182,
  11,
  1184,
  1183,
  687,
  1744,
  1735,
  2292,
  2291,
  457,
  1181,
  816,
  680,
  1178,
  682,
  1174,
  677,
  1175,
  679,
  449,
  1441,
  450,
];

static PRUNE_51: [usize; 17] = [
  2184,
  1639,
  1638,
  558,
  1640,
  2085,
  571,
  976,
  572,
  2195,
  2191,
  1649,
  2186,
  2187,
  576,
  2200,
  2119,
];

static PRUNE_37: [usize; 9] = [
  1055,
  1593,
  1050,
  517,
  536,
  2153,
  1044,
  1587,
  2165,
];

static PRUNE_99: [usize; 16] = [
  424,
  421,
  737,
  1234,
  733,
  1788,
  475,
  1784,
  1240,
  1781,
  2070,
  1798,
  1794,
  1223,
  2073,
  1578,
];

static PRUNE_130: [usize; 19] = [
  1232,
  1366,
  1362,
  706,
  1880,
  840,
  839,
  837,
  1889,
  1350,
  1887,
  1571,
  1572,
  846,
  845,
  1359,
  1358,
  1355,
  2029,
];

static PRUNE_68: [usize; 15] = [
  1145,
  470,
  1143,
  1141,
  1137,
  1139,
  1138,
  1699,
  627,
  624,
  2244,
  623,
  619,
  637,
  632,
];

static PRUNE_6: [usize; 2] = [
  327,
  1988,
];

static PRUNE_23: [usize; 2] = [
  2002,
  346,
];

static PRUNE_116: [usize; 13] = [
  2048,
  106,
  779,
  791,
  1574,
  1844,
  1304,
  1842,
  1298,
  1836,
  97,
  1310,
  1307,
];

static PRUNE_147: [usize; 15] = [
  1417,
  196,
  195,
  193,
  190,
  371,
  191,
  1421,
  898,
  1934,
  896,
  890,
  367,
  324,
  189,
];

static PRUNE_85: [usize; 16] = [
  694,
  692,
  691,
  1185,
  12,
  14,
  1742,
  1196,
  2100,
  698,
  444,
  1748,
  7,
  1179,
  1760,
  0,
];

static PRUNE_54: [usize; 14] = [
  1102,
  1098,
  1652,
  1647,
  2116,
  489,
  2202,
  488,
  1661,
  580,
  1089,
  1542,
  588,
  586,
];

static PRUNE_40: [usize; 19] = [
  1597,
  1548,
  505,
  2129,
  1067,
  482,
  1065,
  1602,
  1604,
  1060,
  2160,
  1616,
  2159,
  982,
  532,
  528,
  499,
  543,
  2122,
];

static PRUNE_133: [usize; 14] = [
  1371,
  154,
  147,
  373,
  843,
  1376,
  1378,
  381,
  157,
  1891,
  853,
  847,
  1899,
  949,
];

static PRUNE_164: [usize; 28] = [
  245,
  1508,
  1462,
  1507,
  1500,
  282,
  281,
  933,
  1469,
  1471,
  343,
  251,
  1464,
  268,
  1974,
  1480,
  934,
  1024,
  259,
  279,
  278,
  1492,
  272,
  316,
  1490,
  1487,
  1035,
  270,
];

static PRUNE_71: [usize; 20] = [
  1146,
  1011,
  647,
  466,
  2272,
  1154,
  973,
  1690,
  1150,
  2250,
  2252,
  1711,
  2110,
  2109,
  1702,
  639,
  2258,
  2257,
  541,
  2255,
];

static PRUNE_102: [usize; 12] = [
  60,
  58,
  415,
  1364,
  748,
  68,
  1259,
  1257,
  2062,
  47,
  1261,
  1802,
];

static PRUNE_9: [usize; 2] = [
  330,
  1991,
];

static PRUNE_124: [usize; 17] = [
  1326,
  1867,
  821,
  822,
  121,
  1875,
  1335,
  1243,
  1329,
  130,
  1343,
  127,
  954,
  819,
  815,
  817,
  814,
];

static PRUNE_155: [usize; 21] = [
  920,
  1956,
  210,
  662,
  1967,
  1960,
  1439,
  1440,
  222,
  1436,
  219,
  1434,
  212,
  1432,
  1946,
  229,
  1446,
  1445,
  994,
  223,
  1443,
];

static PRUNE_93: [usize; 18] = [
  1236,
  1776,
  1772,
  1408,
  30,
  707,
  434,
  24,
  41,
  718,
  709,
  1582,
  727,
  1583,
  1764,
  1218,
  1759,
  1217,
];

static PRUNE_76: [usize; 15] = [
  2270,
  1007,
  1730,
  2267,
  1727,
  2285,
  971,
  2097,
  1168,
  667,
  663,
  2288,
  2263,
  452,
  1171,
];

static PRUNE_45: [usize; 18] = [
  2132,
  554,
  596,
  550,
  978,
  1619,
  1621,
  1613,
  1614,
  2126,
  2173,
  1632,
  2125,
  546,
  501,
  1624,
  494,
  495,
];

static PRUNE_14: [usize; 4] = [
  1996,
  430,
  339,
  2069,
];

static PRUNE_141: [usize; 17] = [
  875,
  1912,
  1914,
  1913,
  1411,
  1918,
  881,
  1919,
  879,
  1916,
  173,
  1390,
  2019,
  2016,
  169,
  1405,
  368,
];

static PRUNE_110: [usize; 15] = [
  1277,
  776,
  1293,
  1830,
  750,
  1290,
  1286,
  1825,
  1283,
  1285,
  774,
  770,
  1309,
  93,
  90,
];

static PRUNE_62: [usize; 12] = [
  2228,
  1012,
  2229,
  2225,
  472,
  1120,
  1660,
  1118,
  1114,
  1129,
  1125,
  1668,
];

static PRUNE_31: [usize; 6] = [
  2137,
  2135,
  512,
  507,
  1045,
  990,
];

static PRUNE_127: [usize; 19] = [
  831,
  828,
  2042,
  2043,
  823,
  1879,
  1877,
  388,
  384,
  1874,
  833,
  1348,
  128,
  1885,
  1340,
  142,
  2036,
  136,
  138,
];

static PRUNE_158: [usize; 10] = [
  238,
  931,
  932,
  930,
  1963,
  925,
  927,
  234,
  362,
  227,
];

static PRUNE_79: [usize; 18] = [
  1732,
  1731,
  1733,
  1728,
  1723,
  1725,
  2104,
  2283,
  519,
  1734,
  672,
  1170,
  671,
  1721,
  1180,
  1584,
  681,
  675,
];

static PRUNE_48: [usize; 9] = [
  2182,
  1543,
  569,
  2189,
  562,
  491,
  1074,
  1091,
  1086,
];

static PRUNE_17: [usize; 2] = [
  1998,
  341,
];

static PRUNE_144: [usize; 13] = [
  1407,
  886,
  1564,
  301,
  1920,
  885,
  877,
  1404,
  185,
  1403,
  1986,
  1399,
  945,
];

static PRUNE_96: [usize; 22] = [
  1235,
  1778,
  783,
  1227,
  731,
  732,
  1247,
  1786,
  1239,
  719,
  38,
  37,
  2068,
  32,
  1226,
  729,
  1222,
  725,
  2076,
  46,
  724,
  44,
];

static PRUNE_34: [usize; 11] = [
  514,
  516,
  1550,
  1589,
  525,
  2140,
  2142,
  1600,
  989,
  983,
  2151,
];

static PRUNE_149: [usize; 13] = [
  1144,
  1428,
  1425,
  201,
  943,
  894,
  1566,
  1565,
  909,
  910,
  998,
  1942,
  1984,
];

static PRUNE_65: [usize; 19] = [
  2094,
  1684,
  1683,
  552,
  617,
  2240,
  974,
  612,
  610,
  2235,
  1694,
  1693,
  1689,
  2232,
  626,
  2113,
  2246,
  1677,
  1132,
];

static PRUNE_87: [usize; 20] = [
  1191,
  966,
  19,
  1187,
  15,
  16,
  10,
  688,
  1202,
  2103,
  704,
  700,
  25,
  701,
  21,
  1419,
  1195,
  447,
  711,
  708,
];

static PRUNE_118: [usize; 19] = [
  2047,
  109,
  107,
  102,
  1858,
  101,
  392,
  114,
  792,
  111,
  1846,
  808,
  400,
  1839,
  801,
  798,
  955,
  1312,
  1308,
];

static PRUNE_56: [usize; 16] = [
  2088,
  2221,
  1653,
  1651,
  1644,
  1104,
  581,
  2199,
  1655,
  595,
  2214,
  2211,
  587,
  584,
  2208,
  2210,
];

static PRUNE_8: [usize; 2] = [
  329,
  1990,
];

static PRUNE_166: [usize; 72] = [
  289,
  243,
  288,
  290,
  241,
  286,
  285,
  1503,
  1505,
  287,
  284,
  300,
  1518,
  1517,
  1474,
  1068,
  297,
  1515,
  1514,
  299,
  1516,
  253,
  298,
  1512,
  1466,
  294,
  296,
  295,
  291,
  293,
  1511,
  1510,
  292,
  1529,
  311,
  312,
  1530,
  1526,
  308,
  1528,
  310,
  309,
  1527,
  1482,
  306,
  305,
  1523,
  262,
  1524,
  938,
  303,
  302,
  1520,
  1522,
  304,
  1521,
  1540,
  1495,
  277,
  1090,
  1541,
  274,
  319,
  276,
  1539,
  1538,
  1535,
  1534,
  1536,
  318,
  1532,
  1533,
];

static PRUNE_73: [usize; 19] = [
  2273,
  469,
  649,
  2274,
  646,
  648,
  644,
  2269,
  656,
  652,
  653,
  1710,
  1707,
  1165,
  1162,
  1161,
  1494,
  636,
  1712,
];

static PRUNE_104: [usize; 23] = [
  65,
  64,
  963,
  417,
  752,
  1245,
  2057,
  70,
  427,
  426,
  67,
  1800,
  1799,
  1796,
  756,
  2064,
  758,
  754,
  1269,
  1264,
  1806,
  1804,
  405,
];

static PRUNE_135: [usize; 16] = [
  695,
  1372,
  1906,
  1365,
  1380,
  1377,
  156,
  1210,
  851,
  1385,
  1000,
  861,
  1900,
  1896,
  1893,
  2027,
];

static PRUNE_25: [usize; 7] = [
  2005,
  987,
  937,
  1029,
  1026,
  349,
  948,
];

static PRUNE_90: [usize; 14] = [
  964,
  1770,
  437,
  31,
  27,
  20,
  428,
  22,
  712,
  33,
  2082,
  2079,
  722,
  720,
];

static PRUNE_152: [usize; 13] = [
  360,
  1957,
  917,
  1954,
  914,
  1427,
  202,
  221,
  1438,
  216,
  907,
  359,
  902,
];

static PRUNE_121: [usize; 13] = [
  1861,
  1276,
  2039,
  1338,
  1334,
  1333,
  396,
  803,
  123,
  1314,
  1855,
  1851,
  1848,
];

static PRUNE_11: [usize; 3] = [
  333,
  1557,
  1993,
];

static PRUNE_42: [usize; 14] = [
  1608,
  1066,
  1605,
  1064,
  1061,
  1620,
  534,
  1075,
  1610,
  2171,
  545,
  2168,
  2167,
  540,
];

static PRUNE_107: [usize; 10] = [
  1279,
  1275,
  2059,
  2060,
  1342,
  411,
  1807,
  769,
  1577,
  1263,
];

static PRUNE_138: [usize; 17] = [
  378,
  869,
  1383,
  161,
  382,
  176,
  175,
  864,
  863,
  865,
  1902,
  1177,
  862,
  858,
  855,
  856,
  946,
];

static PRUNE_59: [usize; 13] = [
  601,
  599,
  2222,
  461,
  478,
  1666,
  2114,
  1663,
  1119,
  594,
  1672,
  590,
  1531,
];

static PRUNE_28: [usize; 3] = [
  2008,
  1032,
  352,
];

static PRUNE_50: [usize; 20] = [
  2183,
  1642,
  1643,
  561,
  560,
  557,
  1636,
  1633,
  2194,
  570,
  2196,
  2190,
  566,
  1646,
  579,
  485,
  575,
  1631,
  2118,
  585,
];

static PRUNE_112: [usize; 16] = [
  1822,
  1823,
  2044,
  1819,
  778,
  1831,
  1288,
  789,
  2051,
  87,
  83,
  1573,
  1297,
  442,
  775,
  408,
];

static PRUNE_143: [usize; 13] = [
  876,
  370,
  868,
  1923,
  884,
  880,
  944,
  1166,
  889,
  187,
  364,
  184,
  178,
];

static PRUNE_19: [usize; 3] = [
  2000,
  344,
  1555,
];

static PRUNE_81: [usize; 16] = [
  1005,
  1743,
  1737,
  2280,
  2099,
  1736,
  969,
  448,
  2290,
  2287,
  6,
  683,
  5,
  1173,
  2,
  674,
];

static PRUNE_67: [usize; 17] = [
  1687,
  1142,
  1009,
  1685,
  1140,
  1698,
  1016,
  1015,
  628,
  2248,
  849,
  2245,
  1700,
  2243,
  1134,
  1676,
  631,
];

static PRUNE_36: [usize; 22] = [
  1052,
  1054,
  1049,
  510,
  2134,
  915,
  1048,
  2150,
  524,
  2148,
  1606,
  1559,
  1059,
  2157,
  535,
  502,
  504,
  1043,
  1042,
  1040,
  1036,
  2164,
];

static PRUNE_160: [usize; 15] = [
  1459,
  1455,
  239,
  255,
  1968,
  254,
  1965,
  928,
  246,
  1484,
  1979,
  1450,
  233,
  231,
  1448,
];

static PRUNE_98: [usize; 12] = [
  1233,
  423,
  1787,
  1783,
  1242,
  1780,
  1238,
  2072,
  1252,
  1793,
  1386,
  50,
];

static PRUNE_129: [usize; 9] = [
  149,
  1349,
  132,
  1883,
  1361,
  1357,
  1221,
  952,
  1354,
];

static PRUNE_5: [usize; 2] = [
  326,
  1981,
];

static PRUNE_146: [usize; 17] = [
  1416,
  1418,
  192,
  1410,
  194,
  2014,
  887,
  1423,
  1936,
  2022,
  1933,
  2021,
  1928,
  188,
  366,
  186,
  1983,
];

static PRUNE_53: [usize; 13] = [
  1641,
  1097,
  1095,
  1094,
  1093,
  1020,
  882,
  1017,
  487,
  1537,
  1088,
  1084,
  1085,
];

static PRUNE_22: [usize; 4] = [
  2003,
  2091,
  460,
  347,
];

static PRUNE_115: [usize; 15] = [
  739,
  105,
  1318,
  99,
  1292,
  1835,
  1287,
  790,
  787,
  1300,
  804,
  1295,
  1315,
  98,
  1306,
];

static PRUNE_84: [usize; 7] = [
  508,
  1745,
  1740,
  1741,
  1752,
  1749,
  1747,
];

static PRUNE_163: [usize; 26] = [
  244,
  1461,
  1460,
  1498,
  1499,
  2013,
  256,
  1560,
  1470,
  2012,
  358,
  357,
  267,
  263,
  264,
  1479,
  258,
  1973,
  1477,
  1972,
  1497,
  271,
  993,
  1491,
  1486,
  315,
];

static PRUNE_132: [usize; 20] = [
  153,
  380,
  151,
  150,
  152,
  145,
  1903,
  372,
  146,
  390,
  852,
  2024,
  1884,
  848,
  1360,
  143,
  1898,
  141,
  2031,
  1351,
];

static PRUNE_70: [usize; 16] = [
  465,
  642,
  1696,
  972,
  1513,
  1706,
  2249,
  1704,
  1703,
  2108,
  638,
  635,
  2260,
  2256,
  630,
  2254,
];

static PRUNE_101: [usize; 15] = [
  59,
  55,
  751,
  1789,
  1244,
  747,
  742,
  1801,
  1260,
  1253,
  1250,
  52,
  49,
  1353,
  1803,
];

static PRUNE_39: [usize; 5] = [
  1547,
  537,
  981,
  2128,
  498,
];

static PRUNE_13: [usize; 4] = [
  335,
  419,
  2055,
  2054,
];

static PRUNE_137: [usize; 16] = [
  1910,
  377,
  1908,
  375,
  1905,
  164,
  166,
  1384,
  159,
  160,
  383,
  854,
  2026,
  1391,
  171,
  168,
];

static PRUNE_106: [usize; 10] = [
  1274,
  2058,
  72,
  1331,
  1258,
  755,
  1271,
  1812,
  765,
  1262,
];

static PRUNE_75: [usize; 12] = [
  1008,
  1726,
  2266,
  661,
  2281,
  659,
  970,
  2096,
  1167,
  2107,
  1718,
  1714,
];

static PRUNE_44: [usize; 5] = [
  1546,
  493,
  549,
  2124,
  1623,
];

static PRUNE_30: [usize; 13] = [
  1552,
  1591,
  1588,
  926,
  2010,
  1033,
  1030,
  355,
  1028,
  640,
  1041,
  1038,
  629,
];

static PRUNE_154: [usize; 17] = [
  1959,
  1958,
  1955,
  919,
  1951,
  912,
  1002,
  1562,
  922,
  218,
  1435,
  354,
  941,
  1978,
  215,
  1431,
  1948,
];

static PRUNE_92: [usize; 19] = [
  17,
  1230,
  1771,
  1231,
  1229,
  29,
  28,
  794,
  23,
  1213,
  716,
  715,
  1209,
  1205,
  1765,
  1767,
  1225,
  1216,
  1397,
];

static PRUNE_61: [usize; 14] = [
  2093,
  606,
  605,
  2227,
  2224,
  2220,
  613,
  2234,
  2231,
  1121,
  2213,
  1674,
  1128,
  1126,
];

static PRUNE_123: [usize; 20] = [
  1866,
  1863,
  825,
  2041,
  120,
  116,
  113,
  403,
  402,
  129,
  395,
  126,
  124,
  818,
  953,
  820,
  1857,
  2034,
  133,
  811,
];

static PRUNE_109: [usize; 20] = [
  784,
  1282,
  961,
  780,
  1815,
  76,
  1289,
  1829,
  1284,
  1824,
  762,
  763,
  82,
  80,
  77,
  79,
  78,
  773,
  414,
  767,
];

static PRUNE_140: [usize; 17] = [
  376,
  1909,
  866,
  1922,
  1917,
  1915,
  1393,
  1395,
  1388,
  1568,
  1569,
  1389,
  2018,
  684,
  1401,
  859,
  1398,
];

static PRUNE_16: [usize; 3] = [
  1997,
  340,
  322,
];

static PRUNE_47: [usize; 20] = [
  555,
  2179,
  551,
  2176,
  1022,
  1019,
  568,
  2193,
  2188,
  1078,
  1077,
  1072,
  1073,
  893,
  1071,
  548,
  2172,
  1626,
  1080,
  1082,
];

static PRUNE_78: [usize; 15] = [
  1724,
  1473,
  655,
  2278,
  2277,
  1169,
  443,
  670,
  1164,
  668,
  1159,
  1720,
  455,
  1176,
  676,
];

static PRUNE_33: [usize; 10] = [
  651,
  2139,
  1549,
  1594,
  513,
  2143,
  2145,
  521,
  1554,
  518,
];

static PRUNE_126: [usize; 15] = [
  1864,
  1337,
  1878,
  1876,
  1873,
  386,
  1870,
  1871,
  1345,
  1347,
  1570,
  1341,
  2038,
  409,
  2035,
];

static PRUNE_95: [usize; 16] = [
  1775,
  735,
  2084,
  1790,
  1785,
  433,
  2067,
  440,
  36,
  439,
  1224,
  730,
  726,
  2075,
  1220,
  43,
];

static PRUNE_2: [usize; 2] = [
  332,
  307,
];

static PRUNE_64: [usize; 18] = [
  1682,
  1680,
  616,
  480,
  1519,
  481,
  1692,
  476,
  2230,
  608,
  1124,
  2112,
  625,
  621,
  1135,
  1131,
  1669,
  1671,
];

static PRUNE_157: [usize; 4] = [
  924,
  220,
  226,
  1442,
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
    run_level(&mut temp_nodes, &LEVEL_2);
    prune(&mut temp_nodes, &PRUNE_2);
    run_level(&mut temp_nodes, &LEVEL_3);
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
    run_level(&mut temp_nodes, &LEVEL_27);
    prune(&mut temp_nodes, &PRUNE_27);
    run_level(&mut temp_nodes, &LEVEL_28);
    prune(&mut temp_nodes, &PRUNE_28);
    run_level(&mut temp_nodes, &LEVEL_29);
    prune(&mut temp_nodes, &PRUNE_29);
    run_level(&mut temp_nodes, &LEVEL_30);
    prune(&mut temp_nodes, &PRUNE_30);
    run_level(&mut temp_nodes, &LEVEL_31);
    prune(&mut temp_nodes, &PRUNE_31);
    run_level(&mut temp_nodes, &LEVEL_32);
    prune(&mut temp_nodes, &PRUNE_32);
    run_level(&mut temp_nodes, &LEVEL_33);
    prune(&mut temp_nodes, &PRUNE_33);
    run_level(&mut temp_nodes, &LEVEL_34);
    prune(&mut temp_nodes, &PRUNE_34);
    run_level(&mut temp_nodes, &LEVEL_35);
    prune(&mut temp_nodes, &PRUNE_35);
    run_level(&mut temp_nodes, &LEVEL_36);
    prune(&mut temp_nodes, &PRUNE_36);
    run_level(&mut temp_nodes, &LEVEL_37);
    prune(&mut temp_nodes, &PRUNE_37);
    run_level(&mut temp_nodes, &LEVEL_38);
    prune(&mut temp_nodes, &PRUNE_38);
    run_level(&mut temp_nodes, &LEVEL_39);
    prune(&mut temp_nodes, &PRUNE_39);
    run_level(&mut temp_nodes, &LEVEL_40);
    prune(&mut temp_nodes, &PRUNE_40);
    run_level(&mut temp_nodes, &LEVEL_41);
    prune(&mut temp_nodes, &PRUNE_41);
    run_level(&mut temp_nodes, &LEVEL_42);
    prune(&mut temp_nodes, &PRUNE_42);
    run_level(&mut temp_nodes, &LEVEL_43);
    prune(&mut temp_nodes, &PRUNE_43);
    run_level(&mut temp_nodes, &LEVEL_44);
    prune(&mut temp_nodes, &PRUNE_44);
    run_level(&mut temp_nodes, &LEVEL_45);
    prune(&mut temp_nodes, &PRUNE_45);
    run_level(&mut temp_nodes, &LEVEL_46);
    prune(&mut temp_nodes, &PRUNE_46);
    run_level(&mut temp_nodes, &LEVEL_47);
    prune(&mut temp_nodes, &PRUNE_47);
    run_level(&mut temp_nodes, &LEVEL_48);
    prune(&mut temp_nodes, &PRUNE_48);
    run_level(&mut temp_nodes, &LEVEL_49);
    prune(&mut temp_nodes, &PRUNE_49);
    run_level(&mut temp_nodes, &LEVEL_50);
    prune(&mut temp_nodes, &PRUNE_50);
    run_level(&mut temp_nodes, &LEVEL_51);
    prune(&mut temp_nodes, &PRUNE_51);
    run_level(&mut temp_nodes, &LEVEL_52);
    prune(&mut temp_nodes, &PRUNE_52);
    run_level(&mut temp_nodes, &LEVEL_53);
    prune(&mut temp_nodes, &PRUNE_53);
    run_level(&mut temp_nodes, &LEVEL_54);
    prune(&mut temp_nodes, &PRUNE_54);
    run_level(&mut temp_nodes, &LEVEL_55);
    prune(&mut temp_nodes, &PRUNE_55);
    run_level(&mut temp_nodes, &LEVEL_56);
    prune(&mut temp_nodes, &PRUNE_56);
    run_level(&mut temp_nodes, &LEVEL_57);
    prune(&mut temp_nodes, &PRUNE_57);
    run_level(&mut temp_nodes, &LEVEL_58);
    prune(&mut temp_nodes, &PRUNE_58);
    run_level(&mut temp_nodes, &LEVEL_59);
    prune(&mut temp_nodes, &PRUNE_59);
    run_level(&mut temp_nodes, &LEVEL_60);
    prune(&mut temp_nodes, &PRUNE_60);
    run_level(&mut temp_nodes, &LEVEL_61);
    prune(&mut temp_nodes, &PRUNE_61);
    run_level(&mut temp_nodes, &LEVEL_62);
    prune(&mut temp_nodes, &PRUNE_62);
    run_level(&mut temp_nodes, &LEVEL_63);
    prune(&mut temp_nodes, &PRUNE_63);
    run_level(&mut temp_nodes, &LEVEL_64);
    prune(&mut temp_nodes, &PRUNE_64);
    run_level(&mut temp_nodes, &LEVEL_65);
    prune(&mut temp_nodes, &PRUNE_65);
    run_level(&mut temp_nodes, &LEVEL_66);
    prune(&mut temp_nodes, &PRUNE_66);
    run_level(&mut temp_nodes, &LEVEL_67);
    prune(&mut temp_nodes, &PRUNE_67);
    run_level(&mut temp_nodes, &LEVEL_68);
    prune(&mut temp_nodes, &PRUNE_68);
    run_level(&mut temp_nodes, &LEVEL_69);
    prune(&mut temp_nodes, &PRUNE_69);
    run_level(&mut temp_nodes, &LEVEL_70);
    prune(&mut temp_nodes, &PRUNE_70);
    run_level(&mut temp_nodes, &LEVEL_71);
    prune(&mut temp_nodes, &PRUNE_71);
    run_level(&mut temp_nodes, &LEVEL_72);
    prune(&mut temp_nodes, &PRUNE_72);
    run_level(&mut temp_nodes, &LEVEL_73);
    prune(&mut temp_nodes, &PRUNE_73);
    run_level(&mut temp_nodes, &LEVEL_74);
    prune(&mut temp_nodes, &PRUNE_74);
    run_level(&mut temp_nodes, &LEVEL_75);
    prune(&mut temp_nodes, &PRUNE_75);
    run_level(&mut temp_nodes, &LEVEL_76);
    prune(&mut temp_nodes, &PRUNE_76);
    run_level(&mut temp_nodes, &LEVEL_77);
    prune(&mut temp_nodes, &PRUNE_77);
    run_level(&mut temp_nodes, &LEVEL_78);
    prune(&mut temp_nodes, &PRUNE_78);
    run_level(&mut temp_nodes, &LEVEL_79);
    prune(&mut temp_nodes, &PRUNE_79);
    run_level(&mut temp_nodes, &LEVEL_80);
    prune(&mut temp_nodes, &PRUNE_80);
    run_level(&mut temp_nodes, &LEVEL_81);
    prune(&mut temp_nodes, &PRUNE_81);
    run_level(&mut temp_nodes, &LEVEL_82);
    prune(&mut temp_nodes, &PRUNE_82);
    run_level(&mut temp_nodes, &LEVEL_83);
    prune(&mut temp_nodes, &PRUNE_83);
    run_level(&mut temp_nodes, &LEVEL_84);
    prune(&mut temp_nodes, &PRUNE_84);
    run_level(&mut temp_nodes, &LEVEL_85);
    prune(&mut temp_nodes, &PRUNE_85);
    run_level(&mut temp_nodes, &LEVEL_86);
    prune(&mut temp_nodes, &PRUNE_86);
    run_level(&mut temp_nodes, &LEVEL_87);
    prune(&mut temp_nodes, &PRUNE_87);
    run_level(&mut temp_nodes, &LEVEL_88);
    prune(&mut temp_nodes, &PRUNE_88);
    run_level(&mut temp_nodes, &LEVEL_89);
    prune(&mut temp_nodes, &PRUNE_89);
    run_level(&mut temp_nodes, &LEVEL_90);
    prune(&mut temp_nodes, &PRUNE_90);
    run_level(&mut temp_nodes, &LEVEL_91);
    prune(&mut temp_nodes, &PRUNE_91);
    run_level(&mut temp_nodes, &LEVEL_92);
    prune(&mut temp_nodes, &PRUNE_92);
    run_level(&mut temp_nodes, &LEVEL_93);
    prune(&mut temp_nodes, &PRUNE_93);
    run_level(&mut temp_nodes, &LEVEL_94);
    prune(&mut temp_nodes, &PRUNE_94);
    run_level(&mut temp_nodes, &LEVEL_95);
    prune(&mut temp_nodes, &PRUNE_95);
    run_level(&mut temp_nodes, &LEVEL_96);
    prune(&mut temp_nodes, &PRUNE_96);
    run_level(&mut temp_nodes, &LEVEL_97);
    prune(&mut temp_nodes, &PRUNE_97);
    run_level(&mut temp_nodes, &LEVEL_98);
    prune(&mut temp_nodes, &PRUNE_98);
    run_level(&mut temp_nodes, &LEVEL_99);
    prune(&mut temp_nodes, &PRUNE_99);
    run_level(&mut temp_nodes, &LEVEL_100);
    prune(&mut temp_nodes, &PRUNE_100);
    run_level(&mut temp_nodes, &LEVEL_101);
    prune(&mut temp_nodes, &PRUNE_101);
    run_level(&mut temp_nodes, &LEVEL_102);
    prune(&mut temp_nodes, &PRUNE_102);
    run_level(&mut temp_nodes, &LEVEL_103);
    prune(&mut temp_nodes, &PRUNE_103);
    run_level(&mut temp_nodes, &LEVEL_104);
    prune(&mut temp_nodes, &PRUNE_104);
    run_level(&mut temp_nodes, &LEVEL_105);
    prune(&mut temp_nodes, &PRUNE_105);
    run_level(&mut temp_nodes, &LEVEL_106);
    prune(&mut temp_nodes, &PRUNE_106);
    run_level(&mut temp_nodes, &LEVEL_107);
    prune(&mut temp_nodes, &PRUNE_107);
    run_level(&mut temp_nodes, &LEVEL_108);
    prune(&mut temp_nodes, &PRUNE_108);
    run_level(&mut temp_nodes, &LEVEL_109);
    prune(&mut temp_nodes, &PRUNE_109);
    run_level(&mut temp_nodes, &LEVEL_110);
    prune(&mut temp_nodes, &PRUNE_110);
    run_level(&mut temp_nodes, &LEVEL_111);
    prune(&mut temp_nodes, &PRUNE_111);
    run_level(&mut temp_nodes, &LEVEL_112);
    prune(&mut temp_nodes, &PRUNE_112);
    run_level(&mut temp_nodes, &LEVEL_113);
    prune(&mut temp_nodes, &PRUNE_113);
    run_level(&mut temp_nodes, &LEVEL_114);
    prune(&mut temp_nodes, &PRUNE_114);
    run_level(&mut temp_nodes, &LEVEL_115);
    prune(&mut temp_nodes, &PRUNE_115);
    run_level(&mut temp_nodes, &LEVEL_116);
    prune(&mut temp_nodes, &PRUNE_116);
    run_level(&mut temp_nodes, &LEVEL_117);
    prune(&mut temp_nodes, &PRUNE_117);
    run_level(&mut temp_nodes, &LEVEL_118);
    prune(&mut temp_nodes, &PRUNE_118);
    run_level(&mut temp_nodes, &LEVEL_119);
    prune(&mut temp_nodes, &PRUNE_119);
    run_level(&mut temp_nodes, &LEVEL_120);
    prune(&mut temp_nodes, &PRUNE_120);
    run_level(&mut temp_nodes, &LEVEL_121);
    prune(&mut temp_nodes, &PRUNE_121);
    run_level(&mut temp_nodes, &LEVEL_122);
    prune(&mut temp_nodes, &PRUNE_122);
    run_level(&mut temp_nodes, &LEVEL_123);
    prune(&mut temp_nodes, &PRUNE_123);
    run_level(&mut temp_nodes, &LEVEL_124);
    prune(&mut temp_nodes, &PRUNE_124);
    run_level(&mut temp_nodes, &LEVEL_125);
    prune(&mut temp_nodes, &PRUNE_125);
    run_level(&mut temp_nodes, &LEVEL_126);
    prune(&mut temp_nodes, &PRUNE_126);
    run_level(&mut temp_nodes, &LEVEL_127);
    prune(&mut temp_nodes, &PRUNE_127);
    run_level(&mut temp_nodes, &LEVEL_128);
    prune(&mut temp_nodes, &PRUNE_128);
    run_level(&mut temp_nodes, &LEVEL_129);
    prune(&mut temp_nodes, &PRUNE_129);
    run_level(&mut temp_nodes, &LEVEL_130);
    prune(&mut temp_nodes, &PRUNE_130);
    run_level(&mut temp_nodes, &LEVEL_131);
    prune(&mut temp_nodes, &PRUNE_131);
    run_level(&mut temp_nodes, &LEVEL_132);
    prune(&mut temp_nodes, &PRUNE_132);
    run_level(&mut temp_nodes, &LEVEL_133);
    prune(&mut temp_nodes, &PRUNE_133);
    run_level(&mut temp_nodes, &LEVEL_134);
    prune(&mut temp_nodes, &PRUNE_134);
    run_level(&mut temp_nodes, &LEVEL_135);
    prune(&mut temp_nodes, &PRUNE_135);
    run_level(&mut temp_nodes, &LEVEL_136);
    prune(&mut temp_nodes, &PRUNE_136);
    run_level(&mut temp_nodes, &LEVEL_137);
    prune(&mut temp_nodes, &PRUNE_137);
    run_level(&mut temp_nodes, &LEVEL_138);
    prune(&mut temp_nodes, &PRUNE_138);
    run_level(&mut temp_nodes, &LEVEL_139);
    prune(&mut temp_nodes, &PRUNE_139);
    run_level(&mut temp_nodes, &LEVEL_140);
    prune(&mut temp_nodes, &PRUNE_140);
    run_level(&mut temp_nodes, &LEVEL_141);
    prune(&mut temp_nodes, &PRUNE_141);
    run_level(&mut temp_nodes, &LEVEL_142);
    prune(&mut temp_nodes, &PRUNE_142);
    run_level(&mut temp_nodes, &LEVEL_143);
    prune(&mut temp_nodes, &PRUNE_143);
    run_level(&mut temp_nodes, &LEVEL_144);
    prune(&mut temp_nodes, &PRUNE_144);
    run_level(&mut temp_nodes, &LEVEL_145);
    prune(&mut temp_nodes, &PRUNE_145);
    run_level(&mut temp_nodes, &LEVEL_146);
    prune(&mut temp_nodes, &PRUNE_146);
    run_level(&mut temp_nodes, &LEVEL_147);
    prune(&mut temp_nodes, &PRUNE_147);
    run_level(&mut temp_nodes, &LEVEL_148);
    prune(&mut temp_nodes, &PRUNE_148);
    run_level(&mut temp_nodes, &LEVEL_149);
    prune(&mut temp_nodes, &PRUNE_149);
    run_level(&mut temp_nodes, &LEVEL_150);
    prune(&mut temp_nodes, &PRUNE_150);
    run_level(&mut temp_nodes, &LEVEL_151);
    prune(&mut temp_nodes, &PRUNE_151);
    run_level(&mut temp_nodes, &LEVEL_152);
    prune(&mut temp_nodes, &PRUNE_152);
    run_level(&mut temp_nodes, &LEVEL_153);
    prune(&mut temp_nodes, &PRUNE_153);
    run_level(&mut temp_nodes, &LEVEL_154);
    prune(&mut temp_nodes, &PRUNE_154);
    run_level(&mut temp_nodes, &LEVEL_155);
    prune(&mut temp_nodes, &PRUNE_155);
    run_level(&mut temp_nodes, &LEVEL_156);
    prune(&mut temp_nodes, &PRUNE_156);
    run_level(&mut temp_nodes, &LEVEL_157);
    prune(&mut temp_nodes, &PRUNE_157);
    run_level(&mut temp_nodes, &LEVEL_158);
    prune(&mut temp_nodes, &PRUNE_158);
    run_level(&mut temp_nodes, &LEVEL_159);
    prune(&mut temp_nodes, &PRUNE_159);
    run_level(&mut temp_nodes, &LEVEL_160);
    prune(&mut temp_nodes, &PRUNE_160);
    run_level(&mut temp_nodes, &LEVEL_161);
    prune(&mut temp_nodes, &PRUNE_161);
    run_level(&mut temp_nodes, &LEVEL_162);
    prune(&mut temp_nodes, &PRUNE_162);
    run_level(&mut temp_nodes, &LEVEL_163);
    prune(&mut temp_nodes, &PRUNE_163);
    run_level(&mut temp_nodes, &LEVEL_164);
    prune(&mut temp_nodes, &PRUNE_164);
    run_level(&mut temp_nodes, &LEVEL_165);
    prune(&mut temp_nodes, &PRUNE_165);
    run_level(&mut temp_nodes, &LEVEL_166);
    prune(&mut temp_nodes, &PRUNE_166);

            

                out.into_iter().map(|c| c.unwrap()).collect()
            }),
        )
        .unwrap()
}

