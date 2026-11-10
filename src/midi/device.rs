use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

use super::protocol::{
    self, apply_cc, live_messages, parse_incoming, program_change, Incoming,
};
use super::types::Preset;

#[derive(Debug, Clone)]
pub struct PortInfo {
    pub id: usize,
    pub name: String,
}

/// Reassemble SysEx that Windows MIDI Services may deliver in 6-byte WinMM chunks.
struct SysexAssembler {
    buf: Vec<u8>,
}

impl SysexAssembler {
    fn new() -> Self {
        Self { buf: Vec::new() }
    }

    fn push(&mut self, message: &[u8]) -> Vec<Vec<u8>> {
        let mut complete = Vec::new();
        if message.is_empty() {
            return complete;
        }

        // Fresh non-sysex short message.
        if message[0] != 0xF0 && self.buf.is_empty() {
            complete.push(message.to_vec());
            return complete;
        }

        if message[0] == 0xF0 {
            self.buf.clear();
        } else if self.buf.is_empty() {
            // Stray continuation without a start — ignore.
            return complete;
        }

        self.buf.extend_from_slice(message);
        if self.buf.last() == Some(&0xF7) {
            complete.push(std::mem::take(&mut self.buf));
        } else if self.buf.len() > 512 {
            // Give up on a runaway fragment stream.
            self.buf.clear();
        }
        complete
    }
}

pub struct MidiDevice {
    _input: Option<MidiInputConnection<()>>,
    output: Option<MidiOutputConnection>,
    rx: Receiver<Incoming>,
    tx: Sender<Incoming>,
    connected_name: Option<String>,
}

