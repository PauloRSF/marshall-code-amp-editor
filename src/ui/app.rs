use std::time::{Duration, Instant};

use eframe::egui::{self, Align, Color32, Layout, RichText, Ui, Vec2};
use rfd::FileDialog;

use crate::midi::{
    cabinet_label, delay_label, is_code_name, mod_label, pedal_label, poweramp_label, preamp_label,
    reverb_label, Incoming, MidiDevice, ModulationType, PedalType, PortInfo, Preset,
    CABINET_TYPES, DELAY_TYPES, MOD_TYPES, PEDAL_TYPES, POWERAMP_TYPES, PREAMP_TYPES, REVERB_TYPES,
};

use super::theme::{self, DANGER, DANGER_HOVER, GOLD, GOLD_BRIGHT, INK_DARK, MUTED, OK};

/// Design resolution — window resize scales the whole UI uniformly.
pub const DESIGN_W: f32 = 960.0;
pub const DESIGN_H: f32 = 820.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToastKind {
    Error,
    Success,
    Info,
}

struct Toast {
    message: String,
    kind: ToastKind,
    until: Instant,
}

pub struct CodeEditorApp {
    midi: MidiDevice,
    ports: Vec<PortInfo>,
    selected_port: usize,
    preset: Preset,
    save_slot: u8,
    status: String,
    dirty: bool,
    connected: bool,
    connecting: bool,
    /// When connect began — used to stop waiting for a SysEx reply that never arrives.
    connect_started: Option<Instant>,
    /// Skip pushing live CCs for a few frames after amp→app updates.
    suppress_live_until: Option<Instant>,
    toast: Option<Toast>,
}

impl Default for CodeEditorApp {
    fn default() -> Self {
        let ports = MidiDevice::list_outputs().unwrap_or_default();
        let selected_port = MidiDevice::preferred_output_index(&ports).unwrap_or(0);
        Self {
            midi: MidiDevice::new(),
            ports,
            selected_port,
            preset: Preset::default(),
            save_slot: 0,
            status: "Power the CODE on, plug in USB, then connect.".into(),
            dirty: false,
            connected: false,
            connecting: false,
            connect_started: None,
            suppress_live_until: None,
            toast: None,
        }
    }
}

impl eframe::App for CodeEditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        theme::apply(ui.ctx());
        self.apply_uniform_zoom(ui.ctx());
        self.poll_midi();
        self.expire_toast();

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::symmetric(28, 20)),
            )
            .show(ui, |ui| {
                self.header(ui);
                ui.add_space(16.0);
                self.toolbar(ui);
                ui.add_space(16.0);

                let before = self.preset.clone();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.signal_chain(ui);
                        ui.add_space(12.0);
                    });

                let suppress = self
                    .suppress_live_until
                    .is_some_and(|t| Instant::now() < t);
                if self.connected && !suppress && before != self.preset {
                    let mut a = before.clone();
                    let mut b = self.preset.clone();
                    a.name.clear();
                    b.name.clear();
                    a.number = 0;
                    b.number = 0;
                    if a != b {
                        if let Err(e) = self.midi.push_live(&before, &self.preset) {
                            self.show_toast(ToastKind::Error, e.clone());
                            self.status = e;
                        } else {
                            self.dirty = true;
                            if let Some(msg) = chain_toggle_status(&before, &self.preset) {
                                self.status = msg;
                            }
                        }
                    }
                }

                ui.add_space(8.0);
                self.footer(ui);
                self.draw_toast(ui);
            });

        ui.ctx()
            .request_repaint_after(Duration::from_millis(50));
    }
}

