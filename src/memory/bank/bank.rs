use strum_macros::Display;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub enum ReadStatusRegisterId {
    RS0,
    RS1,
    RS2,
    RS3,
    RS4,
    RS5,
    RS6,
    RS7,
    RS8,
    RS9,
    RS10,
    RS11,
    RS12,
    RS13,
    RS14,
    RS15,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub enum WriteStatusRegisterId {
    WS0,
    WS1,
    WS2,
    WS3,
    WS4,
    WS5,
    WS6,
    WS7,
    WS8,
    WS9,
    WS10,
    WS11,
    WS12,
    WS13,
    WS14,
    WS15,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PrgSourceRegisterId {
    PS0,
    PS1,
    PS2,
    PS3,
    PS4,
    PS5,
    PS6,
    PS7,
    PS8,
    PS9,
    PS10,
    PS11,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ChrSourceRegisterId {
    CS0,
    CS1,
    CS2,
    CS3,
    CS4,
    CS5,
    CS6,
    CS7,

    // Name Table Top Left
    NTS0,
    // Name Table Top Right
    NTS1,
    // Name Table Bottom Left
    NTS2,
    // Name Table Bottom Right
    NTS3,
}

impl ChrSourceRegisterId {
    pub const ALL_NAME_TABLE_SOURCE_IDS: [Self; 4] = [
        ChrSourceRegisterId::NTS0,
        ChrSourceRegisterId::NTS1,
        ChrSourceRegisterId::NTS2,
        ChrSourceRegisterId::NTS3,
    ];
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Display)]
pub enum MemoryPresence {
    Absent,
    Supported,
    Required,
}