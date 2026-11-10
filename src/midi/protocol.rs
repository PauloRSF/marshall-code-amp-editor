use super::types::{
    CabinetType, DelayType, ModulationType, PedalType, PowerAmpType, PreAmpType, Preset, ReverbType,
};

pub const HEADER: [u8; 7] = [0xF0, 0x00, 0x21, 0x15, 0x7F, 0x7F, 0x7F];
/// Working PC editors + marshall-code-api: 0x73 = current edit buffer, 0x72 = preset slot.
pub const CMD_CURRENT: u8 = 0x73;
pub const CMD_PRESET: u8 = 0x72;
pub const ACTION_RECALL: u8 = 0x01;
pub const ACTION_SET: u8 = 0x02;
pub const ACTION_DATA: u8 = 0x03;
pub const ACTION_OK: u8 = 0x04;
/// Trailing fixed bytes used by the shipping CODE editor (not the PDF's 01..04).
pub const PRESET_TAIL: [u8; 6] = [0x0C, 0x0D, 0x1A, 0x1B, 0x1C, 0x1D];

macro_rules! enum_codec {
    ($ty:ty, $($variant:ident => $code:expr),+ $(,)?) => {
        impl $ty {
            pub fn from_code(code: u8) -> Option<Self> {
                match code {
                    $($code => Some(Self::$variant),)+
                    _ => None,
                }
            }

            pub fn to_code(self) -> u8 {
                match self {
                    $(Self::$variant => $code,)+
                }
            }
        }
    };
}

enum_codec!(
    PedalType,
    Compressor => 0,
    Distortion => 1,
    AutoWah => 2,
    PitchShifter => 3,
);

enum_codec!(
    PreAmpType,
    Jtm45 => 0,
    ClDsl => 1,
    ClAmerican => 2,
    ClJvm => 3,
    Acoustic => 4,
    Bluesbreaker => 5,
    Plexi => 6,
    CrAmerican => 7,
    Jcm800 => 8,
    British50s => 9,
    OdJvm => 10,
    OdDsl => 11,
    OdAmerican => 12,
    OdSilverJubilee => 13,
    Neutral => 14,
);

enum_codec!(
    ModulationType,
    Chorus => 0,
    Flanger => 1,
    Phaser => 2,
    Tremolo => 3,
);

enum_codec!(
    DelayType,
    Studio => 0,
    Vintage => 1,
    Multi => 2,
    Reverse => 3,
);

enum_codec!(
    ReverbType,
    Room => 0,
    Hall => 1,
    Spring => 2,
    Stadium => 3,
);

enum_codec!(
    PowerAmpType,
    ClassicMarshall100w => 0,
    VintageMarshall30w => 1,
    BritishClassA => 2,
    AmericanClassAB => 3,
);

enum_codec!(
    CabinetType,
    Cab1960 => 0,
    Cab1960V => 1,
    Cab1960X => 2,
    Cab1960Hw => 3,
    Cab1936 => 4,
    Cab1936V => 5,
    Cab1912 => 6,
    Cab1974Cx => 7,
);

fn clamp100(v: u8) -> u8 {
    v.min(100)
}