impl CodeEditorApp {
    fn apply_uniform_zoom(&self, ctx: &egui::Context) {
        // Derive zoom from physical pixels so it stays stable across frames.
        let screen = ctx.content_rect().size();
        let ppp = ctx.pixels_per_point();
        let native = ctx.native_pixels_per_point().unwrap_or(1.0).max(0.5);
        let pixel_w = screen.x * ppp;
        let pixel_h = screen.y * ppp;
        let zoom = ((pixel_w / DESIGN_W).min(pixel_h / DESIGN_H) / native).clamp(0.75, 2.5);
        if (ctx.zoom_factor() - zoom).abs() > 0.01 {
            ctx.set_zoom_factor(zoom);
        }
    }

    fn poll_midi(&mut self) {
        let mut touched_by_amp = false;
        for event in self.midi.poll() {
            match &event {
                Incoming::CurrentPreset(p) => {
                    self.preset = p.clone();
                    self.save_slot = p.number;
                    self.dirty = false;
                    self.connecting = false;
                    self.connect_started = None;
                    touched_by_amp = true;
                    self.status = format!("Synced from amp · slot {:02} · {}", p.number, p.name);
                }
                Incoming::StoredPreset(p) => {
                    self.preset = p.clone();
                    self.save_slot = p.number;
                    self.dirty = false;
                    self.connecting = false;
                    self.connect_started = None;
                    touched_by_amp = true;
                    self.status = format!("Loaded preset {:02} · {}", p.number, p.name);
                }
                Incoming::PresetNumber(n) => {
                    // Amp preset selector changed — pull the active edit buffer.
                    self.save_slot = *n;
                    self.status = format!("Amp switched to preset {n:02}. Syncing…");
                    touched_by_amp = true;
                    let _ = self.midi.request_current();
                    let _ = self.midi.request_preset(*n);
                }
                Incoming::Cc { .. } => {
                    MidiDevice::apply_incoming(&mut self.preset, &event);
                    self.dirty = true;
                    touched_by_amp = true;
                }
            }
        }
        if touched_by_amp {
            self.suppress_live_until = Some(Instant::now() + Duration::from_millis(200));
        }

        // Windows MIDI Services often never delivers CODE SysEx replies. Don't hang Connecting.
        if self.connecting {
            let timed_out = self
                .connect_started
                .is_some_and(|t| t.elapsed() > Duration::from_millis(1500));
            if timed_out {
                self.connecting = false;
                self.connect_started = None;
                self.status =
                    "Connected. Amp did not reply over USB MIDI — live edit & save still available."
                        .into();
                self.show_toast(
                    ToastKind::Info,
                    "Connected without sync — live edit & save still work.",
                );
            }
        }
    }

