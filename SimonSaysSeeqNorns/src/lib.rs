//! SimonSaysSeeq Core - High-performance Rust library for MIDI sequencing
//! 
//! This library provides optimized data structures and algorithms for real-time
//! MIDI processing, tempo analysis, and sequencer state management.

use mlua::prelude::*;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::collections::VecDeque;

/// Maximum number of MIDI notes (0-127)
const MAX_MIDI_NOTE: u8 = 127;
/// Maximum number of lanes in the sequencer
const MAX_LANES: u8 = 16;
/// Maximum number of bars
const MAX_BARS: u8 = 16;
/// Maximum number of steps per bar
const MAX_STEPS: u8 = 32;
/// Ticks per step for timing precision
const TICKS_PER_STEP: u32 = 12;

/// High-performance MIDI note event with minimal memory footprint
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MidiNoteEvent {
    pub is_active: bool,
    pub velocity: u8,
    pub tick_count_since_step: u32,
}

impl Default for MidiNoteEvent {
    fn default() -> Self {
        Self {
            is_active: false,
            velocity: 0,
            tick_count_since_step: 0,
        }
    }
}

/// Compact representation of active MIDI notes for efficient processing
#[derive(Debug, Clone)]
pub struct ActiveNote {
    pub note: u8,
    pub velocity: u8,
    pub is_note_on: bool,
}

/// Tempo stability analysis with sliding window
#[derive(Debug)]
pub struct TempoAnalyzer {
    wow_history: VecDeque<f32>,
    flutter_history: VecDeque<f32>,
    wow_window_size: usize,
    flutter_window_size: usize,
    wow_threshold: f32,
    flutter_threshold: f32,
    wow_episodes: u32,
    flutter_episodes: u32,
    total_wow_ticks: u32,
    total_flutter_ticks: u32,
    last_wow_state: bool,
    last_flutter_state: bool,
}

impl TempoAnalyzer {
    pub fn new() -> Self {
        Self {
            wow_history: VecDeque::with_capacity(200),
            flutter_history: VecDeque::with_capacity(50),
            wow_window_size: 192,
            flutter_window_size: 48,
            wow_threshold: 5.0,
            flutter_threshold: 1.0,
            wow_episodes: 0,
            flutter_episodes: 0,
            total_wow_ticks: 0,
            total_flutter_ticks: 0,
            last_wow_state: true,
            last_flutter_state: true,
        }
    }

    pub fn analyze(&mut self, current_tempo: f32) -> (bool, bool) {
        // Add current tempo to history
        self.wow_history.push_back(current_tempo);
        self.flutter_history.push_back(current_tempo);

        // Maintain window sizes
        if self.wow_history.len() > self.wow_window_size {
            self.wow_history.pop_front();
        }
        if self.flutter_history.len() > self.flutter_window_size {
            self.flutter_history.pop_front();
        }

        // Calculate averages efficiently
        let wow_average = if !self.wow_history.is_empty() {
            self.wow_history.iter().sum::<f32>() / self.wow_history.len() as f32
        } else {
            current_tempo
        };

        let flutter_average = if !self.flutter_history.is_empty() {
            self.flutter_history.iter().sum::<f32>() / self.flutter_history.len() as f32
        } else {
            current_tempo
        };

        // Analyze stability
        let wow_stable = (current_tempo - wow_average).abs() <= self.wow_threshold;
        let flutter_stable = (current_tempo - flutter_average).abs() <= self.flutter_threshold;

        // Track episodes and ticks
        if !wow_stable {
            self.total_wow_ticks += 1;
            if self.last_wow_state {
                self.wow_episodes += 1;
            }
        }

        if !flutter_stable {
            self.total_flutter_ticks += 1;
            if self.last_flutter_state {
                self.flutter_episodes += 1;
            }
        }

        self.last_wow_state = wow_stable;
        self.last_flutter_state = flutter_stable;

        (wow_stable, flutter_stable)
    }

    pub fn get_stats(&self) -> (u32, u32, u32, u32) {
        (self.wow_episodes, self.flutter_episodes, self.total_wow_ticks, self.total_flutter_ticks)
    }
}

/// Main sequencer core with optimized data structures
#[derive(Debug)]
pub struct SequencerCore {
    /// Optimized MIDI event storage using flat arrays and hash maps
    midi_events: FxHashMap<(u8, u8, u8, u8, bool), MidiNoteEvent>,
    
    /// Pre-allocated buffer for active notes to avoid allocations
    active_notes_buffer: SmallVec<[ActiveNote; 32]>,
    
    /// Tempo analysis engine
    tempo_analyzer: TempoAnalyzer,
    
    /// Current sequencer state
    current_step: u8,
    current_bar: u8,
    current_lane: u8,
    tick_count: u32,
    tick_count_since_step: u32,
    
    /// Step boundaries
    first_step: u8,
    last_step: u8,
    
    /// Performance counters
    process_count: u64,
    last_active_note_count: usize,
}

