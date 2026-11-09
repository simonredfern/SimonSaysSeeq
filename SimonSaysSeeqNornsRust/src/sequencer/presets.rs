//! Row-specific preset patterns (grid one, row 7, column 3 held + a step on a row).
//!
//! Each sequence row has its own bank, because each row plays a different instrument:
//! row 0 kick, row 1 open hi-hat, row 2 closed hi-hat, row 3 clap, row 4 accent, rows 5-6 synth lines.
//! The step pressed picks the preset: step 0 (the first column) clears the row, step 1 is the
//! standard techno pattern for that instrument, and the presets get denser and more unusual
//! from there. Most presets on grid two (steps 16-31) are two-bar patterns with a variation
//! or fill in the second bar.
//!
//! Patterns are written as text: `x` = step on, `.` = step off. A 16-character pattern is one
//! bar of 16ths and is played twice to fill the 32 steps; a 32-character pattern is two bars.
//! Bar positions: `1 e & a 2 e & a 3 e & a 4 e & a`, so beats are at characters 0, 4, 8 and 12.

/// Kick drum (row 0): four on the floor first, then pickups, broken patterns and rolls
const KICK: [&str; 31] = [
    // 1: four on the floor
    "x...x...x...x...",
    // 2: plus a pickup on the last 16th
    "x...x...x...x..x",
    // 3: plus the "and" of 4
    "x...x...x...x.x.",
    // 4: plus the last 16th of beat 2
    "x...x..xx...x...",
    // 5: plus the last 16th of beat 3
    "x...x...x..xx...",
    // 6: pickups into beats 2 and 4
    "x..xx...x..xx...",
    // 7: galloping 8ths after beats 2 and 4
    "x...x.x.x...x.x.",
    // 8: end-of-bar fill
    "x...x...x...xx.x",
    // 9: broken 3-3-4-3-3, no four on the floor
    "x..x..x...x..x..",
    // 10: tresillo twice
    "x..x..x.x..x..x.",
    // 11: doubled beats 2 and 4
    "x...xx..x...xx..",
    // 12: 8th pairs on beats 1 and 3
    "x.x.x...x.x.x...",
    // 13: syncopated and busier
    "x..xx.x.x..xx.x.",
    // 14: doubled downbeats
    "xx..x...xx..x...",
    // 15: straight 8ths
    "x.x.x.x.x.x.x.x.",
    // 16: four on the floor, 16th roll at the end of bar 2
    "x...x...x...x...x...x...x...xxxx",
    // 17: four on the floor, 8ths building at the end of bar 2
    "x...x...x...x...x...x...x.x.x.xx",
    // 18: four on the floor, broken bar 2
    "x...x...x...x...x..x..x..x..x.x.",
    // 19: broken bar 1, 8ths in bar 2
    "x..x..x...x..x..x..x..x.x.x.x.x.",
    // 20: 3+4+3+4+2 groupings
    "x..x...x..x...x.",
    // 21: off-grid, busy
    "x.xx.x.xx.x..x.x",
    // 22: dense syncopation
    "xx.x.xx.xx.x.xx.",
    // 23: clave-like kick
    "x..x.x..x..x.x.x",
    // 24: triple hits on beats 1 and 3
    "x.xxx...x.xxx...",
    // 25: triple hits on beats 1 and 3, busier
    "xxx.x.x.xxx.x.x.",
    // 26: dense, alternating
    "x.x.xx.xx.x.xx.x",
    // 27: groups of two
    "xx.xx.xx.xx.xx.x",
    // 28: groups of three
    "x.xxx.xxx.xxx.x.",
    // 29: nearly all 16ths
    "xxxx.xxx.xxx.xxx",
    // 30: 8ths, then a 16th roll
    "x.x.x.x.xxxxxxxx",
    // 31: all 16ths
    "xxxxxxxxxxxxxxxx",
];