    fn header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(theme::brand("CODE EDITOR"));
                ui.label(theme::subtitle("Marshall CODE amp control"));
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if self.connected {
                    if danger_button(ui, "Disconnect").clicked() {
                        self.midi.disconnect();
                        self.connected = false;
                        self.connecting = false;
                        self.connect_started = None;
                        self.status = "Disconnected.".into();
                        self.show_toast(ToastKind::Info, "Disconnected from amp.");
                    }
                    if ui.button("Reload").clicked() {
                        match self.midi.request_current() {
                            Ok(()) => self.status = "Refreshing current settings…".into(),
                            Err(e) => {
                                self.show_toast(ToastKind::Error, e.clone());
                                self.status = e;
                            }
                        }
                    }
                    ui.label(
                        RichText::new(self.midi.connected_name().unwrap_or("CODE"))
                            .color(GOLD)
                            .size(13.0),
                    );
                } else {
                    let label = if self.connecting {
                        "Connecting…"
                    } else {
                        "Connect Amp"
                    };
                    let response = gold_button(ui, label);
                    if response.clicked() && !self.connecting {
                        self.connect();
                    }
                }
            });
        });
    }

    fn show_toast(&mut self, kind: ToastKind, message: impl Into<String>) {
        self.toast = Some(Toast {
            message: message.into(),
            kind,
            until: Instant::now() + Duration::from_secs(4),
        });
    }

    fn expire_toast(&mut self) {
        if self
            .toast
            .as_ref()
            .is_some_and(|t| Instant::now() >= t.until)
        {
            self.toast = None;
        }
    }

    fn draw_toast(&self, ui: &mut Ui) {
        let Some(toast) = &self.toast else {
            return;
        };
        let (fill, border) = match toast.kind {
            ToastKind::Error => (Color32::from_rgb(48, 22, 18), DANGER),
            ToastKind::Success => (Color32::from_rgb(22, 40, 22), OK),
            ToastKind::Info => (Color32::from_rgb(36, 30, 18), GOLD),
        };

        let max_width = (ui.max_rect().width() - 48.0).clamp(220.0, 480.0);
        #[allow(deprecated)]
        let toast_area = egui::Area::new(egui::Id::new("code_toast"));
        toast_area
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -20.0))
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(fill)
                    .stroke(egui::Stroke::new(1.5, border))
                    .corner_radius(10.0)
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.set_max_width(max_width);
                        ui.label(
                            RichText::new(&toast.message)
                                .color(theme::CREAM)
                                .size(14.0),
                        );
                    });
            });
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        theme::toolbar_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                labeled(ui, "MIDI PORT", |ui| {
                    let labels: Vec<String> = if self.ports.is_empty() {
                        vec!["No MIDI outputs found".into()]
                    } else {
                        self.ports.iter().map(|p| p.name.clone()).collect()
                    };
                    let mut idx = self.selected_port.min(labels.len().saturating_sub(1));
                    egui::ComboBox::from_id_salt("midi_port")
                        .selected_text(labels.get(idx).cloned().unwrap_or_default())
                        .width(220.0)
                        .show_ui(ui, |ui| {
                            for (i, label) in labels.iter().enumerate() {
                                ui.selectable_value(&mut idx, i, label);
                            }
                        });
                    self.selected_port = idx;

                    if ui
                        .add(egui::Button::new("⟳").min_size(Vec2::splat(28.0)))
                        .on_hover_text("Refresh ports")
                        .clicked()
                    {
                        self.refresh_ports();
                    }
                });

                labeled(ui, "PATCH NAME", |ui| {
                    let mut name = self.preset.name.clone();
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut name)
                                .desired_width(160.0)
                                .char_limit(18)
                                .hint_text("PRESET NAME"),
                        )
                        .changed()
                    {
                        self.preset.name = name.to_uppercase();
                        self.dirty = true;
                    }
                });

                labeled(ui, "PRESET", |ui| {
                    let connected = self.connected;
                    if ui
                        .add_enabled(connected, egui::Button::new("◀").min_size(Vec2::splat(28.0)))
                        .on_hover_text("Previous preset on amp")
                        .clicked()
                    {
                        let slot = self.save_slot.saturating_sub(1);
                        self.select_preset_on_amp(slot);
                    }

                    let mut slot = self.save_slot as i32;
                    let slot_resp = ui.add(
                        egui::DragValue::new(&mut slot)
                            .range(0..=99)
                            .speed(0.35)
                            .min_decimals(0),
                    );
                    if slot_resp.changed() {
                        self.save_slot = slot as u8;
                        // Send as soon as the value settles (drag end or typed commit).
                        if connected && (slot_resp.drag_stopped() || !slot_resp.dragged()) {
                            self.select_preset_on_amp(self.save_slot);
                        }
                    } else if connected && slot_resp.drag_stopped() {
                        self.select_preset_on_amp(self.save_slot);
                    }

                    if ui
                        .add_enabled(connected, egui::Button::new("▶").min_size(Vec2::splat(28.0)))
                        .on_hover_text("Next preset on amp")
                        .clicked()
                    {
                        let slot = (self.save_slot + 1).min(99);
                        self.select_preset_on_amp(slot);
                    }
                });

                ui.add_space(4.0);
                let enabled = self.connected;
                if ui
                    .add_enabled(enabled, egui::Button::new("Recall"))
                    .on_hover_text("Ask the amp for this preset over SysEx (needs MIDI in)")
                    .clicked()
                {
                    self.load_slot(self.save_slot);
                }
                if ui
                    .add_enabled_ui(enabled, |ui| gold_button(ui, "Save to Amp"))
                    .inner
                    .clicked()
                {
                    self.save_to_amp();
                }
                if ui.button("Export JSON").clicked() {
                    self.export_json();
                }
                if ui.button("Import JSON").clicked() {
                    self.import_json();
                }
            });
        });
    }

    fn signal_chain(&mut self, ui: &mut Ui) {
        // 01 Pre Effects
        chain_block(ui, "01", "PRE EFFECTS", &mut self.preset.pedal_enabled, |ui| {
            controls_wrap(ui, |ui| {
                field_combo(
                    ui,
                    "01-type",
                    "TYPE",
                    &mut self.preset.pedal_type,
                    PEDAL_TYPES,
                    pedal_label,
                );
                match self.preset.pedal_type {
                    PedalType::Distortion => {
                        mode_field(
                            ui,
                            "01-mode",
                            "MODE",
                            &mut self.preset.pedal_param1,
                            &["Guv'nor", "Overdrive", "Distortion"],
                        );
                        gold_slider(ui, "DRIVE", &mut self.preset.pedal_param2, 0, 100);
                        gold_slider(ui, "TONE", &mut self.preset.pedal_param3, 0, 100);
                        gold_slider(ui, "LEVEL", &mut self.preset.pedal_param4, 0, 100);
                    }
                    PedalType::AutoWah => {
                        mode_field(
                            ui,
                            "01-mode",
                            "MODE",
                            &mut self.preset.pedal_param1,
                            &["Envelope", "LFO"],
                        );
                        gold_slider(ui, "FREQ", &mut self.preset.pedal_param2, 0, 100);
                        gold_slider(ui, "SENS", &mut self.preset.pedal_param3, 0, 100);
                        gold_slider(ui, "RES", &mut self.preset.pedal_param4, 0, 100);
                    }
                    PedalType::PitchShifter => {
                        gold_slider(ui, "SEMI", &mut self.preset.pedal_param1, 0, 24);
                        gold_slider(ui, "FINE", &mut self.preset.pedal_param2, 0, 100);
                        gold_slider(ui, "REGEN", &mut self.preset.pedal_param3, 0, 100);
                        gold_slider(ui, "MIX", &mut self.preset.pedal_param4, 0, 100);
                    }
                    PedalType::Compressor => {
                        gold_slider(ui, "TONE", &mut self.preset.pedal_param1, 0, 100);
                        gold_slider(ui, "RATIO", &mut self.preset.pedal_param2, 0, 100);
                        gold_slider(ui, "COMP", &mut self.preset.pedal_param3, 0, 100);
                        gold_slider(ui, "LEVEL", &mut self.preset.pedal_param4, 0, 100);
                    }
                }
            });
        });

        // 02 Amplifier
        chain_block(ui, "02", "AMPLIFIER", &mut self.preset.preamp_enabled, |ui| {
            controls_wrap(ui, |ui| {
                field_combo(
                    ui,
                    "02-model",
                    "MODEL",
                    &mut self.preset.preamp_type,
                    PREAMP_TYPES,
                    preamp_label,
                );
                gold_slider(ui, "GAIN", &mut self.preset.gain, 0, 100);
                gold_slider(ui, "BASS", &mut self.preset.bass, 0, 100);
                gold_slider(ui, "MIDDLE", &mut self.preset.middle, 0, 100);
                gold_slider(ui, "TREBLE", &mut self.preset.treble, 0, 100);
                gold_slider(ui, "VOLUME", &mut self.preset.volume, 0, 100);
                gold_slider(ui, "GATE", &mut self.preset.gate, 0, 100);
            });
        });

        // 03 Modulation
        chain_block(
            ui,
            "03",
            "MODULATION",
            &mut self.preset.modulation_enabled,
            |ui| {
                controls_wrap(ui, |ui| {
                    field_combo(
                        ui,
                        "03-type",
                        "TYPE",
                        &mut self.preset.modulation_type,
                        MOD_TYPES,
                        mod_label,
                    );
                    let modes: &[&str] = match self.preset.modulation_type {
                        ModulationType::Chorus => &["Classic", "Vibrato"],
                        ModulationType::Flanger => &["Jet", "Metric"],
                        ModulationType::Phaser => &["Classic", "Vibes"],
                        ModulationType::Tremolo => &["Valve", "Square"],
                    };
                    mode_field(
                        ui,
                        "03-mode",
                        "MODE",
                        &mut self.preset.modulation_param1,
                        modes,
                    );
                    gold_slider(ui, "SPEED", &mut self.preset.modulation_param2, 0, 100);
                    gold_slider(ui, "DEPTH", &mut self.preset.modulation_param3, 0, 100);
                    gold_slider(
                        ui,
                        if self.preset.modulation_type == ModulationType::Flanger {
                            "REGEN"
                        } else {
                            "LEVEL"
                        },
                        &mut self.preset.modulation_param4,
                        0,
                        100,
                    );
                });
            },
        );

        // 04 Delay
        {
            let preset = &mut self.preset;
            let mut ms = preset.delay_ms();
            chain_block(ui, "04", "DELAY", &mut preset.delay_enabled, |ui| {
                controls_wrap(ui, |ui| {
                    field_combo(
                        ui,
                        "04-type",
                        "TYPE",
                        &mut preset.delay_type,
                        DELAY_TYPES,
                        delay_label,
                    );
                    delay_time_slider(ui, &mut ms);
                    gold_slider(ui, "FEEDBACK", &mut preset.delay_param2, 0, 100);
                    gold_slider(ui, "FREQ", &mut preset.delay_param3, 0, 100);
                    gold_slider(ui, "LEVEL", &mut preset.delay_param4, 0, 100);
                });
            });
            preset.set_delay_ms(ms);
        }

        // 05 Reverb
        chain_block(ui, "05", "REVERB", &mut self.preset.reverb_enabled, |ui| {
            controls_wrap(ui, |ui| {
                field_combo(
                    ui,
                    "05-type",
                    "TYPE",
                    &mut self.preset.reverb_type,
                    REVERB_TYPES,
                    reverb_label,
                );
                gold_slider(ui, "DECAY", &mut self.preset.reverb_param1, 0, 100);
                gold_slider(ui, "PRE-DLY", &mut self.preset.reverb_param2, 0, 100);
                gold_slider(ui, "TONE", &mut self.preset.reverb_param3, 0, 100);
                gold_slider(ui, "LEVEL", &mut self.preset.reverb_param4, 0, 100);
            });
        });

        // 06 Power Amp
        chain_block(
            ui,
            "06",
            "POWER AMP",
            &mut self.preset.poweramp_enabled,
            |ui| {
                controls_wrap(ui, |ui| {
                    field_combo(
                        ui,
                        "06-model",
                        "MODEL",
                        &mut self.preset.poweramp_type,
                        POWERAMP_TYPES,
                        poweramp_label,
                    );
                    gold_slider(ui, "PRESENCE", &mut self.preset.presence, 0, 100);
                    gold_slider(ui, "RESONANCE", &mut self.preset.resonance, 0, 100);
                });
            },
        );

        // 07 Cabinet
        chain_block(
            ui,
            "07",
            "CABINET",
            &mut self.preset.cabinet_enabled,
            |ui| {
                controls_wrap(ui, |ui| {
                    field_combo(
                        ui,
                        "07-cab",
                        "CAB",
                        &mut self.preset.cabinet_type,
                        CABINET_TYPES,
                        cabinet_label,
                    );
                });
            },
        );
    }

    fn footer(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("MIDI preset editor").color(MUTED).size(12.0));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(RichText::new("+ New patch").color(GOLD).size(13.0))
                            .frame(false),
                    )
                    .clicked()
                {
                    self.preset = Preset::default();
                    self.dirty = true;
                    self.status = "New patch ready — save to write it to the amp.".into();
                }

                ui.add_space(18.0);
                let state = if self.dirty {
                    format!("Patch {:02} · Unsaved changes", self.save_slot)
                } else {
                    format!("Patch {:02} · {}", self.save_slot, truncate(&self.status, 42))
                };
                ui.label(RichText::new(state).color(MUTED).size(12.0));
            });
        });
    }

    fn refresh_ports(&mut self) {
        match MidiDevice::list_outputs() {
            Ok(ports) => {
                self.ports = ports;
                self.selected_port =
                    MidiDevice::preferred_output_index(&self.ports).unwrap_or(0);
                self.status = if self.ports.is_empty() {
                    "No MIDI outputs found.".into()
                } else {
                    format!("Found {} MIDI output(s).", self.ports.len())
                };
            }
            Err(e) => self.status = e,
        }
    }

    fn connect(&mut self) {
        self.connecting = true;
        self.refresh_ports();
        if self.ports.is_empty() {
            self.connecting = false;
            let msg =
                "No MIDI outputs. Power the CODE on and plug in USB, then try again.".to_string();
            self.status = msg.clone();
            self.show_toast(ToastKind::Error, msg);
            return;
        }
        let index = self.selected_port.min(self.ports.len() - 1);
        if let Some(port) = self.ports.get(index) {
            if !is_code_name(&port.name) {
                // Still allow connect, but warn clearly.
                self.show_toast(
                    ToastKind::Info,
                    format!("Selected “{}” — prefer a port named CODE.", port.name),
                );
            }
        }
        match self.midi.connect(index) {
            Ok(()) => {
                self.connected = true;
                self.connect_started = Some(Instant::now());
                self.status = format!(
                    "Connected to {}. Loading…",
                    self.midi.connected_name().unwrap_or("CODE")
                );
                if let Err(e) = self.midi.request_current() {
                    self.connecting = false;
                    self.connect_started = None;
                    self.show_toast(ToastKind::Error, e.clone());
                    self.status = e;
                } else {
                    let _ = self.midi.request_preset(self.save_slot);
                    // Keep "Connecting…" until SysEx reply or timeout in poll_midi.
                    self.suppress_live_until =
                        Some(Instant::now() + Duration::from_millis(400));
                }
            }
            Err(e) => {
                self.connected = false;
                self.connecting = false;
                self.connect_started = None;
                self.status = e.clone();
                self.show_toast(ToastKind::Error, e);
            }
        }
    }

    /// Program-change the amp to `slot` (PC over USB MIDI).
    fn select_preset_on_amp(&mut self, slot: u8) {
        self.save_slot = slot;
        self.preset.number = slot;
        if !self.connected {
            return;
        }
        match self.midi.switch_preset(slot) {
            Ok(()) => {
                self.suppress_live_until = Some(Instant::now() + Duration::from_millis(250));
                self.status = format!("Amp preset → {slot:02}");
                // Best-effort pull; ignored if Windows drops CODE SysEx replies.
                let _ = self.midi.request_current();
                let _ = self.midi.request_preset(slot);
            }
            Err(e) => {
                self.show_toast(ToastKind::Error, e.clone());
                self.status = e;
            }
        }
    }

    fn load_slot(&mut self, slot: u8) {
        self.select_preset_on_amp(slot);
        if !self.connected {
            return;
        }
        self.suppress_live_until = Some(Instant::now() + Duration::from_millis(400));
        match self
            .midi
            .request_current()
            .and_then(|_| self.midi.request_preset(slot))
        {
            Ok(()) => self.status = format!("Recalling preset {slot:02}…"),
            Err(e) => {
                self.show_toast(ToastKind::Error, e.clone());
                self.status = e;
            }
        }
    }

    fn save_to_amp(&mut self) {
        self.preset.number = self.save_slot;
        self.preset.name = self
            .preset
            .name
            .chars()
            .filter(|c| c.is_ascii())
            .take(18)
            .collect();
        match self.midi.save_to_amp(&self.preset, self.save_slot) {
            Ok(()) => {
                self.dirty = false;
                self.suppress_live_until = Some(Instant::now() + Duration::from_millis(500));
                let msg = format!(
                    "Sent “{}” to amp preset {:02}",
                    self.preset.name, self.save_slot
                );
                self.status = msg.clone();
                self.show_toast(ToastKind::Success, msg);
                // Best-effort re-read (may no-op if Windows drops CODE SysEx replies).
                let _ = self.midi.request_preset(self.save_slot);
                let _ = self.midi.request_current();
            }
            Err(e) => {
                self.status = e.clone();
                self.show_toast(ToastKind::Error, e);
            }
        }
    }

    fn export_json(&mut self) {
        let Some(path) = FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name(format!("{}.json", sanitize(&self.preset.name)))
            .save_file()
        else {
            return;
        };
        match serde_json::to_string_pretty(&self.preset) {
            Ok(text) => match std::fs::write(&path, text) {
                Ok(()) => self.status = format!("Exported {}", path.display()),
                Err(e) => self.status = format!("Export failed: {e}"),
            },
            Err(e) => self.status = format!("Serialize failed: {e}"),
        }
    }

    fn import_json(&mut self) {
        let Some(path) = FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        else {
            return;
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<Preset>(&text) {
                Ok(preset) => {
                    let before = self.preset.clone();
                    self.preset = preset;
                    self.dirty = true;
                    if self.connected {
                        let _ = self.midi.push_live(&before, &self.preset);
                    }
                    self.status = format!("Imported {}", path.display());
                }
                Err(e) => self.status = format!("Invalid preset JSON: {e}"),
            },
            Err(e) => self.status = format!("Import failed: {e}"),
        }
    }
}

