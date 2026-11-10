use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PedalType {
    Compressor,
    Distortion,
    AutoWah,
    PitchShifter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PreAmpType {
    Jtm45,
    ClDsl,
    ClAmerican,
    ClJvm,
    Acoustic,
    Bluesbreaker,
    Plexi,
    CrAmerican,
    Jcm800,
    #[serde(rename = "50S_BRITISH")]
    British50s,
    OdJvm,
    OdDsl,
    OdAmerican,
    OdSilverJubilee,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModulationType {
    Chorus,
    Flanger,
    Phaser,
    Tremolo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DelayType {
    Studio,
    Vintage,
    Multi,
    Reverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReverbType {
    Room,
    Hall,
    Spring,
    Stadium,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PowerAmpType {
    ClassicMarshall100w,
    VintageMarshall30w,
    BritishClassA,
    AmericanClassAB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CabinetType {
    #[serde(rename = "1960")]
    Cab1960,
    #[serde(rename = "1960V")]
    Cab1960V,
    #[serde(rename = "1960X")]
    Cab1960X,
    #[serde(rename = "1960HW")]
    Cab1960Hw,
    #[serde(rename = "1936")]
    Cab1936,
    #[serde(rename = "1936V")]
    Cab1936V,
    #[serde(rename = "1912")]
    Cab1912,
    #[serde(rename = "1974CX")]
    Cab1974Cx,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preset {
    pub number: u8,
    pub name: String,
    pub gain: u8,
    pub bass: u8,
    pub middle: u8,
    pub treble: u8,
    pub volume: u8,
    pub pedal_enabled: bool,
    pub pedal_type: PedalType,
    pub pedal_param1: u8,
    pub pedal_param2: u8,
    pub pedal_param3: u8,
    pub pedal_param4: u8,
    pub preamp_enabled: bool,
    pub preamp_type: PreAmpType,
    pub gate: u8,
    pub modulation_enabled: bool,
    pub modulation_type: ModulationType,
    pub modulation_param1: u8,
    pub modulation_param2: u8,
    pub modulation_param3: u8,
    pub modulation_param4: u8,
    pub delay_enabled: bool,
    pub delay_type: DelayType,
    pub delay_time_msb: u8,
    pub delay_time_lsb: u8,
    pub delay_param2: u8,
    pub delay_param3: u8,
    pub delay_param4: u8,
    pub reverb_enabled: bool,
    pub reverb_type: ReverbType,
    pub reverb_param1: u8,
    pub reverb_param2: u8,
    pub reverb_param3: u8,
    pub reverb_param4: u8,
    pub poweramp_enabled: bool,
    pub poweramp_type: PowerAmpType,
    pub cabinet_enabled: bool,
    pub cabinet_type: CabinetType,
    pub presence: u8,
    pub resonance: u8,
}

impl Default for Preset {
    fn default() -> Self {
        Self {
            number: 0,
            name: "INIT PATCH".into(),
            gain: 50,
            bass: 50,
            middle: 50,
            treble: 50,
            volume: 50,
            pedal_enabled: false,
            pedal_type: PedalType::Compressor,
            pedal_param1: 0,
            pedal_param2: 50,
            pedal_param3: 50,
            pedal_param4: 50,
            preamp_enabled: true,
            preamp_type: PreAmpType::Jtm45,
            gate: 0,
            modulation_enabled: false,
            modulation_type: ModulationType::Chorus,
            modulation_param1: 0,
            modulation_param2: 50,
            modulation_param3: 50,
            modulation_param4: 50,
            delay_enabled: false,
            delay_type: DelayType::Studio,
            delay_time_msb: 0,
            delay_time_lsb: 0,
            delay_param2: 50,
            delay_param3: 50,
            delay_param4: 50,
            reverb_enabled: false,
            reverb_type: ReverbType::Room,
            reverb_param1: 50,
            reverb_param2: 50,
            reverb_param3: 50,
            reverb_param4: 50,
            poweramp_enabled: true,
            poweramp_type: PowerAmpType::ClassicMarshall100w,
            cabinet_enabled: true,
            cabinet_type: CabinetType::Cab1960,
            presence: 50,
            resonance: 50,
        }
    }
}

impl Preset {
    pub fn delay_ms(&self) -> u16 {
        (((self.delay_time_msb as u16) << 7) | self.delay_time_lsb as u16).min(4000)
    }

    pub fn set_delay_ms(&mut self, ms: u16) {
        let clamped = ms.min(4000);
        self.delay_time_msb = ((clamped >> 7) & 0x1f) as u8;
        self.delay_time_lsb = (clamped & 0x7f) as u8;
    }
}

pub const PREAMP_TYPES: &[PreAmpType] = &[
    PreAmpType::Jtm45,
    PreAmpType::ClDsl,
    PreAmpType::ClAmerican,
    PreAmpType::ClJvm,
    PreAmpType::Acoustic,
    PreAmpType::Bluesbreaker,
    PreAmpType::Plexi,
    PreAmpType::CrAmerican,
    PreAmpType::Jcm800,
    PreAmpType::British50s,
    PreAmpType::OdJvm,
    PreAmpType::OdDsl,
    PreAmpType::OdAmerican,
    PreAmpType::OdSilverJubilee,
    PreAmpType::Neutral,
];

pub const PEDAL_TYPES: &[PedalType] = &[
    PedalType::Compressor,
    PedalType::Distortion,
    PedalType::AutoWah,
    PedalType::PitchShifter,
];

pub const MOD_TYPES: &[ModulationType] = &[
    ModulationType::Chorus,
    ModulationType::Flanger,
    ModulationType::Phaser,
    ModulationType::Tremolo,
];

pub const DELAY_TYPES: &[DelayType] = &[
    DelayType::Studio,
    DelayType::Vintage,
    DelayType::Multi,
    DelayType::Reverse,
];

pub const REVERB_TYPES: &[ReverbType] = &[
    ReverbType::Room,
    ReverbType::Hall,
    ReverbType::Spring,
    ReverbType::Stadium,
];

pub const POWERAMP_TYPES: &[PowerAmpType] = &[
    PowerAmpType::ClassicMarshall100w,
    PowerAmpType::VintageMarshall30w,
    PowerAmpType::BritishClassA,
    PowerAmpType::AmericanClassAB,
];

pub const CABINET_TYPES: &[CabinetType] = &[
    CabinetType::Cab1960,
    CabinetType::Cab1960V,
    CabinetType::Cab1960X,
    CabinetType::Cab1960Hw,
    CabinetType::Cab1936,
    CabinetType::Cab1936V,
    CabinetType::Cab1912,
    CabinetType::Cab1974Cx,
];

pub fn preamp_label(t: PreAmpType) -> &'static str {
    match t {
        PreAmpType::Jtm45 => "JTM45",
        PreAmpType::ClDsl => "Clean DSL",
        PreAmpType::ClAmerican => "Clean American",
        PreAmpType::ClJvm => "Clean JVM",
        PreAmpType::Acoustic => "Acoustic",
        PreAmpType::Bluesbreaker => "Bluesbreaker",
        PreAmpType::Plexi => "Plexi",
        PreAmpType::CrAmerican => "Crunch American",
        PreAmpType::Jcm800 => "JCM800",
        PreAmpType::British50s => "50s British",
        PreAmpType::OdJvm => "OD JVM",
        PreAmpType::OdDsl => "OD DSL",
        PreAmpType::OdAmerican => "OD American",
        PreAmpType::OdSilverJubilee => "OD Silver Jubilee",
        PreAmpType::Neutral => "Natural",
    }
}

pub fn pedal_label(t: PedalType) -> &'static str {
    match t {
        PedalType::Compressor => "Compressor",
        PedalType::Distortion => "Distortion",
        PedalType::AutoWah => "Auto Wah",
        PedalType::PitchShifter => "Pitch Shifter",
    }
}

pub fn mod_label(t: ModulationType) -> &'static str {
    match t {
        ModulationType::Chorus => "Chorus",
        ModulationType::Flanger => "Flanger",
        ModulationType::Phaser => "Phaser",
        ModulationType::Tremolo => "Tremolo",
    }
}

pub fn delay_label(t: DelayType) -> &'static str {
    match t {
        DelayType::Studio => "Studio",
        DelayType::Vintage => "Vintage",
        DelayType::Multi => "Multi",
        DelayType::Reverse => "Reverse",
    }
}

pub fn reverb_label(t: ReverbType) -> &'static str {
    match t {
        ReverbType::Room => "Room",
        ReverbType::Hall => "Hall",
        ReverbType::Spring => "Spring",
        ReverbType::Stadium => "Stadium",
    }
}

pub fn poweramp_label(t: PowerAmpType) -> &'static str {
    match t {
        PowerAmpType::ClassicMarshall100w => "Classic Marshall 100W",
        PowerAmpType::VintageMarshall30w => "Vintage Marshall 30W",
        PowerAmpType::BritishClassA => "British Class A",
        PowerAmpType::AmericanClassAB => "American Class A/B",
    }
}

pub fn cabinet_label(t: CabinetType) -> &'static str {
    match t {
        CabinetType::Cab1960 => "1960",
        CabinetType::Cab1960V => "1960V",
        CabinetType::Cab1960X => "1960X",
        CabinetType::Cab1960Hw => "1960HW",
        CabinetType::Cab1936 => "1936",
        CabinetType::Cab1936V => "1936V",
        CabinetType::Cab1912 => "1912",
        CabinetType::Cab1974Cx => "1974CX",
    }
}