impl SequencerCore {
    pub fn new() -> Self {
        Self {
            midi_events: FxHashMap::default(),
            active_notes_buffer: SmallVec::new(),
            tempo_analyzer: TempoAnalyzer::new(),
            current_step: 1,
            current_bar: 1,
            current_lane: 1,
            tick_count: 0,
            tick_count_since_step: 0,
            first_step: 1,
            last_step: 16,
            process_count: 0,
            last_active_note_count: 0,
        }
    }

    /// Set MIDI note event with validation
    pub fn set_midi_event(&mut self, lane: u8, bar: u8, step: u8, note: u8, is_on: bool, event: MidiNoteEvent) -> bool {
        if lane > MAX_LANES || bar > MAX_BARS || step > MAX_STEPS || note > MAX_MIDI_NOTE {
            return false;
        }
        
        self.midi_events.insert((lane, bar, step, note, is_on), event);
        true
    }

    /// Get MIDI note event
    pub fn get_midi_event(&self, lane: u8, bar: u8, step: u8, note: u8, is_on: bool) -> Option<&MidiNoteEvent> {
        self.midi_events.get(&(lane, bar, step, note, is_on))
    }

    /// Fast MIDI processing - the core performance-critical function
    pub fn process_midi_tick(&mut self, current_tempo: f32) -> &[ActiveNote] {
        self.process_count += 1;
        self.tick_count = self.tick_count.wrapping_add(1);
        
        // Clear the buffer for new active notes
        self.active_notes_buffer.clear();
        
        // Analyze tempo stability
        let (_wow_stable, _flutter_stable) = self.tempo_analyzer.analyze(current_tempo);
        
        // Process MIDI events for current position
        let current_key = (self.current_lane, self.current_bar, self.current_step);
        
        // Efficient iteration over potentially active notes
        for note in 0..=MAX_MIDI_NOTE {
            // Check note ON events
            if let Some(event) = self.midi_events.get(&(current_key.0, current_key.1, current_key.2, note, true)) {
                if event.is_active && event.tick_count_since_step == self.tick_count_since_step {
                    self.active_notes_buffer.push(ActiveNote {
                        note,
                        velocity: event.velocity,
                        is_note_on: true,
                    });
                }
            }
            
            // Check note OFF events
            if let Some(event) = self.midi_events.get(&(current_key.0, current_key.1, current_key.2, note, false)) {
                if event.is_active && event.tick_count_since_step == self.tick_count_since_step {
                    self.active_notes_buffer.push(ActiveNote {
                        note,
                        velocity: 0,
                        is_note_on: false,
                    });
                }
            }
        }
        
        self.last_active_note_count = self.active_notes_buffer.len();
        &self.active_notes_buffer
    }

    /// Update sequencer position
    pub fn set_position(&mut self, lane: u8, bar: u8, step: u8) {
        self.current_lane = lane.min(MAX_LANES);
        self.current_bar = bar.min(MAX_BARS);
        self.current_step = step.clamp(self.first_step, self.last_step);
    }

    /// Set step boundaries
    pub fn set_step_range(&mut self, first: u8, last: u8) {
        self.first_step = first.max(1).min(MAX_STEPS);
        self.last_step = last.max(self.first_step).min(MAX_STEPS);
        
        // Ensure current step is within bounds
        self.current_step = self.current_step.clamp(self.first_step, self.last_step);
    }

    /// Advance to next step
    pub fn advance_step(&mut self) -> (u8, u8) {
        self.tick_count_since_step = 0;
        self.current_step += 1;
        
        if self.current_step > self.last_step {
            self.current_step = self.first_step;
            self.current_bar += 1;
            
            if self.current_bar > MAX_BARS {
                self.current_bar = 1;
            }
        }
        
        (self.current_step, self.current_bar)
    }

    /// Increment tick count within current step
    pub fn increment_tick_since_step(&mut self) {
        self.tick_count_since_step = (self.tick_count_since_step + 1) % TICKS_PER_STEP;
    }

    /// Get current position
    pub fn get_position(&self) -> (u8, u8, u8) {
        (self.current_lane, self.current_bar, self.current_step)
    }

    /// Get tempo analysis statistics
    pub fn get_tempo_stats(&self) -> (u32, u32, u32, u32) {
        self.tempo_analyzer.get_stats()
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> (u64, usize, usize) {
        (
            self.process_count,
            self.last_active_note_count,
            self.midi_events.len()
        )
    }

    /// Clear all MIDI events
    pub fn clear_all_events(&mut self) {
        self.midi_events.clear();
    }

    /// Bulk load MIDI events for better performance
    pub fn bulk_load_events(&mut self, events: Vec<((u8, u8, u8, u8, bool), MidiNoteEvent)>) {
        self.midi_events.reserve(events.len());
        for (key, event) in events {
            self.midi_events.insert(key, event);
        }
    }
}

impl LuaUserData for SequencerCore {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Process MIDI tick - main performance function
        methods.add_method_mut("process_midi_tick", |lua, this, tempo: f32| {
            let active_notes = this.process_midi_tick(tempo);
            
            // Convert to Lua table format: {{note, velocity, status}, ...}
            let result = lua.create_table()?;
            for (i, note_data) in active_notes.iter().enumerate() {
                let note_table = lua.create_table()?;
                note_table.set(1, note_data.note)?;
                note_table.set(2, note_data.velocity)?;
                note_table.set(3, if note_data.is_note_on { 144u8 } else { 128u8 })?;
                result.set(i + 1, note_table)?;
            }
            
            Ok(result)
        });