/// Open hi-hat (row 1): offbeat 8ths first, then pushes, 16ths and rolls
const OPEN_HAT: [&str; 31] = [
    // 1: offbeat 8ths, the techno standard
    "..x...x...x...x.",
    // 2: plus the last 16th of the bar
    "..x...x...x...xx",
    // 3: plus the "and" of 3
    "..x...x...x.x.x.",
    // 4: doubled offbeats on 2 and 4
    "..x..xx...x..xx.",
    // 5: offbeat pairs
    "..xx..xx..xx..xx",
    // 6: offbeats plus beats 2, 3 and 4
    "..x.x.x.x.x.x.x.",
    // 7: last 16th of each beat
    "...x...x...x...x",
    // 8: offbeats, busier end of bar 2
    "..x...x...x...x...x...x...x.x.x.",
    // 9: 16th pairs before each offbeat
    ".xx..xx..xx..xx.",
    // 10: every third 16th from the first offbeat
    "..x..x..x..x..x.",
    // 11: every off-16th
    ".x.x.x.x.x.x.x.x",
    // 12: doubled pushes
    "..xx.xx...xx.xx.",
    // 13: straight 8ths
    "x.x.x.x.x.x.x.x.",
    // 14: offbeats with extra pushes
    "..x.x.xx..x.x.xx",
    // 15: three of every four 16ths
    ".xxx.xxx.xxx.xxx",
    // 16: offbeats, then a 16th roll at the end of bar 2
    "..x...x...x...x...x...x...xxxxxx",
    // 17: lopsided offbeats
    "..x....x..x....x",
    // 18: late and early
    "...x..x...x..x..",
    // 19: offbeats with late answers
    "..x..x.x..x..x.x",
    // 20: every third 16th from the second
    ".x..x..x..x..x..",
    // 21: every third 16th from the first
    "x..x..x..x..x..x",
    // 22: triple pushes
    "..xxx.x...xxx.x.",
    // 23: busy syncopation
    ".x.xx.x..x.xx.x.",
    // 24: groups of two
    "xx.xx.xx.xx.xx.x",
    // 25: offbeat pairs, then 16ths
    "..xx..xx..xxxxxx",
    // 26: rolling triples
    ".xxx..x..xxx..x.",
    // 27: dense groups of three
    "x.xxx.xxx.xxx.xx",
    // 28: nearly all 16ths
    ".xxxxxxx.xxxxxxx",
    // 29: dense, gaps every third
    "xx.xxx.xxx.xxx.x",
    // 30: offbeats in bar 1, all 16ths in bar 2
    "..x...x...x...x.xxxxxxxxxxxxxxxx",
    // 31: all 16ths
    "xxxxxxxxxxxxxxxx",
];

/// Closed hi-hat (row 2): 8ths and 16ths first, then shuffles, gaps and rolls
const CLOSED_HAT: [&str; 31] = [
    // 1: straight 8ths
    "x.x.x.x.x.x.x.x.",
    // 2: straight 16ths
    "xxxxxxxxxxxxxxxx",
    // 3: 16ths with the downbeat 16th after each beat left out
    "x.xxx.xxx.xxx.xx",
    // 4: off-16ths
    ".x.x.x.x.x.x.x.x",
    // 5: 8ths plus a last 16th
    "x.x.x.x.x.x.x.xx",
    // 6: 16ths with a gap on each offbeat
    "xx.xxx.xxx.xxx.x",
    // 7: shuffling pairs
    "x.xx.xx.x.xx.xx.",
    // 8: three of every four 16ths
    "xxx.xxx.xxx.xxx.",
    // 9: pairs across the beat
    "x..xx..xx..xx..x",
    // 10: busy off-beat groups
    ".xx.xxx..xx.xxx.",
    // 11: 8ths with pushes
    "x.x.xx.xx.x.xx.x",
    // 12: dense with gaps
    "xxx.xx.xxxx.xx.x",
    // 13: syncopated
    ".x.xx.x..x.xx.xx",
    // 14: every third 16th
    "x..x..x..x..x..x",
    // 15: groups of two
    "xx.xx.xx.xx.xx.x",
    // 16: 8ths, then a 16th roll at the end of bar 2
    "x.x.x.x.x.x.x.x.x.x.x.x.xxxxxxxx",
    // 17: 16ths, thinning to 8ths at the end of bar 2
    "xxxxxxxxxxxxxxxxxxxxxxxxx.x.x.x.",
    // 18: shuffling
    "x.xx.x.xx.xx.x.x",
    // 19: three of every four, late
    ".xxx.xxx.xxx.xxx",
    // 20: sparse
    "x...x.x.x...x.x.",
    // 21: pairs on the beat
    "xx..xx..xx..xx..",
    // 22: lopsided 8ths
    "x.x..x.x..x.x..x",
    // 23: broken
    ".x..xx.x.x..xx.x",
    // 24: dense triples
    "xxx.x.xxx.x.xxx.",
    // 25: very dense
    "x.xxxx.xx.xxxx.x",
    // 26: dense, alternating
    "xx.x.xx.x.xx.x.x",
    // 27: off-grid
    ".xx.x.xx.x.xx.x.",
    // 28: nearly all 16ths
    "xxxx.xxxxx.xxxx.",
    // 29: 8ths with a busy end
    "x.x.x.xxx.x.x.xx",
    // 30: all 16ths in bar 1, off-16ths in bar 2
    "xxxxxxxxxxxxxxxx.x.x.x.x.x.x.x.x",
    // 31: all 16ths with two gaps
    "xx.xxxxx.xxxxx.x",
];