fn chain_toggle_status(before: &Preset, after: &Preset) -> Option<String> {
    let checks: [(&str, bool, bool); 7] = [
        ("Pre FX", before.pedal_enabled, after.pedal_enabled),
        ("Amp", before.preamp_enabled, after.preamp_enabled),
        ("Modulation", before.modulation_enabled, after.modulation_enabled),
        ("Delay", before.delay_enabled, after.delay_enabled),
        ("Reverb", before.reverb_enabled, after.reverb_enabled),
        ("Power Amp", before.poweramp_enabled, after.poweramp_enabled),
        ("Cabinet", before.cabinet_enabled, after.cabinet_enabled),
    ];
    for (name, was, now) in checks {
        if was != now {
            return Some(format!(
                "Sent {name} {} to amp",
                if now { "ON" } else { "OFF" }
            ));
        }
    }
    None
}

fn chain_block(
    ui: &mut Ui,
    index: &str,
    title: &str,
    enabled: &mut bool,
    body: impl FnOnce(&mut Ui),
) {
    theme::block_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(theme::block_index(&format!("{index} /")));
            ui.label(theme::block_title(title));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                power_toggle(ui, enabled);
            });
        });
        ui.add_space(10.0);
        // Params stay editable while off, matching the mock.
        body(ui);
    });
    ui.add_space(10.0);
}