        // Set MIDI event
        methods.add_method_mut("set_midi_event", |_, this, (lane, bar, step, note, is_on, is_active, velocity, tick_offset): (u8, u8, u8, u8, bool, bool, u8, u32)| {
            let event = MidiNoteEvent {
                is_active,
                velocity,
                tick_count_since_step: tick_offset,
            };
            Ok(this.set_midi_event(lane, bar, step, note, is_on, event))
        });

        // Get MIDI event
        methods.add_method("get_midi_event", |lua, this, (lane, bar, step, note, is_on): (u8, u8, u8, u8, bool)| {
            if let Some(event) = this.get_midi_event(lane, bar, step, note, is_on) {
                let result = lua.create_table()?;
                result.set("is_active", event.is_active)?;
                result.set("velocity", event.velocity)?;
                result.set("tick_count_since_step", event.tick_count_since_step)?;
                Ok(Some(result))
            } else {
                Ok(None)
            }
        });

        // Set sequencer position
        methods.add_method_mut("set_position", |_, this, (lane, bar, step): (u8, u8, u8)| {
            this.set_position(lane, bar, step);
            Ok(())
        });

        // Get sequencer position
        methods.add_method("get_position", |_, this, ()| {
            let (lane, bar, step) = this.get_position();
            Ok((lane, bar, step))
        });

        // Set step range
        methods.add_method_mut("set_step_range", |_, this, (first, last): (u8, u8)| {
            this.set_step_range(first, last);
            Ok(())
        });

        // Advance step
        methods.add_method_mut("advance_step", |_, this, ()| {
            let (step, bar) = this.advance_step();
            Ok((step, bar))
        });

        // Increment tick since step
        methods.add_method_mut("increment_tick_since_step", |_, this, ()| {
            this.increment_tick_since_step();
            Ok(())
        });

        // Get tempo statistics
        methods.add_method("get_tempo_stats", |_, this, ()| {
            let (wow_episodes, flutter_episodes, wow_ticks, flutter_ticks) = this.get_tempo_stats();
            Ok((wow_episodes, flutter_episodes, wow_ticks, flutter_ticks))
        });

        // Get performance statistics
        methods.add_method("get_performance_stats", |_, this, ()| {
            let (process_count, active_notes, total_events) = this.get_performance_stats();
            Ok((process_count, active_notes, total_events))
        });

        // Clear all events
        methods.add_method_mut("clear_all_events", |_, this, ()| {
            this.clear_all_events();
            Ok(())
        });

        // Tempo analysis function
        methods.add_method_mut("analyze_tempo_stability", |_, this, tempo: f32| {
            let (wow_stable, flutter_stable) = this.tempo_analyzer.analyze(tempo);
            Ok((wow_stable, flutter_stable))
        });
    }
}

/// Lua module exports
#[mlua::lua_module]
fn simon_says_seeq_core(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;

    // Create new sequencer instance
    exports.set("new_sequencer", lua.create_function(|lua, ()| {
        let sequencer = SequencerCore::new();
        lua.create_userdata(sequencer)
    })?)?;

    Ok(exports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequencer_creation() {
        let sequencer = SequencerCore::new();
        assert_eq!(sequencer.get_position(), (1, 1, 1));
    }

    #[test]
    fn test_midi_event_storage() {
        let mut sequencer = SequencerCore::new();
        let event = MidiNoteEvent {
            is_active: true,
            velocity: 100,
            tick_count_since_step: 5,
        };
        
        assert!(sequencer.set_midi_event(1, 1, 1, 60, true, event));
        
        let retrieved = sequencer.get_midi_event(1, 1, 1, 60, true);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().velocity, 100);
    }

    #[test]
    fn test_tempo_analysis() {
        let mut analyzer = TempoAnalyzer::new();
        
        // Test stable tempo
        let (wow_stable, flutter_stable) = analyzer.analyze(120.0);
        assert!(wow_stable);
        assert!(flutter_stable);
        
        // Test unstable tempo
        let (wow_stable, _) = analyzer.analyze(130.0);
        assert!(!wow_stable);
    }

    #[test]
    fn test_step_advancement() {
        let mut sequencer = SequencerCore::new();
        sequencer.set_step_range(1, 4);
        
        let (step, bar) = sequencer.advance_step();
        assert_eq!(step, 2);
        assert_eq!(bar, 1);
        
        // Test wrap around
        sequencer.current_step = 4;
        let (step, bar) = sequencer.advance_step();
        assert_eq!(step, 1);
        assert_eq!(bar, 2);
    }
}