/// Clap (row 3): beats 2 and 4 first, then ghost notes, offbeats and rolls
const CLAP: [&str; 31] = [
    // 1: beats 2 and 4, the backbeat
    "....x.......x...",
    // 2: plus a pickup at the end of the bar
    "....x.......x..x",
    // 3: plus the "and" of 4
    "....x.......x.x.",
    // 4: plus a ghost after beat 2
    "....x..x....x...",
    // 5: plus a push into beat 4
    "....x......xx...",
    // 6: backbeat with 8th answers
    "....x.x.....x.x.",
    // 7: backbeat, answer at the end of bar 2
    "....x.......x.......x.......x.x.",
    // 8: ghosts after each clap
    "....x..x....x..x",
    // 9: pushes before each clap
    "..x.x.....x.x...",
    // 10: backbeat plus scattered ghosts
    "....x.x..x..x...",
    // 11: busy ghosts
    ".x..x..x.x..x..x",
    // 12: doubled claps
    "....xx......xx..",
    // 13: beats 2, 3 and 4
    "....x...x...x...",
    // 14: on the beat plus 8th answers
    "x...x.x.x...x.x.",
    // 15: backbeat with double answers
    "....x.xx....x.xx",
    // 16: backbeat, snare roll at the end of bar 2
    "....x.......x.......x...x.xxxxxx",
    // 17: backbeat, building at the end of bar 2
    "....x.......x.......x..x.x.x.xxx",
    // 18: displaced
    "...x....x..x....",
    // 19: offbeat claps
    "..x...x...x...x.",
    // 20: late offbeats
    ".x...x...x...x..",
    // 21: every third 16th
    "x..x..x..x..x..x",
    // 22: busy backbeat
    "....x.x.x...x.x.",
    // 23: syncopated
    ".x.xx.x..x.xx.x.",
    // 24: pairs
    "..xx..xx..xx..xx",
    // 25: straight 8ths
    "x.x.x.x.x.x.x.x.",
    // 26: dense off-grid
    ".xx.x.xx.xx.x.xx",
    // 27: bursts after beats 2 and 4
    "....xxxx....xxxx",
    // 28: groups of three
    "x.xxx.xxx.xxx.xx",
    // 29: three of every four 16ths
    ".xxx.xxx.xxx.xxx",
    // 30: backbeat in bar 1, roll through bar 2
    "....x.......x...xxxxxxxxxxxxxxxx",
    // 31: all 16ths
    "xxxxxxxxxxxxxxxx",
];