fn power_toggle(ui: &mut Ui, on: &mut bool) {
    let label = if *on { "ON" } else { "OFF" };
    let fill = if *on { GOLD } else { Color32::from_rgb(48, 48, 48) };
    let text = if *on { INK_DARK } else { MUTED };
    let response = ui.add(
        egui::Button::new(RichText::new(label).color(text).strong().size(12.0))
            .fill(fill)
            .corner_radius(egui::CornerRadius::same(10))
            .min_size(Vec2::new(54.0, 28.0)),
    );
    if response.clicked() {
        *on = !*on;
    }
}

fn labeled(ui: &mut Ui, label: &str, contents: impl FnOnce(&mut Ui)) {
    ui.vertical(|ui| {
        ui.label(theme::field_label(label));
        ui.horizontal(contents);
    });
}

const CONTROL_W: f32 = 156.0;
const CONTROL_H: f32 = 56.0;

fn controls_wrap(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(14.0, 12.0);
        add_contents(ui);
    });
}

fn control_cell(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(
        Vec2::new(CONTROL_W, CONTROL_H),
        Layout::top_down(Align::Min),
        add_contents,
    );
}

fn field_combo<T: Copy + PartialEq>(
    ui: &mut Ui,
    id: &str,
    label: &str,
    value: &mut T,
    options: &[T],
    to_label: fn(T) -> &'static str,
) {
    control_cell(ui, |ui| {
        ui.label(theme::field_label(label));
        egui::ComboBox::from_id_salt(egui::IdSalt::new(id))
            .selected_text(to_label(*value))
            .width(CONTROL_W - 8.0)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
            .show_ui(ui, |ui| {
                ui.set_min_width(CONTROL_W - 8.0);
                for opt in options {
                    ui.selectable_value(value, *opt, to_label(*opt));
                }
            });
    });
}