impl MidiDevice {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            _input: None,
            output: None,
            rx,
            tx,
            connected_name: None,
        }
    }

    pub fn connected_name(&self) -> Option<&str> {
        self.connected_name.as_deref()
    }

    pub fn list_outputs() -> Result<Vec<PortInfo>, String> {
        let output = MidiOutput::new("CODE Editor Out Scan")
            .map_err(|e| format!("MIDI output init failed: {e}"))?;
        Ok(output
            .ports()
            .into_iter()
            .enumerate()
            .filter_map(|(id, port)| {
                output
                    .port_name(&port)
                    .ok()
                    .map(|name| PortInfo { id, name })
            })
            .collect())
    }

    pub fn preferred_output_index(ports: &[PortInfo]) -> Option<usize> {
        ports
            .iter()
            .position(|p| is_code_name(&p.name))
            .or_else(|| {
                ports.iter().position(|p| {
                    let n = p.name.to_uppercase();
                    !n.contains("WAVETABLE") && !n.contains("MICROSOFT GS")
                })
            })
            .or_else(|| ports.first().map(|p| p.id))
    }

    pub fn connect(&mut self, output_index: usize) -> Result<(), String> {
        self.disconnect();

        let midi_out = MidiOutput::new("CODE Editor Out")
            .map_err(|e| format!("MIDI output init failed: {e}"))?;
        let out_ports = midi_out.ports();
        let out_port = out_ports
            .get(output_index)
            .ok_or_else(|| "Selected MIDI output not found".to_string())?;
        let out_name = midi_out
            .port_name(out_port)
            .unwrap_or_else(|_| "CODE".into());

        if looks_like_software_synth(&out_name) {
            return Err(format!(
                "“{out_name}” is a software synth, not the Marshall CODE. Pick the CODE MIDI port."
            ));
        }

        let mut midi_in = MidiInput::new("CODE Editor In")
            .map_err(|e| format!("MIDI input init failed: {e}"))?;
        midi_in.ignore(Ignore::None);
        let in_ports = midi_in.ports();
        let in_port = in_ports
            .iter()
            .find(|p| {
                midi_in
                    .port_name(p)
                    .map(|n| n == out_name || is_code_name(&n))
                    .unwrap_or(false)
            })
            .or_else(|| in_ports.first())
            .ok_or_else(|| "No MIDI input port available".to_string())?;

        let tx = self.tx.clone();
        let assembler = Arc::new(Mutex::new(SysexAssembler::new()));
        let input_conn = midi_in
            .connect(
                in_port,
                "code-in",
                move |_stamp, message, _| {
                    let messages = {
                        let Ok(mut asm) = assembler.lock() else {
                            return;
                        };
                        asm.push(message)
                    };
                    for msg in messages {
                        if let Some(event) = parse_incoming(&msg) {
                            let _ = tx.send(event);
                        }
                    }
                },
                (),
            )
            .map_err(|e| format!("Failed to open MIDI input: {e}"))?;

        let output_conn = midi_out
            .connect(out_port, "code-out")
            .map_err(|e| format!("Failed to open MIDI output: {e}"))?;

        self._input = Some(input_conn);
        self.output = Some(output_conn);
        self.connected_name = Some(out_name);
        Ok(())
    }

    pub fn disconnect(&mut self) {
        self._input = None;
        self.output = None;
        self.connected_name = None;
    }

    pub fn poll(&mut self) -> Vec<Incoming> {
        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            events.push(event);
        }
        events
    }

    pub fn send(&mut self, bytes: &[u8]) -> Result<(), String> {
        let out = self
            .output
            .as_mut()
            .ok_or_else(|| "Not connected".to_string())?;
        out.send(bytes)
            .map_err(|e| format!("MIDI send failed: {e}"))
    }

    pub fn request_current(&mut self) -> Result<(), String> {
        self.send(&protocol::sysex_recall_current())
    }

    pub fn request_preset(&mut self, slot: u8) -> Result<(), String> {
        self.send(&protocol::sysex_recall_preset(slot))
    }

    pub fn switch_preset(&mut self, slot: u8) -> Result<(), String> {
        self.send(&program_change(slot))
    }

    pub fn save_to_amp(&mut self, preset: &Preset, slot: u8) -> Result<(), String> {
        if let Some(name) = &self.connected_name {
            if looks_like_software_synth(name) {
                return Err("Connected to a software synth, not the CODE amp.".into());
            }
        }

        // 1) Push every live CC so the edit buffer matches (works even if SysEx is flaky).
        let blank = Preset::default();
        for msg in live_messages(&blank, preset) {
            self.send(&msg)?;
            thread::sleep(Duration::from_millis(2));
        }
        thread::sleep(Duration::from_millis(40));

        // 2) Write edit buffer (0x73) then store into slot (0x72) — matching working PC editors.
        self.send(&protocol::sysex_set_current(preset))?;
        thread::sleep(Duration::from_millis(60));
        self.send(&protocol::sysex_set_preset(preset, slot))?;
        thread::sleep(Duration::from_millis(80));

        // 3) Program-change to the slot so the amp reloads what was just stored.
        self.send(&program_change(slot))?;
        thread::sleep(Duration::from_millis(40));

        Ok(())
    }

    pub fn push_live(&mut self, before: &Preset, after: &Preset) -> Result<(), String> {
        for msg in live_messages(before, after) {
            self.send(&msg)?;
        }
        Ok(())
    }

    pub fn apply_incoming(preset: &mut Preset, event: &Incoming) -> bool {
        match event {
            Incoming::CurrentPreset(p) | Incoming::StoredPreset(p) => {
                *preset = p.clone();
                true
            }
            Incoming::PresetNumber(n) => {
                preset.number = *n;
                true
            }
            Incoming::Cc { controller, value } => {
                apply_cc(preset, *controller, *value);
                true
            }
        }
    }
}

pub fn is_code_name(name: &str) -> bool {
    let upper = name.to_uppercase();
    upper.contains("CODE") || upper.contains("MARSHALL") || upper.contains("SOFTUBE")
}

fn looks_like_software_synth(name: &str) -> bool {
    let upper = name.to_uppercase();
    upper.contains("WAVETABLE")
        || upper.contains("MICROSOFT GS")
        || upper.contains("LOOPBACK")
        || upper.contains("VIRTUAL")
}