/// Accent (row 4): beats 1 and 3 first, then offbeats, syncopation and dense patterns
const ACCENT: [&str; 31] = [
    // 1: beats 1 and 3
    "x.......x.......",
    // 2: once per bar, on the downbeat
    "x...............",
    // 3: every beat
    "x...x...x...x...",
    // 4: offbeat 8ths
    "..x...x...x...x.",
    // 5: beats 1 and 3 plus the "and" of 4
    "x.......x.....x.",
    // 6: every beat plus the last 16th
    "x...x...x...x..x",
    // 7: tresillo
    "x..x..x.x..x..x.",
    // 8: broken 3-3-4-3-3
    "x..x..x...x..x..",
    // 9: once per bar, busier end of bar 2
    "x...............x.....x...x.x.x.",
    // 10: late offbeats
    ".x...x...x...x..",
    // 11: straight 8ths
    "x.x.x.x.x.x.x.x.",
    // 12: 3+4+3+4+2 groupings
    "x..x...x..x...x.",
    // 13: displaced
    "...x..x...x..x..",
    // 14: syncopated
    "x.xx.x..x.xx.x..",
    // 15: pairs on the beat
    "xx..xx..xx..xx..",
    // 16: beats 1 and 3, building in bar 2
    "x.......x.......x...x...x.x.xxxx",
    // 17: lopsided 8ths
    "x.x..x.x..x.x..x",
    // 18: off-16ths
    ".x.x.x.x.x.x.x.x",
    // 19: clave-like
    "x..x.x..x..x.x.x",
    // 20: busy
    "x...xx.xx...xx.x",
    // 21: pairs before the offbeats
    ".xx..xx..xx..xx.",
    // 22: groups of three
    "x.xxx.xxx.xxx.xx",
    // 23: dense syncopation
    "xx.x.xx.xx.x.xx.",
    // 24: dense, alternating
    "x.x.xx.xx.x.xx.x",
    // 25: three of every four 16ths
    ".xxx.xxx.xxx.xxx",
    // 26: three of every four, early
    "xxx.xxx.xxx.xxx.",
    // 27: dense bursts
    "x.xxxxx.x.xxxxx.",
    // 28: nearly all 16ths
    "xx.xxxxxxx.xxxxx",
    // 29: once in bar 1, all 16ths in bar 2
    "x...............xxxxxxxxxxxxxxxx",
    // 30: all 16ths with two gaps
    "xxxx.xxxxxxx.xxx",
    // 31: all 16ths
    "xxxxxxxxxxxxxxxx",
];

/// Synth lines (rows 5 and 6): offbeat bass first, then rolling bass, arps and odd groupings
const SYNTH: [&str; 31] = [
    // 1: offbeat bass, the techno standard
    "..x...x...x...x.",
    // 2: rolling bass (every 16th except the beat)
    ".xxx.xxx.xxx.xxx",
    // 3: offbeat pairs
    "..xx..xx..xx..xx",
    // 4: straight 8ths
    "x.x.x.x.x.x.x.x.",
    // 5: broken 3-3-4-3-3
    "x..x..x...x..x..",
    // 6: every third 16th from the first offbeat
    "..x..x..x..x..x.",
    // 7: off-16ths
    ".x.x.x.x.x.x.x.x",
    // 8: every third 16th with an extra at the end
    "x..x..x..x..x.x.",
    // 9: groups of two
    "xx.xx.xx.xx.xx.x",
    // 10: shuffling pairs
    "x.xx.xx.x.xx.xx.",
    // 11: shuffling pairs, late
    ".xx.xx.x.xx.xx.x",
    // 12: sparse with a push
    "x...x.xx..x.x.x.",
    // 13: lopsided
    "x.x..x.x.x..x.x.",
    // 14: triple pushes
    "..xxx...xxx...xx",
    // 15: all 16ths (arp)
    "xxxxxxxxxxxxxxxx",
    // 16: offbeat bass, 16th run at the end of bar 2
    "..x...x...x...x...x...x...x..xxx",
    // 17: rolling bass, thinning at the end of bar 2
    ".xxx.xxx.xxx.xxx.xxx.xxx.x.x.x.x",
    // 18: clave-like
    "x..x.x..x..x.x.x",
    // 19: dotted quarters (every 6th 16th)
    "x.....x.....x...",
    // 20: every third 16th
    "x..x..x..x..x..x",
    // 21: 4+3+3+4+2 groupings
    "x...x..x..x...x.",
    // 22: every third 16th from the second
    ".x..x..x.x..x..x",
    // 23: dense syncopation
    "x.x.xx.x.xx.x.xx",
    // 24: dense pairs
    "xx..x.xx..x.xx.x",
    // 25: dense groups
    "x.xxx.xx.xxx.xx.",
    // 26: nearly all 16ths
    ".xxxxx.x.xxxxx.x",
    // 27: 8ths, then 16ths
    "x.x.x.x.xxxxxxxx",
    // 28: dense triples
    "xxx.xx.x.xxx.xx.",
    // 29: all 16ths in bar 1, off-16ths in bar 2
    "xxxxxxxxxxxxxxxx.x.x.x.x.x.x.x.x",
    // 30: dense with gaps every third
    "xx.xxx.xxx.xxx.x",
    // 31: all 16ths with two gaps
    "xxxx.xxxxxx.xxxx",
];