fn mode_field(ui: &mut Ui, id: &str, label: &str, value: &mut u8, options: &[&str]) {
    control_cell(ui, |ui| {
        ui.label(theme::field_label(label));
        let text = options.get(*value as usize).copied().unwrap_or("?");
        egui::ComboBox::from_id_salt(egui::IdSalt::new(id))
            .selected_text(text)
            .width(CONTROL_W - 8.0)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
            .show_ui(ui, |ui| {
                ui.set_min_width(CONTROL_W - 8.0);
                for (i, opt) in options.iter().enumerate() {
                    ui.selectable_value(value, i as u8, *opt);
                }
            });
    });
}

fn gold_slider(ui: &mut Ui, label: &str, value: &mut u8, min: u8, max: u8) {
    control_cell(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(theme::field_label(label));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{value}"))
                        .color(theme::CREAM)
                        .size(12.0)
                        .strong(),
                );
            });
        });
        let mut v = *value as f32;
        if ui
            .add(
                egui::Slider::new(&mut v, f32::from(min)..=f32::from(max))
                    .show_value(false)
                    .trailing_fill(true),
            )
            .changed()
        {
            *value = v.round() as u8;
        }
    });
}

fn delay_time_slider(ui: &mut Ui, ms: &mut u16) {
    control_cell(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(theme::field_label("TIME"));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{ms} ms"))
                        .color(theme::CREAM)
                        .size(12.0)
                        .strong(),
                );
            });
        });
        let mut v = *ms as f32;
        if ui
            .add(
                egui::Slider::new(&mut v, 0.0..=4000.0)
                    .show_value(false)
                    .trailing_fill(true),
            )
            .changed()
        {
            *ms = v.round() as u16;
        }
    });
}

fn gold_button(ui: &mut Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).color(INK_DARK).strong())
            .fill(GOLD_BRIGHT)
            .corner_radius(egui::CornerRadius::same(8))
            .min_size(Vec2::new(0.0, 32.0)),
    )
}

fn danger_button(ui: &mut Ui, text: &str) -> egui::Response {
    let response = ui.add(
        egui::Button::new(RichText::new(text).color(Color32::WHITE).strong())
            .fill(DANGER)
            .corner_radius(egui::CornerRadius::same(8))
            .min_size(Vec2::new(0.0, 32.0)),
    );
    if response.hovered() {
        ui.painter().rect_filled(
            response.rect,
            egui::CornerRadius::same(8),
            DANGER_HOVER,
        );
        ui.painter().text(
            response.rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        );
    }
    response
}

fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        s.to_string()
    } else {
        format!(
            "{}…",
            s.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    }
}

fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.trim_matches('_').is_empty() {
        "code-preset".into()
    } else {
        cleaned
    }
}