fn read_name(data: &[u8]) -> String {
    data.iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn preset_from_sysex(data: &[u8]) -> Option<Preset> {
    // Need through resonance at index 67; replies may be 73 (PDF) or 75 (editor) bytes.
    if data.len() < 68 || data[0] != 0xF0 {
        return None;
    }
    Some(Preset {
        number: data[9],
        name: read_name(&data[10..28]),
        gain: data[29],
        bass: data[30],
        middle: data[31],
        treble: data[32],
        volume: data[33],
        pedal_enabled: data[34] == 1,
        pedal_type: PedalType::from_code(data[35])?,
        pedal_param1: data[36],
        pedal_param2: data[37],
        pedal_param3: data[38],
        pedal_param4: data[39],
        preamp_enabled: data[40] == 1,
        preamp_type: PreAmpType::from_code(data[41])?,
        gate: data[42],
        modulation_enabled: data[43] == 1,
        modulation_type: ModulationType::from_code(data[44])?,
        modulation_param1: data[45],
        modulation_param2: data[46],
        modulation_param3: data[47],
        modulation_param4: data[48],
        delay_enabled: data[49] == 1,
        delay_type: DelayType::from_code(data[50])?,
        delay_time_msb: data[51],
        delay_time_lsb: data[52],
        delay_param2: data[53],
        delay_param3: data[54],
        delay_param4: data[55],
        reverb_enabled: data[56] == 1,
        reverb_type: ReverbType::from_code(data[57])?,
        reverb_param1: data[58],
        reverb_param2: data[59],
        reverb_param3: data[60],
        reverb_param4: data[61],
        poweramp_enabled: data[62] == 1,
        poweramp_type: PowerAmpType::from_code(data[63])?,
        cabinet_enabled: data[64] == 1,
        cabinet_type: CabinetType::from_code(data[65])?,
        presence: data[66],
        resonance: data[67],
    })
}

pub fn encode_preset_data(preset: &Preset) -> Vec<u8> {
    let mut name = preset.name.chars().take(18).collect::<String>();
    while name.len() < 18 {
        name.push(' ');
    }
    let mut bytes: Vec<u8> = name.bytes().map(|b| b & 0x7f).collect();
    bytes.extend_from_slice(&[
        0x00,
        clamp100(preset.gain),
        clamp100(preset.bass),
        clamp100(preset.middle),
        clamp100(preset.treble),
        clamp100(preset.volume),
        u8::from(preset.pedal_enabled),
        preset.pedal_type.to_code(),
        clamp100(preset.pedal_param1),
        clamp100(preset.pedal_param2),
        clamp100(preset.pedal_param3),
        clamp100(preset.pedal_param4),
        u8::from(preset.preamp_enabled),
        preset.preamp_type.to_code(),
        clamp100(preset.gate),
        u8::from(preset.modulation_enabled),
        preset.modulation_type.to_code(),
        clamp100(preset.modulation_param1),
        clamp100(preset.modulation_param2),
        clamp100(preset.modulation_param3),
        clamp100(preset.modulation_param4),
        u8::from(preset.delay_enabled),
        preset.delay_type.to_code(),
        preset.delay_time_msb.min(31),
        preset.delay_time_lsb.min(127),
        clamp100(preset.delay_param2),
        clamp100(preset.delay_param3),
        clamp100(preset.delay_param4),
        u8::from(preset.reverb_enabled),
        preset.reverb_type.to_code(),
        clamp100(preset.reverb_param1),
        clamp100(preset.reverb_param2),
        clamp100(preset.reverb_param3),
        clamp100(preset.reverb_param4),
        u8::from(preset.poweramp_enabled),
        preset.poweramp_type.to_code(),
        u8::from(preset.cabinet_enabled),
        preset.cabinet_type.to_code(),
        clamp100(preset.presence),
        clamp100(preset.resonance),
    ]);
    bytes.extend_from_slice(&PRESET_TAIL);
    bytes
}

pub fn sysex_recall_current() -> Vec<u8> {
    let mut msg = HEADER.to_vec();
    msg.extend_from_slice(&[CMD_CURRENT, ACTION_RECALL, 0x00, 0xF7]);
    msg
}

pub fn sysex_recall_preset(slot: u8) -> Vec<u8> {
    let mut msg = HEADER.to_vec();
    msg.extend_from_slice(&[CMD_PRESET, ACTION_RECALL, slot.min(99), 0xF7]);
    msg
}

pub fn sysex_set_current(preset: &Preset) -> Vec<u8> {
    let mut msg = HEADER.to_vec();
    msg.extend_from_slice(&[CMD_CURRENT, ACTION_SET, 0x00]);
    msg.extend(encode_preset_data(preset));
    msg.push(0xF7);
    msg
}

pub fn sysex_set_preset(preset: &Preset, slot: u8) -> Vec<u8> {
    let mut msg = HEADER.to_vec();
    msg.extend_from_slice(&[CMD_PRESET, ACTION_SET, slot.min(99)]);
    msg.extend(encode_preset_data(preset));
    msg.push(0xF7);
    msg
}

pub fn cc(controller: u8, value: u8) -> [u8; 3] {
    [0xB0, controller, value.min(127)]
}

pub fn program_change(slot: u8) -> [u8; 2] {
    [0xC0, slot.min(99)]
}

#[derive(Debug, Clone)]
pub enum Incoming {
    PresetNumber(u8),
    CurrentPreset(Preset),
    StoredPreset(Preset),
    Cc { controller: u8, value: u8 },
}

pub fn parse_incoming(data: &[u8]) -> Option<Incoming> {
    if data.is_empty() {
        return None;
    }
    match data[0] {
        0xB0 if data.len() >= 3 => Some(Incoming::Cc {
            controller: data[1],
            value: data[2],
        }),
        0xC0 if data.len() >= 2 => Some(Incoming::PresetNumber(data[1])),
        0xF0 => {
            if data.len() < 10 || data[1..4] != [0x00, 0x21, 0x15] {
                return None;
            }
            // Amp replies use action 0x03; ignore bare ACK/error frames.
            match data.get(8).copied() {
                Some(ACTION_DATA) => {}
                Some(ACTION_OK) | Some(0x05) => return None,
                // Some stacks deliver SET/RECALL echoes; still try to parse payload.
                Some(ACTION_SET) | Some(ACTION_RECALL) => {}
                _ => return None,
            }
            let preset = preset_from_sysex(data)?;
            match data[7] {
                CMD_PRESET => Some(Incoming::StoredPreset(preset)),
                CMD_CURRENT => Some(Incoming::CurrentPreset(preset)),
                // PDF labels are swapped vs firmware; accept either as a preset dump.
                _ => Some(Incoming::CurrentPreset(preset)),
            }
        }
        _ => None,
    }
}

pub fn apply_cc(preset: &mut Preset, controller: u8, value: u8) {
    match controller {
        31 => preset.delay_time_msb = value,
        63 => preset.delay_time_lsb = value,
        70 => preset.gain = value,
        71 => preset.bass = value,
        72 => preset.middle = value,
        73 => preset.treble = value,
        74 => preset.volume = value,
        75 => preset.pedal_enabled = value == 1,
        76 => {
            if let Some(t) = PedalType::from_code(value) {
                preset.pedal_type = t;
                preset.pedal_param1 = 0;
                preset.pedal_param2 = 0;
                preset.pedal_param3 = 0;
                preset.pedal_param4 = 0;
            }
        }
        77 => preset.pedal_param1 = value,
        78 => preset.pedal_param2 = value,
        79 => preset.pedal_param3 = value,
        80 => preset.pedal_param4 = value,
        81 => preset.preamp_enabled = value == 1,
        82 => {
            if let Some(t) = PreAmpType::from_code(value) {
                preset.preamp_type = t;
            }
        }
        83 => preset.gate = value,
        85 => preset.modulation_enabled = value == 1,
        86 => {
            if let Some(t) = ModulationType::from_code(value) {
                preset.modulation_type = t;
            }
        }
        87 => preset.modulation_param1 = value,
        89 => preset.modulation_param2 = value,
        90 => preset.modulation_param3 = value,
        102 => preset.modulation_param4 = value,
        103 => preset.delay_enabled = value == 1,
        104 => {
            if let Some(t) = DelayType::from_code(value) {
                preset.delay_type = t;
            }
        }
        105 => preset.delay_param2 = value,
        106 => preset.delay_param3 = value,
        107 => preset.delay_param4 = value,
        108 => preset.reverb_enabled = value == 1,
        109 => {
            if let Some(t) = ReverbType::from_code(value) {
                preset.reverb_type = t;
            }
        }
        110 => preset.reverb_param1 = value,
        111 => preset.reverb_param2 = value,
        112 => preset.reverb_param3 = value,
        113 => preset.reverb_param4 = value,
        114 => preset.poweramp_enabled = value == 1,
        115 => {
            if let Some(t) = PowerAmpType::from_code(value) {
                preset.poweramp_type = t;
            }
        }
        116 => preset.cabinet_enabled = value == 1,
        117 => {
            if let Some(t) = CabinetType::from_code(value) {
                preset.cabinet_type = t;
            }
        }
        118 => preset.presence = value,
        119 => preset.resonance = value,
        _ => {}
    }
}

/// Live-edit helpers: send MIDI CC for a field change.
pub fn live_messages(before: &Preset, after: &Preset) -> Vec<[u8; 3]> {
    let mut out = Vec::new();
    macro_rules! diff_cc {
        ($ctrl:expr, $field:ident) => {
            if before.$field != after.$field {
                out.push(cc($ctrl, after.$field));
            }
        };
    }
    macro_rules! diff_bool {
        ($ctrl:expr, $field:ident) => {
            if before.$field != after.$field {
                out.push(cc($ctrl, u8::from(after.$field)));
            }
        };
    }
    macro_rules! diff_enum {
        ($ctrl:expr, $field:ident) => {
            if before.$field != after.$field {
                out.push(cc($ctrl, after.$field.to_code()));
            }
        };
    }

    diff_cc!(31, delay_time_msb);
    diff_cc!(63, delay_time_lsb);
    diff_cc!(70, gain);
    diff_cc!(71, bass);
    diff_cc!(72, middle);
    diff_cc!(73, treble);
    diff_cc!(74, volume);
    diff_bool!(75, pedal_enabled);
    diff_enum!(76, pedal_type);
    diff_cc!(77, pedal_param1);
    diff_cc!(78, pedal_param2);
    diff_cc!(79, pedal_param3);
    diff_cc!(80, pedal_param4);
    diff_bool!(81, preamp_enabled);
    diff_enum!(82, preamp_type);
    diff_cc!(83, gate);
    diff_bool!(85, modulation_enabled);
    diff_enum!(86, modulation_type);
    diff_cc!(87, modulation_param1);
    diff_cc!(89, modulation_param2);
    diff_cc!(90, modulation_param3);
    diff_cc!(102, modulation_param4);
    diff_bool!(103, delay_enabled);
    diff_enum!(104, delay_type);
    diff_cc!(105, delay_param2);
    diff_cc!(106, delay_param3);
    diff_cc!(107, delay_param4);
    diff_bool!(108, reverb_enabled);
    diff_enum!(109, reverb_type);
    diff_cc!(110, reverb_param1);
    diff_cc!(111, reverb_param2);
    diff_cc!(112, reverb_param3);
    diff_cc!(113, reverb_param4);
    diff_bool!(114, poweramp_enabled);
    diff_enum!(115, poweramp_type);
    diff_bool!(116, cabinet_enabled);
    diff_enum!(117, cabinet_type);
    diff_cc!(118, presence);
    diff_cc!(119, resonance);
    out
}
