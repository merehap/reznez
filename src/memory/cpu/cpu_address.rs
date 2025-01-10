use std::fmt;
use std::str::FromStr;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct CpuAddress(u16);

impl CpuAddress {
    pub const ZERO: CpuAddress = CpuAddress::new(0x0000);

    pub const fn new(value: u16) -> CpuAddress {
        CpuAddress(value)
    }

    pub fn from_low_high(low: u8, high: u8) -> CpuAddress {
        CpuAddress::new(((u16::from(high)) << 8) + (u16::from(low)))
    }

    pub fn zero_page(low: u8) -> CpuAddress {
        CpuAddress::new(u16::from(low))
    }

    pub const fn to_raw(self) -> u16 {
        self.0
    }

    pub const fn to_u32(self) -> u32 {
        self.0 as u32
    }

    pub fn to_usize(self) -> usize {
        usize::from(self.0)
    }

    pub fn to_low_high(self) -> (u8, u8) {
        (self.0 as u8, (self.0 >> 8) as u8)
    }

    pub fn to_mesen_string(self) -> String {
        let basic_string = &format!("${:04X}", self.0);
        match self.0 {
            0x2000 => "PpuControl_2000",
            0x2001 => "PpuMask_2001",
            0x2002 => "PpuStatus_2002",
            0x2003 => "OamAddr_2003",
            0x2004 => "OamData_2004",
            0x2005 => "PpuScroll_2005",
            0x2006 => "PpuAddr_2006",
            0x2007 => "PpuData_2007",
            0x4000 => "Sq0Duty_4000",
            0x4001 => "Sq0Sweep_4001",
            0x4002 => "Sq0Timer_4002",
            0x4003 => "Sq0Length_4003",
            0x4004 => "Sq1Duty_4004",
            0x4005 => "Sq1Sweep_4005",
            0x4006 => "Sq1Timer_4006",
            0x4007 => "Sq1Length_4007",
            0x4008 => "TrgLinear_4008",
            // 0x4009 is unused.
            0x400A => "TrgTimer_400A",
            0x400B => "TrgLength_400B",
            0x400C => "NoiseVolume_400C",
            // 0x400D is unused.
            0x400E => "NoisePeriod_400E",
            0x400F => "NoiseLength_400F",
            0x4010 => "DmcFreq_4010",
            0x4011 => "DmcCounter_4011",
            0x4012 => "DmcAddress_4012",
            0x4013 => "DmcLength_4013",
            0x4014 => "SpriteDma_4014",
            0x4015 => "ApuStatus_4015",
            0x4016 => "Ctrl1_4016",
            0x4017 => "Ctrl2_FrameCtr_4017",
            _ => basic_string,
        }.to_owned()
    }

    pub fn low_byte(self) -> u8 {
        u8::try_from(self.0 & 0x00FF).unwrap()
    }

    pub fn high_byte(self) -> u8 {
        u8::try_from(self.0 >> 8).unwrap()
    }

    pub fn advance(self, value: u8) -> CpuAddress {
        CpuAddress::new(self.0.wrapping_add(u16::from(value)))
    }

    pub fn offset(self, value: i8) -> CpuAddress {
        CpuAddress::new((i32::from(self.0)).wrapping_add(i32::from(value)) as u16)
    }

    pub fn offset_with_carry(&mut self, value: i8) -> (CpuAddress, i8) {
        let temp = self.offset(value);
        let carry = (i16::from(temp.high_byte()) - i16::from(self.high_byte())) as i8;
        *self = CpuAddress::from_low_high(temp.low_byte(), self.high_byte());
        (*self, carry)
    }

    pub fn offset_low(&mut self, value: u8) -> (CpuAddress, bool) {
        let (low, high) = self.to_low_high();
        let (low, carry) = low.overflowing_add(value);
        *self = CpuAddress::from_low_high(low, high);
        (*self, carry)
    }

    pub fn offset_high(&mut self, value: i8) -> CpuAddress {
        let (low, high) = self.to_low_high();
        let high = high.wrapping_add_signed(value);
        *self = CpuAddress::from_low_high(low, high);
        *self
    }

    pub fn inc(&mut self) -> CpuAddress {
        self.0 = self.0.wrapping_add(1);
        *self
    }

    pub fn page(self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn index_within_page(self) -> u8 {
        self.0 as u8
    }

    pub fn is_end_of_page(self) -> bool {
        self.index_within_page() == 0xFF
    }

    pub fn is_odd(self) -> bool {
        self.0 % 2 == 1
    }
}

impl fmt::Display for CpuAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:04X}", self.0)
    }
}

impl FromStr for CpuAddress {
    type Err = String;

    fn from_str(value: &str) -> Result<CpuAddress, String> {
        let raw = u16::from_str(value).map_err(|err| err.to_string())?;
        Ok(CpuAddress(raw))
    }
}