/// The preset bank for a sequence row (rows 5 and 6 share the synth bank)
fn bank(row: usize) -> &'static [&'static str; 31] {
    // `match` on the row number; `&` returns a reference to the constant array
    match row {
        // Kick drum
        0 => &KICK,
        // Open hi-hat
        1 => &OPEN_HAT,
        // Closed hi-hat
        2 => &CLOSED_HAT,
        // Clap
        3 => &CLAP,
        // Accent
        4 => &ACCENT,
        // Synth lines (and anything else)
        _ => &SYNTH,
    }
}

/// The 32 grid values (0 off, 1 on) for preset `column` on `row`. Column 0 clears the row.
pub fn row_preset(row: usize, column: usize) -> Vec<u8> {
    // Column 0, or anything past the last preset: all steps off
    if column == 0 || column > 31 {
        // 32 zeros
        return vec![0; 32];
    }
    // Banks are numbered from column 1, so subtract one for the array index
    let text = bank(row)[column - 1];
    // `bytes()` walks the characters; `cycle()` repeats a 16-character bar to fill 32 steps
    text.bytes().cycle().take(32).map(|c| if c == b'x' { 1 } else { 0 }).collect()
}

#[cfg(test)]
mod tests {
    // Everything from the module above
    use super::*;

    #[test]
    fn every_bank_is_well_formed_with_no_repeats() {
        // Each bank, by name for the error message
        for (name, row) in [("kick", 0), ("open hat", 1), ("closed hat", 2), ("clap", 3), ("accent", 4), ("synth", 5)] {
            // The bank's patterns
            let patterns = bank(row);
            // `enumerate` gives (index, pattern)
            for (i, text) in patterns.iter().enumerate() {
                // One or two bars of 16ths
                assert!(text.len() == 16 || text.len() == 32, "{} preset {} has length {}", name, i + 1, text.len());
                // Only x and .
                assert!(text.bytes().all(|c| c == b'x' || c == b'.'), "{} preset {} has other characters", name, i + 1);
                // Not empty (column 0 is the only empty preset)
                assert!(text.contains('x'), "{} preset {} is empty", name, i + 1);
            }
            // No two presets in a bank play the same 32 steps
            let mut seen = std::collections::HashSet::new();
            // Column numbers 1-31
            for column in 1..=31 {
                // `insert` returns false if it was already there
                assert!(seen.insert(row_preset(row, column)), "{} preset {} repeats an earlier one", name, column);
            }
        }
    }

    #[test]
    fn column_0_clears_and_column_1_is_the_standard() {
        // Column 0 is all off on every row
        for row in 0..7 {
            // 32 zeros
            assert_eq!(row_preset(row, 0), vec![0; 32]);
        }
        // Kick: four on the floor (steps 0, 4, 8, ...)
        let kick = row_preset(0, 1);
        // Every 4th step on, the rest off
        assert!(kick.iter().enumerate().all(|(i, &v)| v == if i % 4 == 0 { 1 } else { 0 }));
        // Clap: beats 2 and 4 (steps 4 and 12 of each bar)
        let clap = row_preset(3, 1);
        // `filter` keeps the steps that are on
        let on: Vec<usize> = (0..32).filter(|&i| clap[i] == 1).collect();
        // Both bars
        assert_eq!(on, vec![4, 12, 20, 28]);
    }

    #[test]
    fn presets_get_denser_along_the_row() {
        // Each bank
        for row in 0..6 {
            // Steps on, averaged over a range of columns
            let density = |columns: std::ops::RangeInclusive<usize>| -> f32 {
                // Number of presets in the range
                let count = columns.clone().count() as f32;
                // Total steps on across them, divided by the count
                columns.map(|c| row_preset(row, c).iter().filter(|&&v| v == 1).count() as f32).sum::<f32>() / count
            };
            // The last eight columns are denser on average than the first eight
            assert!(density(24..=31) > density(1..=8), "row {} doesn't get denser", row);
        }
    }

    #[test]
    fn sixteen_step_presets_repeat_and_thirty_two_step_presets_dont() {
        // Kick column 1 (16 characters): bar 2 is the same as bar 1
        let kick = row_preset(0, 1);
        // `[..16]` is the first bar, `[16..]` the second
        assert_eq!(kick[..16], kick[16..]);
        // Kick column 16 (32 characters): bar 2 has the roll
        let fill = row_preset(0, 16);
        // The bars differ
        assert_ne!(fill[..16], fill[16..]);
    }
}
