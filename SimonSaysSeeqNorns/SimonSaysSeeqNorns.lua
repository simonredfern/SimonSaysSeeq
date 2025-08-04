-- SimonSaysSeeq on Norns
-- Left Button Stop. Right Start
-- Licenced under the AGPL.
version = "1.9.1"

version_string = "SimonSaysSeeq Norns v" .. version

NO_FEATURE = "NO_FEATURE"

the_current_tick_count_since_start = 0

function get_script_path()
    local info = debug.getinfo(1, 'S');
    local script_path = info.source:match [[^@?(.*[\/])[^\/]-$]]
    return script_path
end

print("Current script path is " .. get_script_path())



local open = io.open

local function read_file(path)
    local file = open(path, "rb")  -- r read mode and b binary mode
    if not file then return nil end
    local content = file:read "*a" -- *a or *all reads the whole file
    file:close()
    return content
end


function file_exists(name)
    local f = io.open(name, "r")
    if f ~= nil then
        io.close(f)
        return true
    else return false end
end

-- See gml.noaa.gov/ccgg/trends/ for additional details.

-- Helper function to validate CO2 values consistently
function validate_co2_value(raw_value)
    local co2_numeric = tonumber(raw_value)
    -- Check if value is numeric, positive, within reasonable range (0-10000 ppm), and not NaN
    if co2_numeric and co2_numeric > 0 and co2_numeric < 10000 and co2_numeric == co2_numeric then
        return co2_numeric
    else
        return nil
    end
end

local co2_ppm_daily_latest_value = tonumber(read_file(
_path.dust .. "data/SimonSaysSeeqNorns/simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_daily_latest.csv"));

if (co2_ppm_daily_latest_value) then
    print("here is the co2_ppm_daily_latest_value we got from the file: " .. co2_ppm_daily_latest_value);
    we_have_last_daily_co2_ppm_value = true
else
    print("We do NOT have a daily co2 ppm ");
    we_have_last_daily_co2_ppm_value = false
end



local all_days_path =
_path.dust .. "data/SimonSaysSeeqNorns/simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_all_daily.csv"

if (file_exists(all_days_path)) then
    print("Yes all days path file exists");
    -- create a table out of the csv data.
    -- note the last match is greedy with *
    co2_ppm_list = {}
    no_of_co2_ppm_records = 0
    for line in io.lines(all_days_path) do
        local year, month, day, something, the_co2_ppm_value = line:match("%s*(.-),%s*(.-),%s*(.-),%s*(.-),%s*(.*)")

        -- Validate CO2 value during parsing to catch invalid data early
        local co2_numeric = validate_co2_value(the_co2_ppm_value)
        if co2_numeric then
            co2_ppm_list[#co2_ppm_list + 1] = { year = year, month = month, day = day, something = something, the_co2_ppm_value =
            the_co2_ppm_value }
            no_of_co2_ppm_records = no_of_co2_ppm_records + 1
        else
            print("WARNING: Skipping invalid CO2 record - raw value: " .. tostring(the_co2_ppm_value) .. ", parsed: " .. tostring(co2_numeric))
        end
    end

    -- for i,v in ipairs(co2_ppm_list) do
    --  -- print(i, v.year .. "-" .. v.month .. "-" .. v.day .. " is " .. v.the_co2_ppm_value)

    --   print(i, v.the_co2_ppm_value)

    -- end

    -- Final validation: ensure we have at least some valid CO2 records
    if no_of_co2_ppm_records > 0 then
        we_have_all_daily_co2_ppm_values = true
        print("Successfully loaded " .. no_of_co2_ppm_records .. " valid CO2 records")
    else
        we_have_all_daily_co2_ppm_values = false
        print("WARNING: No valid CO2 records found in file, CO2 features disabled")
    end
else
    print("We do NOT have ALL daily co2 ppm ");
    we_have_all_daily_co2_ppm_values = false
    -- Initialize CO2 variables safely when no data is available
    co2_ppm_list = {}
    no_of_co2_ppm_records = 0
end





-- To measure / display tempo instability
-- Two windows for averaging the tempo.
-- WOW - Big instabillity in Tempo
wow_window_size = 192        -- This is 16 steps at 12 ticks per step.
wow_window_tick_position = 0 -- how far through the averaging window we are.
wow_tempo_sum = 0            -- sum of tempo through the window so far
total_wow_tempo_ticks = 0    -- ticks we are unstable for
wow_tempo_episodes = 0       -- number of unstable episodes.
wow_average_tempo = 0        -- the average tempo over the window
tempo_wow_is_good = 1        -- All good at the moment, (no wow) (assume it is to start with.)
wow_threshold = 3.0          -- the difference between current and average tempo (in bpm) that triggers an episode

-- Flutter - Small instability in Tempo
flutter_window_size = 192
flutter_window_tick_position = 0
flutter_tempo_sum = 0
total_flutter_tempo_ticks = 0
flutter_tempo_episodes = 0
flutter_average_tempo = 0
tempo_flutter_is_good = 1
flutter_threshold = 0.25


sequence_button_x = 0
sequence_button_y = 0
sequence_button_midi = 0
sequence_button_is_pressed = false


arm_row7 = NO_FEATURE
arm_control = NO_FEATURE

print("Current matrix is " ..
    sequence_button_x .. " " .. sequence_button_y .. " " .. sequence_button_midi .. " " .. arm_row7 .. " " .. arm_control)



-- clock is currently external so cant script clock


local volts = 0
local slew = 0

-- on/off for stepped sequence
transport_is_active = true


-- Dimensions of Monome Grid
COLS = 16
ROWS = 8

GATE_12 = 12
GATE_11 = 11
GATE_10 = 10
GATE_9 = 9
GATE_8 = 8
GATE_7 = 7


first_midi_step = 1
last_midi_step = 16

ONE = 1
first_step = 1
midi_step_count = first_step
last_step = COLS

midi_bar_count = 1


arm_feature = NO_FEATURE
preset_grid_button = 0
arm_clock_button = 0
preset_mozart_button = 0


arm_euclidian_rotation_button = 0
arm_euclidian_length_button = 0


arm_put_slide_on = 0
arm_take_slide_off = 0


arm_swing_button = 0

swing_mode = 1

TOTAL_SEQUENCE_ROWS = 7 -- was 6

GRID_ONE_STATE_FILE = _path.dust .. "data/SimonSaysSeeqNorns/SimonSaysSeeq-grid-one-state-" .. version .. ".tbl"

MOZART_STATE_FILE = _path.dust .. "data/SimonSaysSeeqNorns/SimonSaysSeeq-mozart-state-"  .. version .. ".tbl"

SCROLL_STATE_FILE = _path.dust .. "data/SimonSaysSeeqNorns/SimonSaysSeeq-scroll-state-" .. version .. ".tbl"

SLIDE_STATE_FILE = _path.dust .. "data/SimonSaysSeeqNorns/SimonSaysSeeq-slide-state-" .. version .. ".tbl"

ROW_STATES_FILE = _path.dust .. "data/SimonSaysSeeqNorns/SimonSaysSeeq-row-states-".. version .. ".tbl"

function get_row_states_tally(row_states)
    -- A helper debug function to show the state the row_states table
    -- We use this to get an error if the expected keys are not there.
    local tally = "id:" .. row_states["id"] .. " "
    for row = 1, ROWS do
        tally = tally ..
        " Row: " ..
        row ..
        " first_step is: " .. row_states[row]["first_step"] .. " last_step is: " .. row_states[row]["last_step"]
    end
    return tally
end

function create_row_states()
    -- This table stores various settings and states for each  sequence row.

    local row_states = {}
    for row = 1, TOTAL_SEQUENCE_ROWS do
        row_states[row] = {}   -- create a table for each row
        row_states[row]["first_step"] = 1
        row_states[row]["last_step"] = 16
        row_states[row]["current_step"] = 1
    end

    return row_states
end

function load_row_states()
    row_states = Tab.load(ROW_STATES_FILE)

    print("Result of table load is:")
    print(row_states)

    print(get_row_states_tally(row_states))

    return row_states
end

function init_row_states_table()
    print("Hello from init_row_states_table")

    -- Try to load the table
    local status, err = pcall(load_row_states)

    if status then
        print("load row_states state seems ok. row_states is:")
        print(row_states)
    else
        print("Seems we got an error - setting row_states to nil so we will create it and save it: " .. err)
        row_states = nil
    end

    -- if it doesn't exist
    if row_states == nil then
        print("No row_states table, I will generate a structure and save that")

        row_states = create_row_states()

        Tab.save(row_states, ROW_STATES_FILE)
        row_states = Tab.load(ROW_STATES_FILE)
    else
        print("I already have a row_states table, no need to generate one")
    end


    -- Push Undo so we can get back to initial state
    --push_mozart_undo()

    print("Bye from init_row_states_table")
end -- end init_sequence_row_states_table

last_action_method = ""
last_x = 0
last_y = 0
last_grid_value = 0
last_mozart_value = 0
last_slide_value = 0


last_midi_note_in = -1
last_midi_velocity_in = -1
last_midi_channel_in = -1
last_midi_on_off_in = -1
last_midi_device_in = -1

last_midi_note_on_out = -1
last_midi_on_velocity_out = -1
last_midi_channel_out = -1
last_midi_note_off_out = -1
last_midi_device_out = -1




held_x = 0
held_y = 0



end_of_line_text = ""
output_text = ""

Tab = require "lib/tabutil"

-- note probably on different ports
MIDI_CHANNEL_GATES = 1
MIDI_KEYBOARD_CHANNEL = 1


current_midi_lane = 1 -- so far in norns we only have one any MIDI note number divided by 12 is how the pitch is expressed in voltage (assuming volt per octave)




-- This works well with Flame MGTV factory default settings.
--  http://flame.fortschritt-musik.de/pdf/Manual_Flame_MGTV_module_v100_eng.pdf
LOWEST_MIDI_NOTE_NUMBER_FOR_GATE = 47 -- at least 1 will be added to this.

MIDI_NOTE_ON_VELOCITY = 127
MIDI_NOTE_OFF_VELOCITY = 0

C_MIDI_NOTE_ON = 1;
C_MIDI_NOTE_OFF = 0;

lowest_keyboard_midi_note = 0
highest_keyboard_midi_note = 127



MOZART_BASE_MIDI_NOTE = 24        -- C1  -- 33 = A1
MOZART_INTERVAL_PERFECT_FIFTH = 7 -- go up in fifths
MOZART_INTERVAL_PERFECT_FOURTH = 4
MOZART_INTERVAL_MAJOR_THIRD = 3
MOZART_INTERVAL_MINOR_THIRD = 2

MOZART_RANDOM_MAX_DELTA = 48


-- WARNING --------------------------
-- enabling some of these (not sure which) will cause noticable occasional wow and flutter of tempo. (search for wow and flutter in this file for more info)
enable_midi_clock_out = 0
enable_analog_clock_out = 0 -- guess this is the culprit because it causes many midi messages (analog clock is sent via midi, crow clock out didn't seem to work)
enable_audio_clock_out = 0
-----------------------------
-- End WARNING --------------


need_to_start_midi = true -- Check gate clock situation.
run_conditional_clocks = false

SCREEN_INFO_X = 40
SCREEN_INFO_Y = 49

-- audio_clock_file = _path.dust.."code/softcut-studies/lib/whirl1.aif"

--audio_clock_file = _path.dust.."audio/SimonSaysSeeqAudio/E-RM_multiclock_sample.wav"


-- To copy the file use scp:
-- (this click file was copied from http://www.makenoisemusic.com/)
-- scp modular-pulse.wav we@norns.local:/home/we/dust/audio/SimonSaysSeeqAudio

-- audio_clock_file = _path.dust.."audio/SimonSaysSeeqAudio/0-1-2-3-4-5.wav"

audio_clock_file = _path.dust .. "audio/SimonSaysSeeqAudio/modular-pulse.wav"

BUTTONS = {}

--7th Row -- Probably not used because now using for gates
ROW7_BUTTON_01 = "Button1"
ROW7_BUTTON_02 = "Button2"
ROW7_BUTTON_03 = "Button3"
ROW7_BUTTON_04 = "Button4"
table.insert(BUTTONS, { name = ROW7_BUTTON_01, x = 1, y = 7 })
table.insert(BUTTONS, { name = ROW7_BUTTON_02, x = 2, y = 7 })
table.insert(BUTTONS, { name = ROW7_BUTTON_03, x = 3, y = 7 })
table.insert(BUTTONS, { name = ROW7_BUTTON_04, x = 4, y = 7 })

ROW7_BUTTON_05 = "Button5"
ROW7_BUTTON_06 = "Button6"
ROW7_BUTTON_07 = "Button7"
ROW7_BUTTON_08 = "Button8"
table.insert(BUTTONS, { name = ROW7_BUTTON_05, x = 5, y = 7 })
table.insert(BUTTONS, { name = ROW7_BUTTON_06, x = 6, y = 7 })
table.insert(BUTTONS, { name = ROW7_BUTTON_07, x = 7, y = 7 })
table.insert(BUTTONS, { name = ROW7_BUTTON_08, x = 8, y = 7 })

ROW7_BUTTON_09 = "Button9"
ROW7_BUTTON_10 = "Button10"
ROW7_BUTTON_11 = "Button11"
ROW7_BUTTON_12 = "Button12"
table.insert(BUTTONS, { name = ROW7_BUTTON_09, x = 9, y = 7 })
table.insert(BUTTONS, { name = ROW7_BUTTON_10, x = 10, y = 7 })
table.insert(BUTTONS, { name = ROW7_BUTTON_11, x = 11, y = 7 })
table.insert(BUTTONS, { name = ROW7_BUTTON_12, x = 12, y = 7 })

ROW7_BUTTON_13 = "Button13"
ROW7_BUTTON_14 = "Button14"
ROW7_BUTTON_15 = "Button15"
ROW7_BUTTON_16 = "Button16"
table.insert(BUTTONS, { name = ROW7_BUTTON_13, x = 13, y = 7 })
table.insert(BUTTONS, { name = ROW7_BUTTON_14, x = 14, y = 7 })
table.insert(BUTTONS, { name = ROW7_BUTTON_15, x = 15, y = 7 })
table.insert(BUTTONS, { name = ROW7_BUTTON_16, x = 16, y = 7 })




-- 8th Row
UNDO_GRID_BUTTON = "UndoGridButton"
REDO_GRID_BUTTON = "RedoGridButton"
UNDO_MOZART_BUTTON = "UndoMozartButton"
REDO_MOZART_BUTTON = "RedoMozartButton"
table.insert(BUTTONS, { name = UNDO_GRID_BUTTON, x = 1, y = 8 })
table.insert(BUTTONS, { name = REDO_GRID_BUTTON, x = 2, y = 8 })
table.insert(BUTTONS, { name = UNDO_MOZART_BUTTON, x = 3, y = 8 })
table.insert(BUTTONS, { name = REDO_MOZART_BUTTON, x = 4, y = 8 })


ARM_EUCLIDIAN_EVENTS_BUTTON = "ArmEuclidianEvents"
ARM_EUCLIDIAN_LENGTH_BUTTON = "ArmEuclidianLength"
ARM_EUCLIDIAN_ROTATION_BUTTON = "ArmEuclidianRotation"
ARM_RATCHET_BUTTON = "ArmRatchet"

table.insert(BUTTONS, { name = ARM_EUCLIDIAN_EVENTS_BUTTON, x = 5, y = 8 })
table.insert(BUTTONS, { name = ARM_EUCLIDIAN_LENGTH_BUTTON, x = 6, y = 8 })
table.insert(BUTTONS, { name = ARM_EUCLIDIAN_ROTATION_BUTTON, x = 7, y = 8 })
table.insert(BUTTONS, { name = ARM_RATCHET_BUTTON, x = 8, y = 8 })

ARM_RANDOMISE_GRID_BUTTON = "RandomiseGrid"
ARM_RANDOMISE_MOZART_BUTTON = "RandomiseMozart"
ARM_PRESET_GRID_BUTTON = "PresetGrid"
ARM_PRESET_MOZART_BUTTON = "PresetMozart"
table.insert(BUTTONS, { name = ARM_RANDOMISE_GRID_BUTTON, x = 9, y = 8 })
table.insert(BUTTONS, { name = ARM_RANDOMISE_MOZART_BUTTON, x = 10, y = 8 })
table.insert(BUTTONS, { name = ARM_PRESET_GRID_BUTTON, x = 11, y = 8 })
table.insert(BUTTONS, { name = ARM_PRESET_MOZART_BUTTON, x = 12, y = 8 })

ARM_MOZART_DOWN_BUTTON = "ArmMozartDown"
ARM_MOZART_UP_BUTTON = "ArmMozartUp"
ARM_SLIDE_OFF_BUTTON = "ArmSlideOff"
ARM_SLIDE_ON_BUTTON = "ArmSlideOn"
table.insert(BUTTONS, { name = ARM_MOZART_DOWN_BUTTON, x = 13, y = 8 })
table.insert(BUTTONS, { name = ARM_MOZART_UP_BUTTON, x = 14, y = 8 })
table.insert(BUTTONS, { name = ARM_SLIDE_OFF_BUTTON, x = 15, y = 8 })
table.insert(BUTTONS, { name = ARM_SLIDE_ON_BUTTON, x = 16, y = 8 })


function reset_all_sequence_counters()
    init_midi_step_count()
    init_midi_bar_count()


    -- Initialize CO2 counters safely based on available data
    if no_of_co2_ppm_records > 0 then
        total_step_co2_count = 1 -- This will loop around the co2 ppm rows
        total_tick_co2_count = 1 -- This will also loop around the co2 ppm rows but faster (on each tick)
    else
        total_step_co2_count = 0 -- No CO2 data available
        total_tick_co2_count = 0 -- No CO2 data available
        print("WARNING: No CO2 data available, CO2 counters set to 0")
    end

    -- Safety check: ensure row_states is properly initialized before accessing it
    if row_states == nil then
        print("WARNING: row_states is nil in reset_all_sequence_counters, creating default settings")
        row_states = create_row_states()
    end


    -- why do we do this stuff here? (we just created above?)

    --for row = 1, TOTAL_SEQUENCE_ROWS do
    --    -- Additional safety check for each row
    --    if row_states[row] == nil then
    --        print("WARNING: row_states[" .. row .. "] is nil, creating default row settings")
    --        row_states[row] = {}
    --        row_states[row]["first_step"] = 1
    --        row_states[row]["last_step"] = 16
    --        row_states[row]["current_step"] = 1
    --    end

    --    row_states[row]["first_step"] = first_step
    --    row_states[row]["last_step"] = last_step
    --    row_states[row]["current_step"] = row_states[row]["first_step"]
    --end
end

tick_text = "."

tick_count = 0





function InitStepCountSinceStep()
    the_current_tick_count_since_step = 0
end

function IncrementStepCountSinceStep()
    the_current_tick_count_since_step = the_current_tick_count_since_step + 1
end

InitStepCountSinceStep()


PPQN24_GATES_ARE_ENABLED = true -- kind of duplicated setting


greetings_done = false


MIN_BAR = 1
MAX_BAR = 4

MIN_LANE = 1
MAX_LANE = 2

MAX_STEP = 16


MozartPointer = {}
MozartPointer.__index = MozartPointer

function MozartPointer:new()
    return setmetatable({
        is_active = 0,
        current_midi_lane = 0,
        midi_bar_count = 0,
        midi_step_count = 0,
        midi_note_number = 0
    }, MozartPointer)
end

-- Define the SequenceNote class
SequenceNote = {}
SequenceNote.__index = SequenceNote

-- Constructor for SequenceNote
-- Store extra data about the note (velocity, "exactly" when in a step etc)
-- Note name (number) and step information is stored in the array below.

-- For each sequence step / midi note number  / on-or-off we store a SequenceNote (which defines a bit more info)
-- Arrays are ZERO INDEXED but here we define the SIZE of each DIMENSION of the Array.
-- This way we can easily access a step and the notes there.
-- [step][midi_note][on-or-off]
-- [step] will store a digit between 0 and 15 to represent the step of the sequence.
-- [midi_note] will store between 0 and 127
-- [on-or-off] will store either 1 for MIDI_NOTE_ON or 0 for MIDI_NOTE_OFF
-- SequenceNote keyboard_midi_note_events[MAX_STEP+1][128][2];


function SequenceNote:new()
    return setmetatable({
        velocity = 0,
        tick_count_since_step = 0,
        is_active = 0,
        tick_count_since_start = 0
    }, SequenceNote)
end

function init_keyboard_midi_note_events()
    -- this is a global
    keyboard_midi_note_events = {}

    for lane = MIN_LANE, MAX_LANE do
        keyboard_midi_note_events[lane] = {}
        for bar = MIN_BAR, MAX_BAR do
            keyboard_midi_note_events[lane][bar] = {}
            for step = 1, MAX_STEP do
                keyboard_midi_note_events[lane][bar][step] = {}
                for note = 0, 127 do
                    keyboard_midi_note_events[lane][bar][step][note] = {}
                    for index = 0, 1 do
                        keyboard_midi_note_events[lane][bar][step][note][index] = SequenceNote:new()
                    end
                end
            end
        end
    end
end

function DisableAndTurnOffActiveKeyboardMidiNotes(skip)
    -- An approach to thin out the midi sequence
    -- if skip is 1, actually we don't skip any notes.
    -- TODO find a way to better balance the active on and off events of one note number.
    -- i.e. if we disable on ON note event, we could scroll forward to disable the next OFF event for the same note.
    -- calling this function repeatedly should thin the midi sequence to empty (which it does).
    -- (it might be a bit weird on the way).
    -- Note we want to explicitly also disable note off events even at the risk of creating stuck notes because otherwise
    -- we end up with many more send note off events.




    last_function = 21741

    if skip <= 0 or skip > 10 then
        error("skip should be 1 to 10.")
    end

    local count_active_on_disabled = 0
    local count_active_off_disabled = 0



    -- Disable that note for all steps
    for bc = MIN_BAR, MAX_BAR do
        for sc = first_midi_step, last_midi_step do
            for note = 0, 127 do
                -- From the perspective of active on notes:

                if keyboard_midi_note_events[current_midi_lane][bc][sc][note][1].is_active == 1 then --and keyboard_midi_note_events[current_midi_lane][bc][sc][note][1].velocity > 0 then
                    if count_active_on_disabled % skip == 0 then
                        -- Hmm we should be disabling the note across all steps not just the step where we find it.
                        -- or, how do we disable the corresponding off note?

                        keyboard_midi_note_events[current_midi_lane][bc][sc][note][1].is_active = 0 -- make the note on inactive.
                        keyboard_midi_note_events[current_midi_lane][bc][sc][note][1].velocity = 0 -- make the note velocity zero
                        keyboard_midi_note_events[current_midi_lane][bc][sc][note][0].is_active = 0 -- disable any note off at that position.
                        keyboard_midi_note_events[current_midi_lane][bc][sc][note][0].velocity = 0 -- make any note off zero velocity.

                        SendMidiKeyboardNoteOn(note, 0, 1)                                -- send midi off for that one note
                    end

                    count_active_on_disabled = count_active_on_disabled + 1
                end

                -- From the perspective of active off notes: (see note above)

                if keyboard_midi_note_events[current_midi_lane][bc][sc][note][0].is_active == 1 then
                    if count_active_off_disabled % skip == 0 then
                        keyboard_midi_note_events[current_midi_lane][bc][sc][note][1].is_active = 0 -- make the note on inactive.
                        keyboard_midi_note_events[current_midi_lane][bc][sc][note][1].velocity = 0 -- make the note velocity zero
                        keyboard_midi_note_events[current_midi_lane][bc][sc][note][0].is_active = 0 -- disable any note off at that position.
                        keyboard_midi_note_events[current_midi_lane][bc][sc][note][0].velocity = 0 -- make any note off zero velocity.

                        SendMidiKeyboardNoteOn(note, 0, 1)                                -- send midi off for that one note
                    end

                    count_active_off_disabled = count_active_off_disabled + 1
                end
            end
        end
    end
end

-- Function to disable MIDI notes
function DisableKeyboardMidiNotes(note)
    last_function = 28749

    -- Disable that note for all steps
    for bc = MIN_BAR, MAX_BAR do
        for sc = first_midi_step, last_midi_step do
            keyboard_midi_note_events[current_midi_lane][bc][sc][note][1].velocity = 0
            keyboard_midi_note_events[current_midi_lane][bc][sc][note][1].is_active = 0
            keyboard_midi_note_events[current_midi_lane][bc][sc][note][0].velocity = 0
            keyboard_midi_note_events[current_midi_lane][bc][sc][note][0].is_active = 0
        end
    end

    -- TODO ActiveKeyboardMidiNoteSet[note] = nil  -- Remove note from active set
end

local noteNames = { "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B" }

function midiNoteToName(midiNote)
    if midiNote < 0 or midiNote > 127 then
        return "---" -- unknown
        --  return nil, "Invalid MIDI note number"
    end

    local noteIndex = (midiNote % 12) + 1      -- Lua indices start at 1
    local octave = math.floor(midiNote / 12) - 1 -- MIDI note 0 is in octave -1

    return noteNames[noteIndex] .. octave
end

function SendMidiKeyboardNoteOn(note, velocity, channel)
    --  print ("Hello from SendMidiKeyboardNoteOn")

    SanityCheckMidiNote(note)
    SanityCheckMidiVelocity(velocity)
    SanityCheckMidiChannel(channel) -- don't need to return this, just check it and carry on.

    -- print("SendMidiKeyboardNoteOn note: " .. tostring(note) .. " velocity: " .. tostring(velocity) .. " channel: " .. tostring(channel))

    -- Safety check: ensure MIDI device is initialized before use
    if midi_keyboard_usb_device_port then
        midi_keyboard_usb_device_port:note_on(note, velocity, channel)
    else
        print("WARNING: midi_keyboard_usb_device_port is nil, cannot send MIDI note")
    end


    -- for display
    if velocity > 0 then
        last_midi_note_on_out = note
        last_midi_on_velocity_out = velocity
    else
        last_midi_note_off_out = note
    end

    last_midi_channel_out = channel
end

function AllMidiNotesOff()
    -- this should only be used for a panic.
    -- with normal clear behaviour should only note off the notes that are active.
    print("hello from AllMidiNotesOff")
    for note = 0, 127 do
        SendMidiKeyboardNoteOn(note, 0, 1)
    end
end

function test_midi_output()
    print("Testing MIDI output...")

    if midi_keyboard_usb_device_port then
        print("Sending test MIDI note C4 (60) velocity 100")
        SendMidiKeyboardNoteOn(60, 100, 1)
        clock.sleep(1)
        print("Sending test MIDI note OFF")
        SendMidiKeyboardNoteOn(60, 0, 1)
    else
        print("ERROR: No MIDI keyboard device connected for test")
    end
end

-- Function to process incoming MIDI note events
function OnMidiNoteInEvent(on_off, note, velocity, channel)
    last_function = 466942


    last_midi_note_in = note
    last_midi_velocity_in = velocity
    last_midi_channel_in = channel
    last_midi_on_off_in = on_off

    if channel == MIDI_KEYBOARD_CHANNEL then
        if note >= lowest_keyboard_midi_note and note <= highest_keyboard_midi_note then
            print("on_off is " .. on_off)
            if on_off == C_MIDI_NOTE_ON then
                -- If velocity is low, treat as note off
                if velocity < 40 then
                    -- print(string.format("*** I GOT A LOW VELOCITY %d so will remove note %d from the sequence ***", velocity, note))

                    DisableKeyboardMidiNotes(note)

                    -- Turn the note off
                    SendMidiKeyboardNoteOn(note, 0, channel) -- not send explicit note off or this is the same?
                else
                    -- Process note on

                    -- print(string.format("************* Setting MIDI note ON for note %d When step is %d velocity is %d", note, midi_step_count, velocity))

                    keyboard_midi_note_events[current_midi_lane][midi_bar_count][midi_step_count][note][1].tick_count_since_step =
                    the_current_tick_count_since_step
                    keyboard_midi_note_events[current_midi_lane][midi_bar_count][midi_step_count][note][1].velocity =
                    velocity
                    keyboard_midi_note_events[current_midi_lane][midi_bar_count][midi_step_count][note][1].is_active = 1

                    -- Pass through the note
                    -- NOTE this might cause double ON if our keyboard has both MIDI IN and MIDI OUT connected.
                    -- Echo the midi note through norns to the synth.
                    -- TODO this should be configurable via the grid easily turn on / off midi echo for different keyboard / synth setups.
                    -- Note this was probably not doing anything
                    SendMidiKeyboardNoteOn(note, velocity, channel)
                end

                -- last_note_on = note
                print(string.format("Done setting MIDI note ON for note %d when step is %d velocity is %d", note,
                    midi_step_count, velocity))
            else
                -- Process MIDI note off
                -- print(string.format("Set MIDI note OFF for note %d when bar is %d and step is %d", note, midi_bar_count, midi_step_count))


                keyboard_midi_note_events[current_midi_lane][midi_bar_count][midi_step_count][note][0].tick_count_since_step =
                the_current_tick_count_since_step
                keyboard_midi_note_events[current_midi_lane][midi_bar_count][midi_step_count][note][0].velocity =
                velocity
                keyboard_midi_note_events[current_midi_lane][midi_bar_count][midi_step_count][note][0].is_active = 1

                -- Echo the midi note through norns to the synth.
                -- TODO this should be configurable via the grid easily turn on / off midi echo for different keyboard / synth setups.
                SendMidiKeyboardNoteOn(note, 0, channel)

                -- last_note_off = note


                -- print(string.format("Done setting MIDI note OFF for note %d when bar is %d and step is %d", note, midi_bar_count, midi_step_count))
            end
        else
            print(string.format("###### Note %d out of range (Allowed: %d to %d)", note, lowest_keyboard_midi_note,
                highest_keyboard_midi_note))
        end
    else
        print("###### Ignoring MIDI event on channel " .. channel .. " Expected: " .. MIDI_KEYBOARD_CHANNEL)
    end
end

--------------------------------




my_grid_one = grid.connect(1)
print(my_grid_one)
my_grid_two = grid.connect(2)
print(my_grid_two)





grids_are_dirty = false




INITIAL_MIDI_GATES_PORT = 1 -- In the currrent cable setup this is CLOCK IN and GATES OUT
INITIAL_MIDI_KEYBOARD_PORT = 2




-- Which USB midi ports we should use (defaults)

midi_gates_usb_device_port = midi.connect(INITIAL_MIDI_GATES_PORT)
midi_keyboard_usb_device_port = midi.connect(INITIAL_MIDI_KEYBOARD_PORT)


-- And we can change them and connect after changes.
params:add { type = "number", id = "midi_gates_usb_device_port_id", name = "Gates MIDI Device", min = 1, max = 4, default = INITIAL_MIDI_GATES_PORT, action = function(
    value)
    midi_gates_usb_device_port.event = nil
    midi_gates_usb_device_port = midi.connect(value)
    midi_gates_usb_device_port.event = midi_event

    print("i changed the midi_gates_usb_device_port parameter !")
end }


params:add { type = "number", id = "midi_keyboard_usb_device_port_id", name = "Keyboard MIDI Device", min = 1, max = 4, default = INITIAL_MIDI_KEYBOARD_PORT, action = function(
    value)
    midi_keyboard_usb_device_port.event = nil
    midi_keyboard_usb_device_port = midi.connect(value)
    midi_keyboard_usb_device_port.event = midi_event

    print("i changed the midi keyboard parameter !")
end }



-- defaults
normal_midi_note_is_on = false  -- what is this for?
normal_midi_note_is_off = false
normal_midi_note_in = -1


captured_midi_note_in = -1
midi_note_key_pressed = -1

direction = 1


-- psudo random for our grid ids
math.randomseed(os.time())
math.random() -- call a few times so it gets more random (apparently)
math.random()
math.random()


function Set(list)
    local set = {}
    for _, l in ipairs(list) do set[l] = true end
    return set
end

-- swing 8ths
-- SWING_STEPS = Set { 3, 7, 11, 15 }

-- swing 16
-- we are not doing swing in code
SWING_STEPS = Set { 2, 4, 6, 8, 10, 12, 14, 16 }

-- Show Euclidean instructions on startup
function show_euclidean_instructions()
    screen.clear()

    screen.move(1, 7)
    screen.text("Euclidean Sequencer Ready")

    screen.move(1, 17)
    screen.text("Row 8 Controls:")

    screen.move(1, 24)
    screen.text("5: ARM_EUCLIDIAN_ROTATION")

    screen.move(1, 31)
    screen.text("6: ARM_EUCLIDIAN_LENGTH")

    screen.move(1, 38)
    screen.text("7: ARM_EUCLIDIAN_EVENTS")

    screen.move(1, 48)
    screen.text("All ops work on selected row")

    screen.move(1, 55)
    screen.text("Use buttons 1&2 to undo/redo")

    screen.update()
    clock.sleep(3)
end

-- Fonts: Note, we can use the Foundry app to view all the fonts.
-- Tried to find a fixed font (so strings don't jump around), but currently using the default font
-- Best approach probably is not to have long strings and instead place short strings at specific locations on the screen.


function init_wow_and_flutter_counters()
    -- use on transport start
    total_wow_tempo_ticks = 0
    total_flutter_tempo_ticks = 0
    wow_tempo_episodes = 0
    flutter_tempo_episodes = 0
end

function init_wow_window()
    wow_window_tick_position = 0
    wow_tempo_sum = 0
end

-- small variations in tempo
function init_flutter_window()
    flutter_window_tick_position = 0
    flutter_tempo_sum = 0
end

function SanityCheckMidiNote(note)
    last_function = 9822243
    -- print ("Hello from SanityCheckMidiNote")


    if (note == nil) then
        error("SanityCheckMidiNote says note is nil")
    end

    if not (tonumber(note) >= 0 and tonumber(note) <= 127) then
        error("SanityCheckMidiNote says note is out of bounds with the value: " .. tostring(note))
    end

    return tonumber(note)
end

function SanityCheckMidiVelocity(velocity)
    last_function = 9122243
    -- print ("Hello from SanityCheckMidiVelocity")


    if (velocity == nil) then
        error("SanityCheckMidiVelocity says velocity is nil")
    end

    if not (tonumber(velocity) >= 0 and tonumber(velocity) <= 127) then
        error("SanityCheckMidiVelocity says velocity is out of bounds with the value: " .. tostring(velocity))
    end

    return tonumber(velocity)
end

function SanityCheckMidiChannel(channel)
    last_function = 987643
    -- print ("Hello from SanityCheckMidiChannel")


    if (channel == nil) then
        error("SanityCheckMidiChannel says channel is nil")
    end
    -- MIDI channels are 1-16 based on code usage
    if not (tonumber(channel) >= 1 and tonumber(channel) <= 16) then
        error("SanityCheckMidiChannel says channel is out of bounds with the value: " .. tostring(channel))
    end

    return tonumber(channel)
end

g_count_of_active_midi_on = 0  -- effectively gives a count of active note on events at the current midi step
g_count_of_active_midi_off = 0 -- effectively gives a count of active note OFF events at the current midi step



function PlayMidi()
    -- This function, which gets called every tick,
    -- loops through all 127 midi notes,
    -- and GETS the note on event that matches the current midi lane, bar and step.
    -- (which we WILL becuase we init a table with all possible lane,bar,step,note combinations.)
    -- Then we check if the note is active.
    -- Then we check if the note should be played on this particular tick.
    -- This means we can have only ONE note name (e.g. C4) per step, but it can be on any tick within the step which is kind of nice.

    last_function = 364892

    local count_of_active_midi_on = 0
    local count_of_active_midi_off = 0

    -- print ("hello from PlayMidi midi_step_count is " .. midi_step_count)

    for n = 0, 127 do
        -- print ("PlayMidi says current_midi_lane is " .. current_midi_lane .. " midi_bar_count is " .. midi_bar_count .. " midi_step_count is " .. midi_step_count .. " n is " .. n)

        local note_on_event = keyboard_midi_note_events[current_midi_lane][midi_bar_count][midi_step_count][n][1]

        if note_on_event.is_active == 1 then
            -- Turn an led on on grid_two to show there is an active note here
            -- we are interested in the first 6 notes on one step.

            -- Add bounds checking to prevent negative or out-of-bounds LED access
            local led_row = math.max(1, math.min(8, 6 - count_of_active_midi_on))
            if my_grid_two then
                my_grid_two:led(midi_step_count, led_row, note_on_event.velocity)
                my_grid_two:refresh()
            end

            count_of_active_midi_on = count_of_active_midi_on + 1


            if note_on_event.tick_count_since_step == the_current_tick_count_since_step then
                -- Can we flash the screen here or flash the new grids?

                -- Send MIDI Note ON
                SendMidiKeyboardNoteOn(n, note_on_event.velocity, SanityCheckMidiChannel(MIDI_KEYBOARD_CHANNEL))
                -- print ("I sent Midi note " .. n .. " on step " .. midi_step_count)
            else
                -- print("note_on_event.tick_count_since_step did not equal the_current_tick_count_since_step " .. note_on_event.tick_count_since_step .. " vs " .. the_current_tick_count_since_step)
            end
        else
            -- print ("note " .. n .. " is not active ")
        end

        -- Read MIDI sequence (Note OFFs)
        local note_off_event = keyboard_midi_note_events[current_midi_lane][midi_bar_count][midi_step_count][n][0]

        if note_off_event.is_active == 1 then
            count_of_active_midi_off = count_of_active_midi_off + 1
            if note_off_event.tick_count_since_step == the_current_tick_count_since_step then
                -- Send MIDI Note OFF
                --print("before SendMidiKeyboardNoteOn for note off current_midi_lane is "  .. current_midi_lane .. " midi_bar_count " .. midi_bar_count .. " midi_step_count " .. midi_step_count .. " n " .. n)
                SendMidiKeyboardNoteOn(n, 0, SanityCheckMidiChannel(MIDI_KEYBOARD_CHANNEL))
            end
        end
    end

    -- so we can track active on / off notes per step or however often we call play midi
    g_count_of_active_midi_on = count_of_active_midi_on
    g_count_of_active_midi_off = count_of_active_midi_off

    -- print ("Bye from PlayMidi count_of_active_midi_on is " .. count_of_active_midi_on .. " count_of_active_midi_off is " .. count_of_active_midi_off)
end

------------- ON TICK ontick FUNCTION - THIS IS THE MAIN TIMING LOOP - The Main Loop!---------------------------



function tick()
    while true do
        -- quick question. why is this never zero?
        -- print(" the_current_tick_count_since_step is: " .. the_current_tick_count_since_step .. " the_current_tick_count_since_start is: " .. the_current_tick_count_since_start .. " transport_is_active:  " .. tostring(transport_is_active))

        -- This is for informational purposes
        current_tempo = clock.get_tempo()

        -- Check for big differences in tempo from average
        if (math.abs(wow_average_tempo - current_tempo) > wow_threshold) then
            -- UNSTABLE WOW
            -- Track the total number of ticks we have wow since we started the clock
            total_wow_tempo_ticks = total_wow_tempo_ticks + 1

            -- Moving from stable to unstable
            if (tempo_wow_is_good == 1) then
                wow_tempo_episodes = wow_tempo_episodes + 1

                -- As soon as we are stable start a new tempo windown
                -- init_wow_window()
            end

            -- New state is unstable
            tempo_wow_is_good = 0
        else
            tempo_wow_is_good = 1
        end

        -- Check for small differences in tempo from average
        if (math.abs(flutter_average_tempo - current_tempo) > flutter_threshold) then
            -- UNSTABLE
            total_flutter_tempo_ticks = total_flutter_tempo_ticks + 1

            -- Moving from stable to unstable (previous state was stable)
            if (tempo_flutter_is_good == 1) then
                flutter_tempo_episodes = flutter_tempo_episodes + 1

                -- As soon as we are stable start a new tempo windown
                -- init_flutter_window()
            end

            -- New state is unstable
            tempo_flutter_is_good = 0
        else
            tempo_flutter_is_good = 1
        end

        if (tempo_wow_is_good == 0 or tempo_flutter_is_good == 0) then
            tempo_status_string_1 = "Current Tempo (UNSTABLE): " .. string.format("%.2f", current_tempo)
            tempo_is_stable = 0
        else
            tempo_status_string_1 = "Current Tempo: " .. string.format("%.2f", current_tempo)
            tempo_is_stable = 1
        end



        tempo_status_string_2 = "Wow Av Tempo: " .. string.format("%.2f", wow_average_tempo)
        tempo_status_string_3 = "Flutter Av Tempo: " .. string.format("%.2f", flutter_average_tempo)
        tempo_status_string_4 = "Wow Epsds: " .. wow_tempo_episodes .. " Ticks: " .. total_wow_tempo_ticks
        tempo_status_string_5 = "Flutter Epsds: " .. flutter_tempo_episodes .. " Ticks: " .. total_flutter_tempo_ticks


        if (we_have_last_daily_co2_ppm_value) then
            co2_ppm_status_string = "CO2 PPM: " .. co2_ppm_daily_latest_value
        else
            co2_ppm_status_string = "CO2 PPM: UNKOWN"
        end



        if (tempo_is_stable == 0) then
            -- print (tempo_status_string_1)
            -- print (tempo_status_string_2)
            -- print (tempo_status_string_3)
            -- print (tempo_status_string_4)
            -- print (tempo_status_string_5)
        end

        if swing_mode == 1 then
            swing_amount = 0
        else
            -- some kind of swing amount between zero and nearly 1/192
            swing_amount = (swing_mode / 18) * (1 / 480)
        end


        --print ("tick says: current_step is: " .. current_step .. " tick_count is: " .. tick_count .. " blip_count is: " .. blip_count)

        clock.sync(1 / 48) -- Run at twice 24 PPQN so the even we can send gate on (for clock) and on the odd we can send gate off.


        if transport_is_active then
            -- Every 12 ticks we want to advance the sequencer (if transport is active)



            -- Less frequently triggered gates
            if tick_count % (192 * 1) == 0 then -- At 12 ticks per step, this is every 16 steps.but this is independent of any step_count.
                clock.run(process_clock_gate, GATE_12)
                --print("tick_count is: " .. tick_count .. " GATE_12 ")
            end

            if tick_count % (192 * 2) == 0 then
                clock.run(process_clock_gate, GATE_11)
                --print("tick_count is: " .. tick_count .. " GATE_11 ")
            end

            if tick_count % (192 * 4) == 0 then
                clock.run(process_clock_gate, GATE_10)
                --print("tick_count is: " .. tick_count .. " GATE_10 ")
            end

            if tick_count % (192 * 8) == 0 then
                clock.run(process_clock_gate, GATE_9)
                --print("tick_count is: " .. tick_count .. " GATE_9 ")
            end

            if tick_count % (192 * 16) == 0 then
                clock.run(process_clock_gate, GATE_8)
                --print("tick_count is: " .. tick_count .. " GATE_8 ")
            end

            -- Note: make sure reset of tick_count is at least this otherwise we won't go in here
            if tick_count % (192 * 32) == 0 then
                clock.run(process_clock_gate, GATE_7)
                --print("tick_count is: " .. tick_count .. " GATE_7 ")
            end


            -- Safety check: only increment CO2 counter if we have valid data
            if no_of_co2_ppm_records > 0 then
                total_tick_co2_count = util.wrap(total_tick_co2_count + 1, 1, no_of_co2_ppm_records)
            end

        -- Advance step counters FIRST (before PlayMidi) to sync visual with audio
        if transport_is_active and tick_count % 12 == 0 then
            InitStepCountSinceStep()

            --  print("tick_count is: " .. tick_count .. " blip_count is: " .. blip_count)

            process_step()

            -- Advance the midi step based on tick_count mod 12.
            midi_step_count = util.wrap(midi_step_count + 1, first_step, last_step)

            if (midi_step_count == 1) then
                midi_bar_count = util.wrap(midi_bar_count + 1, MIN_BAR, MAX_BAR)
            end

            -- print ("Advanced step to: " .. midi_step_count)

            -- Advance the step for each row each_row_step
            for row = 1, TOTAL_SEQUENCE_ROWS do
                row_states[row]["current_step"] = util.wrap(row_states[row]["current_step"] + 1, 1, row_states[row]["last_step"])
                --print ("Advanced step for Row: " .. row .. " to: " .. row_states[row]["current_step"])
            end

            -- Safety check: only increment CO2 counter if we have valid data
            if no_of_co2_ppm_records > 0 then
                total_step_co2_count = util.wrap(total_step_co2_count + 1, 1, no_of_co2_ppm_records)
            end
        end

        -- Now play MIDI based on the updated step positions
        PlayMidi() -- play on every tick because we record notes to tick accuracy

        -- In clock sync, 1 refers to a quarter note so if we clock.sync(1) we will count 4 beats per bar
        -- if we clock.sync(1/4) we will count 16 beats per bar. (16 steps in the sequence)
        -- if we clock.sync(1/24) this is 24PPQN Pulses Per Quarter Note, I.e. standard MIDI clock

        -- Continue with the rest of the step processing for grid display
        if transport_is_active and tick_count % 12 == 0 then
            ------------------------------------------------------------------------

            -- Collect and display the NOTE ON events for the current step. HEREHEREHERE

            local count_of_active_midi_on = 0
            local collected_note_ons = {}

            -- Create a table and reset all the columns on the current step. TODO reset all the steps for a bar when bar changes?
            for i = 1, 8 do
                collected_note_ons[i] = {} -- create a table for each col
                if my_grid_two then
                    my_grid_two:led(midi_step_count, i, 0) -- turn off the led for the current column (we scroll left to right)
                end
            end

            -- loop through all midi note numbers note on events and if we have an active note on, collect it in our collection table
            for n = 0, 127 do
                local note_on_event = keyboard_midi_note_events[current_midi_lane][midi_bar_count][midi_step_count]
                [n][1]

                -- For each proper note on event we find,
                if note_on_event.is_active == 1 and note_on_event.velocity > 0 then
                    -- store it so we can come back to it once we've collected them.
                    count_of_active_midi_on = count_of_active_midi_on + 1
                    -- our index on the table will start at 1 and go up.
                    -- The lowest midi notes will be earlier in the table.

                    -- Initialize table entry if it doesn't exist
                    if not collected_note_ons[count_of_active_midi_on] then
                        collected_note_ons[count_of_active_midi_on] = {}
                    end

                    -- hmm
                    collected_note_ons[count_of_active_midi_on].velocity = note_on_event.velocity
                    collected_note_ons[count_of_active_midi_on].midi_note_number = n
                end
            end

            -- Turn an led on on grid_two to show there is an active note here
            -- we are interested in the first 6 notes on one step.

            if count_of_active_midi_on > 0 then
                for c = 1, count_of_active_midi_on do
                    -- Grid x,y starts from top left
                    if c <= 8 then -- show a max of 8 notes.
                        -- we might want to spread these notes out over the 8 grid notes we have.

                        -- We want the lowest note to be at the bottom of the grid
                        local y = 1 + math.abs(c - 8)
                        -- Add bounds checking for grid LED access
                        local led_row = math.max(1, math.min(8, y))

                        if my_grid_two then
                            my_grid_two:led(midi_step_count, led_row, collected_note_ons[c].velocity)
                        end

                        -- This table stores the relationship between the grid x,y and the mozart note it represents.
                        -- so we can later press the button and turn off a note in the mozart table.

                        scroll_state[midi_step_count][y] = MozartPointer:new()
                        scroll_state[midi_step_count][y].is_active = true
                        scroll_state[midi_step_count][y].current_midi_lane = current_midi_lane
                        scroll_state[midi_step_count][y].midi_bar_count = midi_bar_count
                        scroll_state[midi_step_count][y].midi_step_count = midi_step_count
                        scroll_state[midi_step_count][y].midi_note_number = collected_note_ons[c].midi_note_number

                        --print ("midi_note_number is: ")
                        --print (scroll_state[midi_step_count][count_of_active_midi_on].midi_note_number)
                        --print ("is_active: ")
                        --print (scroll_state[midi_step_count][count_of_active_midi_on].is_active)
                        --print ("midi_note_number: ")
                        --print (scroll_state[midi_step_count][count_of_active_midi_on].midi_note_number)
                    end
                end
            end

            if my_grid_two then my_grid_two:refresh() end

            -- NEXT
            ---We want to turn off a note when we click it on the grid - but the grid is scrolling.
            --so for each grid button that is lit, we must have recorded the midi_bar, the midi_step and the note number
            --then when we press it off we can use that tripple to disable the note in mozart_state

            ------------------------------------------------------

            redraw()
        end -- end step processing

            --blip_count = blip_count - 1
        end

        -- So tick_count doesn't get too big over the course of a long running session. (would end up slowing down modulus calcs?)
        tick_count = tick_count + 1
        the_current_tick_count_since_start = the_current_tick_count_since_start + 1


        wow_window_tick_position = wow_window_tick_position + 1
        wow_tempo_sum = wow_tempo_sum + current_tempo


        flutter_window_tick_position = flutter_window_tick_position + 1
        flutter_tempo_sum = flutter_tempo_sum + current_tempo


        if wow_window_tick_position == wow_window_size then --
            -- This means we calculate the average tempo over a fixed period of 192 ticks however we start the window again as soon as we have a stable tempo

            screen.clear()
            screen.move(1, 10)

            -- Don't floor because we don't want to go down one bpm if we're just under
            if wow_window_tick_position > 0 then
                wow_average_tempo = wow_tempo_sum / wow_window_tick_position
            else
                -- Fallback: use current tempo if we can't calculate average
                wow_average_tempo = current_tempo
            end

            screen.text("Average Wow Tempo" .. wow_average_tempo)

            init_wow_window()
        end


        if flutter_window_tick_position == flutter_window_size then --
            -- This means we calculate the average tempo over a fixed period of 192 ticks however we start the window again as soon as we have a stable tempo

            screen.clear()
            screen.move(1, 20)

            -- Don't floor because we don't want to go down one bpm if we're just under
            if flutter_window_tick_position > 0 then
                flutter_average_tempo = flutter_tempo_sum / flutter_window_tick_position
            else
                -- Fallback: use current tempo if we can't calculate average
                flutter_average_tempo = current_tempo
            end

            screen.text("Average Flutter Tempo" .. flutter_average_tempo)

            init_flutter_window()
        end



        if (tick_count == 192 * 64) then -- Reset so we don't have too many numbers on which we do modulus calculations
            -- This means we calculate the average tempo over a fixed period of 192 ticks

            screen.clear()

            screen.move(1, 10)
            screen.text(version_string)

            init_tick_count()
        end

        IncrementStepCountSinceStep()
    end -- end while
end   -- end on tick function

function init_tick_count()
    tick_count = 0
end

function init_midi_step_count()
    midi_step_count = 1
end

function init_midi_bar_count()
    midi_bar_count = 1
end

-- Note: This effictively gets called multiple times at boot
function greetings()
    -- presumption of success but still seems to get run multiple times.
    greetings_done = true

    screen.clear()

    screen.move(1, 10)
    screen.text(version_string)

    screen.move(1, 20)


    grid_text = "Unknown"

    if (not my_grid_one) then
        grid_text = "Grid NOT CONNECTED"
    else
        grid_text = "Grid: " .. tostring(my_grid_one.name)
    end

    screen.text(grid_text)

    screen.move(1, 30)
    if my_grid_one then
        screen.text(my_grid_one.cols .. " X " .. my_grid_one.rows)
    else
        screen.text("No Grid One Connected")
    end

    local y_position = 40


    -- local do_print_midi = false

    -- if do_print_midi then

    print("midi.devices are:")

    for key, value in pairs(midi.devices) do
        local midi_text = ""

        print(key, " -- ", value)
        for sub_key, sub_value in pairs(value) do
            print("  " .. sub_key, " -- ", sub_value)

            if sub_key == "port" then
                midi_text = midi_text .. " P" .. sub_value

                if sub_value == INITIAL_MIDI_GATES_PORT then
                    midi_text = midi_text .. "GTES"
                end

                if sub_value == INITIAL_MIDI_KEYBOARD_PORT then
                    midi_text = midi_text .. "KYBD"
                end
            end

            if sub_key == "name" then
                midi_text = midi_text .. " " .. sub_value
            end
        end

        screen.move(1, y_position)
        screen.text(midi_text)




        print("midi_text is: " .. midi_text)
        y_position = y_position + 10
        -- screen.update()
    end -- end loop of midi devices

    screen.update()
    clock.sleep(4)

    -- Show Euclidean instructions screen
    show_euclidean_instructions()

    --print("now awake")
    greetings_done = true

    -- print_audio_file_info(audio_clock_file)
end

function process_step()
    print ("hello from process_step midi_step_count is:  " .. midi_step_count)


    local ratchet_mode = 1 -- default is 1 but it will be set

    --engine.hz(400) -- just to give some audible sign for debugging timing

    if need_to_start_midi == true then
        if midi_step_count == ONE then
            --engine.hz(800) -- just to give some audible sign for debugging timing

            -- we only want to start midi clock at the right time!

            if (enable_midi_clock_out == 1) then
                print("Send MIDI Start midi_step_count is: " .. midi_step_count)
                if midi_gates_usb_device_port then
                    midi_gates_usb_device_port:start()
                else
                    print("WARNING: midi_gates_usb_device_port is nil, cannot start MIDI gates")
                end
                if midi_keyboard_usb_device_port then
                    midi_keyboard_usb_device_port:start()
                else
                    print("WARNING: midi_keyboard_usb_device_port is nil, cannot start MIDI keyboard")
                end
            else
                print("NOT Send MIDI Start (disabled) midi_step_count is: " .. midi_step_count)
            end
            run_conditional_clocks = true -- so our 24PPQN etc stays on when midi clock is on
            need_to_start_midi = false
        else
            if (enable_midi_clock_out == 1) then
                print("Waiting to MIDI Start midi_step_count is: " .. midi_step_count)
            else
                print(" MIDI Clock out disabled")
            end
        end
    end -- End check midi start

    -- For each sequence row...
    for sequence_row = 1, TOTAL_SEQUENCE_ROWS do

        print("sequence_row is: " .. sequence_row)

        -- on the current step...
        -- Prior to POLYR
        --ratchet_mode = grid_one_state[current_step][sequence_row]

        ratchet_mode = grid_one_state[row_states[sequence_row]["current_step"]][sequence_row]


        print("ratchet_mode for row and step is: " .. ratchet_mode)

        -- process step should run independently
        clock.run(process_ratchet, sequence_row, ratchet_mode)

        -- Sent appropriate midi note out as cv

        -- To quote dan_dirks, "any MIDI note number divided by 12 is how the pitch is expressed in voltage (assuming volt per octave)"
        -- https://llllllll.co/t/frequencies-and-cv-converting-back-and-forth-in-lua-math-math-math/50984

        -- Send the midi note number as CV we have previously captured (this currently sends even if the step is not active)

        -- We have 4 outputs on crow to output eurorack CV
        -- Here we check the slide and set the voltage to the pitch accordingly.
        if sequence_row >= 3 and sequence_row <= 6 then
            -- conditional_change_crow_output(current_step, sequence_row)

            conditional_change_crow_output(row_states[sequence_row]["current_step"], sequence_row)
        end
    end -- end for
    print ("bye from process_step midi_step_count is:  " .. midi_step_count)
end -- end function

function conditional_change_crow_output(current_step, sequence_row)
    crow_output = sequence_row - 2



    -- only change slew and voltage if the sequence step is active
    if grid_one_state[current_step][sequence_row] ~= 0 then
        if slide_state[current_step][sequence_row] == 1 then
            crow.output[crow_output].slew = 0.1
        else
            crow.output[crow_output].slew = 0
        end

        -- Row 3 special case for the CO2 PPM data
        if (sequence_row == 3) then
            --print("hello from row 6 total_step_co2_count is " .. total_step_co2_count)

            -- Safety check: ensure CO2 data is available and bounds are valid
            if we_have_all_daily_co2_ppm_values and co2_ppm_list and no_of_co2_ppm_records > 0 and
               total_step_co2_count >= 1 and total_step_co2_count <= no_of_co2_ppm_records and
               co2_ppm_list[total_step_co2_count] and co2_ppm_list[total_step_co2_count].the_co2_ppm_value then

                local co2_value = validate_co2_value(co2_ppm_list[total_step_co2_count].the_co2_ppm_value)
                -- Comprehensive validation using helper function
                if co2_value then
                    co2_ppm_step_offset = co2_value / 50
                    --print (co2_ppm_step_offset)
                    crow.output[crow_output].volts = co2_ppm_step_offset + (mozart_state[current_step][sequence_row] / 12)
                else
                    local raw_value = co2_ppm_list[total_step_co2_count].the_co2_ppm_value
                    print("WARNING: Invalid CO2 value at step index " .. total_step_co2_count .. " (raw: " .. tostring(raw_value) .. ", parsed: " .. tostring(co2_value) .. ")")
                    crow.output[crow_output].volts = mozart_state[current_step][sequence_row] / 12 -- fallback to normal mode
                end
            else
                print("WARNING: CO2 data not available for sequence_row 3, using fallback")
                crow.output[crow_output].volts = mozart_state[current_step][sequence_row] / 12 -- fallback to normal mode
            end
        elseif (sequence_row == 4) then
            -- Safety check: ensure CO2 data is available and bounds are valid
            if we_have_all_daily_co2_ppm_values and co2_ppm_list and no_of_co2_ppm_records > 0 and
               total_tick_co2_count >= 1 and total_tick_co2_count <= no_of_co2_ppm_records and
               co2_ppm_list[total_tick_co2_count] and co2_ppm_list[total_tick_co2_count].the_co2_ppm_value then

                local co2_value = validate_co2_value(co2_ppm_list[total_tick_co2_count].the_co2_ppm_value)
                -- Comprehensive validation using helper function
                if co2_value then
                    co2_ppm_tick_offset = co2_value / 50
                    --print (co2_ppm_tick_offset)
                    crow.output[crow_output].volts = co2_ppm_tick_offset + (mozart_state[current_step][sequence_row] / 12)
                else
                    local raw_value = co2_ppm_list[total_tick_co2_count].the_co2_ppm_value
                    print("WARNING: Invalid CO2 value at tick index " .. total_tick_co2_count .. " (raw: " .. tostring(raw_value) .. ", parsed: " .. tostring(co2_value) .. ")")
                    crow.output[crow_output].volts = mozart_state[current_step][sequence_row] / 12 -- fallback to normal mode
                end
            else
                print("WARNING: CO2 data not available for sequence_row 4, using fallback")
                crow.output[crow_output].volts = mozart_state[current_step][sequence_row] / 12 -- fallback to normal mode
            end
        else
            -- use the notes from the grid
            crow.output[crow_output].volts = mozart_state[current_step][sequence_row] / 12 -- no offset
        end
    end


    --if (sequence_row ==5) then
    --print("hello from row 5 voltage just set was " .. mozart_state[current_step][sequence_row] / 12 )
    --end
end -- end function

function process_ratchet(output, ratchet_mode)
    -- This is independent of the main clock. Thus its run with clock.run(process_ratchet, output, ratchet_mode)
    if ratchet_mode == 1 then -- could be 1 or 2 (ratchet) or...
        -- direct relation between value on grid at count of send_gates we will get

        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)
    elseif ratchet_mode == 2 then
        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)

        clock.sync(1 / 8)

        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)
    elseif ratchet_mode == 3 then
        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)

        clock.sync(1 / 12)

        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)

        clock.sync(1 / 12)

        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)
    elseif ratchet_mode == 4 then
        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)

        clock.sync(1 / 32)

        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)

        clock.sync(1 / 32)

        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)

        clock.sync(1 / 32)

        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)
    elseif ratchet_mode == 5 then
        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)

        clock.sync(1 / 8)

        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)

        clock.sync(1 / 32)

        gate_on(output)
        clock.sync(1 / 62)
        gate_off(output)

        clock.sync(1 / 32)

        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)

        -- this is a "lag" (pause before playing)
        -- This applies lag to a single step.
        -- If we want to apply swing we'd need to look at a swing setting of the track
        -- Then if the track is swung we'd need to process each step's lag differently
        -- depending on the type of swing
    elseif ratchet_mode == 6 then
        clock.sync(1 / 32)
        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)
        -- Lag
    elseif ratchet_mode == 7 then
        clock.sync(1 / 16)
        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)
        -- Lag
    elseif ratchet_mode == 8 then
        clock.sync(1 / 8)
        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)
        -- Lag
    elseif ratchet_mode == 9 then
        clock.sync(1 / 6)
        gate_on(output)
        clock.sync(1 / 64)
        gate_off(output)
    end -- end non zero
end

-- MUST be run as clock.run(process_clock_gate, output) - so syncs are independent
function process_clock_gate(output)
    -- if (enable_analog_clock_out == 1) then
    gate_on(output)
    clock.sync(1 / 64)
    gate_off(output)
    -- end
end

function gate_on(output)
    --print ("A ON LOWEST_MIDI_NOTE_NUMBER_FOR_GATE" .. LOWEST_MIDI_NOTE_NUMBER_FOR_GATE .. " MIDI_NOTE_ON_VELOCITY " .. MIDI_NOTE_ON_VELOCITY .. " sequence_row + MIDI_CHANNEL_GATES " .. sequence_row + MIDI_CHANNEL_GATES)
    if midi_gates_usb_device_port then
        midi_gates_usb_device_port:note_on(LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + output, MIDI_NOTE_ON_VELOCITY,
            MIDI_CHANNEL_GATES)
    else
        print("WARNING: midi_gates_usb_device_port is nil, cannot send gate on")
    end
end

function gate_off(output)
    --print ("A OFF LOWEST_MIDI_NOTE_NUMBER_FOR_GATE" .. LOWEST_MIDI_NOTE_NUMBER_FOR_GATE .. " MIDI_NOTE_OFF_VELOCITY " .. MIDI_NOTE_OFF_VELOCITY .. " sequence_row + MIDI_CHANNEL_GATES " .. sequence_row + MIDI_CHANNEL_GATES)
    if midi_gates_usb_device_port then
        midi_gates_usb_device_port:note_off(LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + output, MIDI_NOTE_OFF_VELOCITY,
            MIDI_CHANNEL_GATES)
    else
        print("WARNING: midi_gates_usb_device_port is nil, cannot send gate off")
    end
end

function clock.transport.start() -- transport start
    -- This function is maybe called
    -- Via the system when midi start is detected. Confirmed.

    -- Note: See right button for other actions.

    print("====================== transport.start says Hello ========================")

    init_tick_count()
    InitStepCountSinceStep()
    init_midi_bar_count()

    the_current_tick_count_since_start = 0
    screen.clear()



    init_wow_window()

    init_flutter_window()

    init_wow_and_flutter_counters()

    screen.move(1, 63)
    screen.text("Transport Start")
    screen.update()

    print("end of transport start")

    if (transport_is_active == true) then -- see right button
        -- init_keyboard_midi_note_events()
    end

    transport_is_active = true
end

function request_midi_start()
    print("request_midi_start")
    need_to_start_midi = true
end

function clock.transport.stop() -- transport stop
    -- This function is maybe called
    -- 1) Via code attached to the Norns Right Button
    -- 2) Via the system when midi stop is detected. ? check this.


    print("================= transport.stop says Hello =======================")

    print("total_flutter_tempo_ticks since last start: " .. total_flutter_tempo_ticks)
    print("flutter_tempo_episodes since last start: " .. flutter_tempo_episodes)

    reset_all_sequence_counters()



    refresh_grid_and_screen()


    --  screen.clear()

    transport_is_active = false
    --screen.move(80,80)
    --screen.text("Transport STOP")
    --screen.update()

    --clock.sleep(5)


    display_tempo_status()
end

function request_midi_stop()
    print("request_midi_stop")
    need_to_start_midi = false
    -- can stop the midi clock at any time.

    if (enable_midi_clock_out == 1) then
        if midi_gates_usb_device_port then
            midi_gates_usb_device_port:stop()
        end
        if midi_keyboard_usb_device_port then
            midi_keyboard_usb_device_port:stop()
        end
    end


    run_conditional_clocks = false
end

function enc(n, d)
    if n == 3 then
        params:delta("clock_tempo", d)
    end
end

-- Norns (Shield) key presses - (This is not the monome grid )
function key(n, z)
    print("key pressed.  n:" .. n .. " z:" .. z)

    -- since MIDI and Link offer their own start/stop messages,
    -- we'll only need to manually start if using internal or crow clock sources:
    -- if params:string("clock_source") == "internal" then

    -- STOP left button pressed
    if n == 2 and z == 1 then
        clock.transport.stop()

        reset_all_sequence_counters()



        screen_dirty = true
    end

    -- START Right button pressed
    if n == 3 and z == 1 then
        if not transport_is_active then
            clock.transport.start()
            screen.move(1, 63)
            screen.text("Transport Start")
            screen.update()
        else
            DisableAndTurnOffActiveKeyboardMidiNotes(4) -- only want to clear this when we are running.
            screen.move(1, 63)
            screen.text("Cleared Active MIDI")
            screen.update()
        end

        screen_dirty = true
    end

    -- end
end

function grid_button_function_name(x, y)
    local ret = "NOT_FOUND"

    --print ("-------------------- BEGIN -----------------")


    -- BUTTONS is a table of tables
    -- Loop through each button definition
    for key, value in pairs(BUTTONS) do
        local found = 0
        local name = "NAME_NOT_FOUND"
        --print(key, " -- ", value)
        -- Each inner table contains an "x" (grid column), "y" (grid row) and "name"
        -- "name" describes the button function
        -- Loop through the definition trying to find the x and y requested so we can return the "name"
        -- We must match both x and y so we only use the name if found == 2
        for sub_key, sub_value in pairs(value) do
            --print("  " .. sub_key, " -- ", sub_value)

            if sub_key == "x" and sub_value == x then
                --print ("x found")
                found = found + 1
            end

            if sub_key == "y" and sub_value == y then
                --print ("y found")
                found = found + 1
            end

            if sub_key == "name" then
                name = sub_value
            end
        end -- end sub loop

        if found == 2 then
            ret = name
            --print ("will BREAK and ret is " .. ret)
            break -- break out of inner loop
        end

        if found == 2 then
            break -- break out of outer loop
        end
    end -- end outer loop

    -- print ("grid_button_function_name says Bye. I will return: " .. ret)

    if ret == "NOT_FOUND" then
        print("x was: " .. x .. " y was: " .. y)
    end

    return ret
end -- end function definition

function init()
    print("Hello from init. Version is " .. version)


    print("#### Here are the grids ####")

    for id, dev in pairs(grid.devices) do
        print("Grid ID: " .. id .. ", Device: " .. tostring(dev))
    end


    -- Last In First Out (LIFO) tables for Undo and Redo of grid state functionality
    undo_grid_lifo = {}
    redo_grid_lifo = {}

    undo_mozart_lifo = {}
    redo_mozart_lifo = {}

    print("!!! before init tables !!!")

    print("before init_row_states_table")
    init_row_states_table()


    print("before init_grid_one_state_table")
    init_grid_one_state_table()


    print("before init_mozart_state_table")
    init_mozart_state_table()

    print("before init_scroll_state_table")
    init_scroll_state_table()

    print("before init_slide_state_table")
    init_slide_state_table()


    print("before init_held_state_table")
    init_held_state_table()

    -- Verify critical variables are initialized before proceeding
    print("Verifying initialization state...")
    if row_states == nil then
        print("ERROR: row_states is still nil after table initialization!")
        row_states = create_row_states()
        print("Created row_states")
    end

    if grid_one_state == nil then
        print("ERROR: grid_one_state is still nil after table initialization!")
    end

    if mozart_state == nil then
        print("ERROR: mozart_state is still nil after table initialization!")
    end

    print("Initialization verification complete")

    reset_all_sequence_counters()

    refresh_grid_and_screen()

    print("before init_keyboard_midi_note_events")
    init_keyboard_midi_note_events()

    print("hello")
    -- my_grid_one:all(2)
    if my_grid_one then my_grid_one:refresh() end -- refresh the LEDs
    if my_grid_two then my_grid_two:refresh() end


    print("my_grid_one follows: ")
    print(my_grid_one)
    if my_grid_one then
        print("my_grid_one.name is: " .. my_grid_one.name)
        print("my_grid_one.cols is: " .. my_grid_one.cols)
        print("my_grid_one.rows is: " .. my_grid_one.rows)
    else
        print("my_grid_one is nil - no grid connected")
    end


    --print ("midi.devices are:")
    --for key, value in pairs(midi.devices) do
    --  print(key, " -- ", value)
    --  for sub_key, sub_value in pairs(value) do
    --    print("  " .. sub_key, " -- ", sub_value)
    --  end
    --end -- end loop of midi devices


    -- Set the starting tempo. Can be changed with right knob
    -- TODO store and retreive this
    params:set("clock_tempo", 80)

    --midi_start_on_bar_id = clock.run(midi_start_on_bar)

    -- print ("******START***************")
    -- print(grid_button_function_name (11,8))
    -- print(grid_button_function_name (2,1))
    -- print ("========END============")


    --crow.output[1].scale = {0,7,2,9}

    current_tempo = clock.get_tempo()
    flutter_average_tempo = clock.get_tempo() -- just for initial value
    wow_average_tempo = clock.get_tempo() -- initialize with current tempo to avoid false instability detection

    init_wow_and_flutter_counters()
    init_wow_window()
    init_flutter_window()



    print("init says: Starting main sequencer timing called tick.  the_current_tick_count_since_step is: " ..
    the_current_tick_count_since_step)

    -- Display current Euclidean system status on startup
    print("Complete Euclidean Sequencer Integration Initialized")
    show_euclidean_events_info()
    print("Euclidean Controls: LENGTH (pos 6) → EVENTS (pos 7) → ROTATION (pos 5)")
    print("All ARM_EUCLIDIAN operations work on the selected row")
    print("Press ARM_EUCLIDIAN_LENGTH_BUTTON + column/row to set sequence length for that row")
    print("Press ARM_EUCLIDIAN_EVENTS_BUTTON + column/row to set event count for that row")
    print("Press ARM_EUCLIDIAN_ROTATION_BUTTON + column/row to generate rotated pattern for that row")

    -- Test MIDI output on startup
    clock.run(test_midi_output)

    clock.run(tick) -- start the sequencer
end   -- end init

-- Periodically check if we need to save the grid state to file.
-- TODO - if we're not saving this when running then might as well just save it when we stop (rather than have a loop)
clock.run(function()
    while true do
        clock.sleep(5)
        -- TODO fix bug here, we only save table if grid_one has changed.
        if (grids_are_dirty == true) then
            if (transport_is_active == false) then -- only save if we're stopped. (not sure we really need this) for sure we don't want to write to disk when playing
                Tab.save(grid_one_state, GRID_ONE_STATE_FILE)

                -- for now do mozart at the same time.
                Tab.save(mozart_state, MOZART_STATE_FILE)

                -- for now do mozart at the same time.
                Tab.save(slide_state, SLIDE_STATE_FILE)


                grids_are_dirty = false

                print("I saved tables.")

                screen.move(SCREEN_INFO_X, SCREEN_INFO_Y)
                screen.text("I saved tables.")
                screen.update()
            end
        end
    end
end)




function load_grid_one_state()
    grid_one_state = Tab.load(GRID_ONE_STATE_FILE)
    -- NOTE: get_tally serves to check the table is at least kind of OK.
    -- if error pcall will return false which makes us create the table
    print(get_tally(grid_one_state))
    return grid_one_state
end

function load_mozart_state()
    mozart_state = Tab.load(MOZART_STATE_FILE)
    -- NOTE: get_tally serves to check the table is at least kind of OK.
    print(get_tally(mozart_state))
    return mozart_state
end

function load_scroll_state()
    scroll_state = Tab.load(SCROLL_STATE_FILE)
    -- NOTE: get_tally serves to check the table is at least kind of OK.
    print(get_tally(scroll_state))
    return scroll_state
end

function load_slide_state()
    slide_state = Tab.load(SLIDE_STATE_FILE)
    -- NOTE: get_tally serves to check the table is at least kind of OK.
    print(get_tally(scroll_state))
    return scroll_state
end

-- a general grid. This is used for grid, mozart, slide etc.
function create_a_grid(is_scroll_in)
    local is_scroll = is_scroll_in or false
    local local_grid = {}

    local_grid["id"] = math.random(1, 99999999999999) -- an ID for debugging purposes

    for col = 1, COLS do
        local_grid[col] = {}        -- create a table for each col
        for row = 1, ROWS do
            if (is_scroll == true) then -- If we are creating a scroll table, each entry points to a MozartPointer so we can manipulate mozart_state
                local_grid[col][row] = MozartPointer:new()
            else
                local_grid[col][row] = 0
            end
        end
    end
    return local_grid
end

function init_grid_one_state_table()
    print("Hello from init_grid_one_state_table")

    -- Try to load the table
    local success, err = pcall(load_grid_one_state) -- note grid_one_state is loaded into a global

    if success then
        print("load grid state seems ok. grid_one_state is:")
        print(grid_one_state)
        print(get_tally(grid_one_state))
    else
        print("Seems we got an error - setting grid_one_state to nil so we will create it and save it: ")
        print(err)
        grid_one_state = nil
    end

    -- if it doesn't exist
    if grid_one_state == nil then
        print("No table, I will generate a structure and save that")

        grid_one_state = create_a_grid()

        Tab.save(grid_one_state, GRID_ONE_STATE_FILE)
        grid_one_state = Tab.load(GRID_ONE_STATE_FILE)
    else
        print("I already have a grid_one_state table, no need to generate one")
    end

    -- We want to make sure row 8 are all off.
    -- Note: row 7 may have kind of dual function but 8 is all control.
    for y = TOTAL_SEQUENCE_ROWS + 1, 8 do
        for x = 1, 16 do
            print("turn off x:" .. x .. " y:" .. y)
            unconditional_set_grid_non_seq_button(x, y, 0)
        end
    end


    print("grid tally is: " .. get_tally(grid_one_state))

    print("clock.get_tempo() is: " .. clock.get_tempo())




    -- Push Undo so we can get back to initial state
    push_grid_undo()


    print("Bye from init_grid_one_state_table")
end -- end init_grid_one_state_table

----

function init_mozart_state_table()
    print("Hello from init_mozart_state_table")

    -- Try to load the table
    local success, err = pcall(load_mozart_state) -- note mozart_state is loaded into a global

    if success then
        print("load mozart state seems ok. mozart_state is:")
        print(mozart_state)
        print(get_tally(mozart_state))
    else
        print("Seems we got an error - setting mozart_state to nil so we will create it and save it: ")
        print(err)
        mozart_state = nil
    end

    -- if it doesn't exist
    if mozart_state == nil then
        print("No table, I will generate a structure and save that")

        mozart_state = create_a_grid()
        Tab.save(mozart_state, MOZART_STATE_FILE)
        mozart_state = Tab.load(MOZART_STATE_FILE)
    else
        print("I already have a mozart_state table, no need to generate one")
    end

    print("tally is: " .. get_tally(mozart_state))

    print("clock.get_tempo() is: " .. clock.get_tempo())

    -- Push Undo so we can get back to initial state
    push_mozart_undo()

    print("Bye from init_mozart_state_table")
end -- end init_mozart_state_table

function init_scroll_state_table()
    print("Hello from init_scroll_state_table")

    -- Try to load the table
    local success, err = pcall(load_scroll_state) -- note scroll_state is loaded into a global

    if success then
        print("load scroll state seems ok. scroll_state is:")
        print(scroll_state)
        --print (get_tally(scroll_state)) -- problematic here
    else
        print("Seems we got an error - setting scroll_state to nil so we will create it and save it: ")
        print(err)
        scroll_state = nil
    end

    -- if it doesn't exist
    if scroll_state == nil then
        print("No table, I will generate a structure and save that")

        scroll_state = create_a_grid(true)
        Tab.save(scroll_state, SCROLL_STATE_FILE)
        scroll_state = Tab.load(SCROLL_STATE_FILE)
    else
        print("I already have a scroll_state table, no need to generate one")
    end

    -- print ("tally is: " .. get_tally(scroll_state))


    print("Bye from init_scroll_state_table")
end -- end init_scroll_state_table

function init_slide_state_table()
    print("Hello from init_slide_state_table")

    -- Try to load the table
    local success, err = pcall(load_slide_state)

    if success then
        print("load slide state seems ok. slide_state is:")
        print(slide_state)
        print(get_tally(slide_state))
    else
        print("Seems we got an error - setting slide_state to nil so we will create it and save it: ")
        print(err)
        slide_state = nil
    end

    -- if it doesn't exist
    if slide_state == nil then
        print("No table, I will generate a structure and save that")

        slide_state = create_a_grid()



        Tab.save(slide_state, SLIDE_STATE_FILE)
        slide_state = Tab.load(SLIDE_STATE_FILE)
    else
        print("I already have a slide_state table, no need to generate one")
    end

    print("tally is: " .. get_tally(slide_state))

    print("clock.get_tempo() is: " .. clock.get_tempo())





    print("Bye from init_slide_state_table")
end -- end init_slide_state_table

--------------

function init_held_state_table()
    print("Hello from init_held_state_table")

    -- Don't want to load or save - always create new
    held_state = create_a_grid()
    print("Bye from init_held_state_table")
end -- end init_held_state_table

--------------
-----

function push_grid_undo()
    --print("push_grid_undo says hello. Store Undo LIFO")
    -- TODO check memory / count of states? - if this gets very large, truncate from the other side

    -- When we push to the undo_grid_lifo, we want to *copy* the grid_one_state (not reference) so that any subsequent changes to grid_one_state are not saved on the undo_grid_lifo
    -- Inserts in the last position of the table (push)
    table.insert(undo_grid_lifo, get_copy_of_grid(grid_one_state))

    --print ("undo_grid_lifo size is: ".. lifo_size(undo_grid_lifo))
end

function push_mozart_undo()
    table.insert(undo_mozart_lifo, get_copy_of_grid(mozart_state))
end

function pop_grid_undo()
    if lifo_size(undo_grid_lifo) > 1 then
        -- 2) Pop from the undo_grid_lifo to the current state.
        -- Removes from the last element of the table (pop)
        local undo_state = table.remove(undo_grid_lifo)

        grid_one_state = get_copy_of_grid(undo_state)

        --print ("undo_grid_lifo size is: ".. lifo_size(undo_grid_lifo))

        -- Thus if A through G are all the states we've seen, and E is the current state, we'd have the following:

        --    ABCDEFG
        --        *
        -- grid_one_state: E
        --
        -- undo_grid_lifo       redo_grid_lifo
        --    D                F
        --    C                G
        --    B
        --    A
    else
        print("Not poping last undo_grid_lifo because its size is 1 or less ")
    end
end

function pop_mozart_undo()
    if lifo_size(undo_mozart_lifo) > 1 then
        local undo_state = table.remove(undo_mozart_lifo)
        mozart_state = get_copy_of_grid(undo_state)
    else
        print("Not poping last undo_mozart_lifo because its size is 1 or less ")
    end
end

function push_grid_redo()
    -- 1) Push the current state to the redo_grid_lifo so we can get back to it.
    -- Similarly we want to *copy* the grid_one_state (not reference)
    -- so any subsequent changes to the grid_one_state are not reflected in the redo_grid_lifo
    table.insert(redo_grid_lifo, get_copy_of_grid(grid_one_state))

    --print ("redo_grid_lifo size is: ".. lifo_size(redo_grid_lifo))
end

function push_mozart_redo()
    table.insert(redo_mozart_lifo, get_copy_of_grid(mozart_state))
end

function pop_grid_redo()
    if lifo_size(redo_grid_lifo) > 1 then
        -- 2) Pop the redo_grid_lifo into the current state.

        -- TODO need to copy this?
        local redo_state = table.remove(redo_grid_lifo)
        grid_one_state = get_copy_of_grid(redo_state)

        --print ("redo_grid_lifo size is: ".. lifo_size(redo_grid_lifo))
    else
        print("Not poping last redo_grid_lifo because its size is 1 or less ")
    end
end

function pop_mozart_redo()
    if lifo_size(redo_mozart_lifo) > 1 then
        local redo_state = table.remove(redo_mozart_lifo)
        mozart_state = get_copy_of_grid(redo_state)
    else
        print("Not poping last redo_mozart_lifo because its size is 1 or less ")
    end
end

function have_held()
    if held_x ~= 0 and held_y ~= 0 then
        print("have_held is true")
        return true
    else
        print("have_held is false")
        return false
    end
end

function unconditional_set_mozart(x, y, midi_note_number, set_grid_on)
    print("unconditional_set_mozart says: Got x: " .. x .. " y: " .. y .. " midi_note_number: " .. midi_note_number)

    if midi_note_number < 0 then
        midi_note_number = 0
    end

    if midi_note_number > 127 then
        midi_note_number = 127
    end

    mozart_state[x][y] = midi_note_number -- Note we don't have any note off

    last_mozart_value = midi_note_number

    slide_state[x][y] = 0 -- this is so we can always hear our change i.e. the pitch is not masked by a slide

    -- set the button on
    if set_grid_on == 1 then
        unconditional_set_grid(x, y, 1)
    end

    grids_are_dirty = true

    print("unconditional_set_mozart says: Just set x: " .. x .. " y: " .. y .. " to midi note: " .. midi_note_number)
end

function unconditional_set_grid_non_seq_button(x, y, integer)
    if y > TOTAL_SEQUENCE_ROWS then
        if integer < 0 then
            integer = 0
        end

        if integer > 1 then
            integer = 1
        end

        grid_one_state[x][y] = integer
    else
        print("Error: unconditional_set_grid_non_seq_button will not set state of sequence button ")
    end
end

function unconditional_set_grid(x, y, integer)
    if y >= 0 and y <= TOTAL_SEQUENCE_ROWS then
        if integer < 0 then
            integer = 0
        end

        if integer > 9 then
            integer = 9
        end

        grid_one_state[x][y] = integer

        last_x = x
        last_y = y
        last_grid_value = integer
    else
        print("Error: unconditional_set_grid will not set state of non sequence button ")
    end
end

function randomize_grid(x, y)
    -- x and y should be the button pressed

    -- Depending on the button pressed, we can randomised in two different ways

    -- 1) (column 1-8 on the row we want to change) we make the pattern more or less dense.
    -- 2) (column 9-16 on the row we want to change) we randomise some of the steps (more to the right less to the left)


    -- i.e.
    -- <- sparse RANDOM PATTERN CREATION dense -> <- low chance of change - PATTERN CHANGE - high chance of change ->


    -- if x >= 1 and x <= 8 then

    --   -- RANDOM PATTERN CREATION

    --   for j = 1, 16 do
    --     -- on the current step...
    --     -- chance of that step becoming 1 (on) (higher chance if we pressed button 8)
    --     chance = math.random(1, 9 - x)
    --     if chance == 1 then
    --       -- random_grid_value = math.random(0, 1)
    --       unconditional_set_grid(j, y, 1)
    --     else
    --       unconditional_set_grid(j, y, 0)
    --     end
    --   end


    -- else

    -- PATTERN CHANGE  we want to loop through all steps (columns) (i) and set them
    for i = 1, 16 do
        -- on the current step...
        -- if we pressed a key on the right of the grid we have a high probability of chance being 1.
        -- i.e. key on left less chance the sequence will change. key on right, high chance it will change
        -- x will have the value 9 - 16
        chance = math.random(1, 17 - x)
        if chance == 1 then
            random_grid_value = math.random(0, 1)
            unconditional_set_grid(i, y, random_grid_value)
        end
    end

    --end
end

function randomize_mozart(x, y)
    -- x and y should be the button pressed
    -- however we want to loop through all columns (j) and set them
    for j = 1, 16 do
        -- chance = math.random(1, 5)
        chance = math.random(1, 17 - x)
        if chance == 1 then
            -- on the current step...
            random_mozart_value = math.random(MOZART_BASE_MIDI_NOTE, MOZART_BASE_MIDI_NOTE + MOZART_RANDOM_MAX_DELTA)
            unconditional_set_mozart(j, y, random_mozart_value, 0) -- we don't want to also set the grid in this case.
            -- TODO instead of just choosing a random note for each step, we could choose a note near to the note currently used on this step?
            -- So each time we press randomise we would diverge from the current notes
        end
    end
end

-- Euclidean rhythm generator using Bresenham's line algorithm
-- events: number of beats/events in the pattern
-- length: total number of steps in the pattern
-- rotation: rotate the pattern by this many steps (optional, default 0)
function generate_euclidean_rhythm(events, length, rotation)
    rotation = rotation or 0

    -- Sanity checks
    if events < 0 then events = 0 end
    if events > length then events = length end
    if length <= 0 then return {} end

    local pattern = {}

    -- Initialize pattern with all zeros
    for i = 1, length do
        pattern[i] = 0
    end

    -- If no events requested, return empty pattern
    if events == 0 then
        return pattern
    end

    -- Use Bresenham's line algorithm to distribute events evenly
    local slope = events / length
    local bucket = 0

    for i = 1, length do
        bucket = bucket + slope
        if bucket >= 1 then
            pattern[i] = 1
            bucket = bucket - 1
        end
    end

    -- Apply rotation if specified
    if rotation ~= 0 then
        local rotated_pattern = {}
        for i = 1, length do
            local new_index = ((i - 1 + rotation) % length) + 1
            rotated_pattern[new_index] = pattern[i]
        end
        pattern = rotated_pattern
    end

    return pattern
end

-- Apply Euclidean rhythm to a grid row
-- row: which row to apply the pattern to (1-8)
-- events: number of beats in the pattern
-- length: total steps (defaults to 16)
-- rotation: rotate pattern by this many steps
function apply_euclidean_to_row(row, events, length, rotation)
    length = length or 16
    rotation = rotation or 0

    -- Generate the Euclidean rhythm
    local pattern = generate_euclidean_rhythm(events, length, rotation)

    -- Apply the pattern to the grid row
    for step = 1, math.min(length, 16) do -- Limit to 16 steps max
        if pattern[step] then
            unconditional_set_grid(step, row, pattern[step])
        end
    end

    print("Applied Euclidean rhythm: " .. events .. "/" .. length .. " to row " .. row)
end

-- Create common Euclidean rhythm presets
function euclidean_preset(preset_number, row)
    if preset_number == 1 then
        -- Classic 3/8 tresillo pattern
        apply_euclidean_to_row(row, 3, 8, 0)
    elseif preset_number == 2 then
        -- 5/8 pattern
        apply_euclidean_to_row(row, 5, 8, 0)
    elseif preset_number == 3 then
        -- 3/4 waltz-like pattern extended to 16 steps
        apply_euclidean_to_row(row, 6, 16, 0)
    elseif preset_number == 4 then
        -- 5/12 pattern
        apply_euclidean_to_row(row, 5, 12, 0)
    elseif preset_number == 5 then
        -- 7/16 complex pattern
        apply_euclidean_to_row(row, 7, 16, 0)
    elseif preset_number == 6 then
        -- 9/16 dense pattern
        apply_euclidean_to_row(row, 9, 16, 0)
    elseif preset_number == 7 then
        -- 2/5 sparse pattern extended to 15 steps
        apply_euclidean_to_row(row, 6, 15, 0)
    elseif preset_number == 8 then
        -- 4/7 pattern extended to 14 steps
        apply_euclidean_to_row(row, 8, 14, 0)
    else
        -- Default: simple 4/4 pattern
        apply_euclidean_to_row(row, 4, 16, 0)
    end
end

-- Generate Euclidean rhythm for a row using its last_step setting
-- row: which row to apply the pattern to (1-8)
-- events: number of beats/events in the pattern
-- rotation: rotate pattern by this many steps (optional, default 0)
function apply_euclidean_to_row_with_length(row, events, rotation)
    rotation = rotation or 0
    local length = row_states[row]["last_step"]

    -- Generate the Euclidean rhythm using the row's length
    local pattern = generate_euclidean_rhythm(events, length, rotation)

    -- Clear the row first
    for step = 1, 16 do
        unconditional_set_grid(step, row, 0)
    end

    -- Apply the pattern to the grid row
    for step = 1, math.min(length, 16) do -- Limit to 16 steps max
        if pattern[step] then
            unconditional_set_grid(step, row, pattern[step])
        end
    end

    print("Applied Euclidean rhythm: " .. events .. "/" .. length .. " to row " .. row)
end

-- Complete Euclidean Sequencer Integration
-- ========================================
-- This system provides full control over Euclidean rhythm generation using three dedicated buttons:
--
-- ARM_EUCLIDIAN_LENGTH_BUTTON (Position 6): Sets the sequence length (1-16 steps)
-- ARM_EUCLIDIAN_EVENTS_BUTTON (Position 5): Sets the number of events/beats (1-16 events)
-- ARM_EUCLIDIAN_ROTATION_BUTTON (Position 7): Sets rotation and generates the pattern
--
-- Complete Workflow:
-- 1. Press ARM_EUCLIDIAN_LENGTH_BUTTON + grid position to set sequence length for that row
--    - Column (x) = length (1-16), Row (y) = target sequence row (1-8)
-- 2. Press ARM_EUCLIDIAN_EVENTS_BUTTON + grid position to set event count for that row
--    - Column (x) = events (1-16), Row (y) = target sequence row (1-8)
-- 3. Press ARM_EUCLIDIAN_ROTATION_BUTTON + grid position to generate pattern with rotation for that row
--    - Column (x) = rotation amount (0-15 steps), Row (y) = target sequence row (1-8)
--
-- Example: Create a 5/8 pattern rotated by 2 steps on row 3:
-- 1. ARM_EUCLIDIAN_LENGTH_BUTTON + column 8, row 3 (sets length to 8 for row 3)
-- 2. ARM_EUCLIDIAN_EVENTS_BUTTON + column 5, row 3 (sets events to 5 for row 3)
-- 3. ARM_EUCLIDIAN_ROTATION_BUTTON + column 3, row 3 (generates 5/8 pattern, rotated by 2 for row 3)
--
-- Benefits:
-- - Direct control over all Euclidean parameters (length, events, rotation)
-- - Visual feedback on event count changes
-- - Automatic pattern generation and application
-- - Maintains sequence first_step setting for playback
--
-- Advanced Euclidean generation with ARM_EUCLIDIAN_ROTATION integration
function generate_euclidean_with_rotation(row, rotation_step)
    local sequence_length = row_states[row]["last_step"]
    local rotation = rotation_step - 1  -- Convert to 0-based rotation
    local events = euclidean_events_count  -- Use the globally set event count

    -- Ensure events doesn't exceed sequence length
    events = math.min(events, sequence_length)

    -- Generate and apply the Euclidean pattern
    apply_euclidean_to_row_with_length(row, events, rotation)

    print("ARM_EUCLIDIAN_ROTATION Euclidean: " .. events .. "/" .. sequence_length ..
          " rotation:" .. rotation .. " row:" .. row)

    return events, rotation
end

-- Global variable to store Euclidean event count
euclidean_events_count = 4  -- Default to 4 events

-- Function to set Euclidean event count for specific row (called when ARM_EUCLIDIAN_EVENTS_BUTTON + grid position pressed)
function set_euclidean_events(events, row)
    -- Save state for undo before making changes
    push_grid_undo()

    -- Clamp events between 1 and 16
    events = math.max(1, math.min(16, events))
    euclidean_events_count = events
    print("Euclidean events count set to: " .. events .. "/16 for row " .. row)

    -- Auto-generate Euclidean pattern for the specific row with current settings
    local sequence_length = row_states[row]["last_step"]
    apply_euclidean_to_row_with_length(row, events, 0)
    print("ARM_EUCLIDIAN_EVENTS: Generated " .. events .. "/" .. sequence_length .. " Euclidean pattern for row " .. row)

    -- Mark grids as dirty so they get saved
    grids_are_dirty = true

    -- Visual feedback: briefly flash the event count on row 8
    flash_event_count_on_grid(events)

    return events
end

-- Function to get current Euclidean event count
function get_euclidean_events()
    return euclidean_events_count
end

-- Function to display current event count info
function show_euclidean_events_info()
    local info = "Euclidean events: " .. euclidean_events_count .. "/16"
    print(info)
    return info
end

-- Visual feedback function to show event count on the grid
function flash_event_count_on_grid(events)
    if not my_grid_one then return end

    -- Clear row 8 briefly
    for i = 1, 16 do
        my_grid_one:led(i, 8, 0)
    end

    -- Light up LEDs from 1 to events count
    for i = 1, math.min(events, 16) do
        my_grid_one:led(i, 8, 15) -- Full brightness
    end

    -- Send the grid update
    my_grid_one:refresh()

    -- Schedule to restore normal row 8 display after a brief delay
    clock.run(function()
        clock.sleep(0.5) -- Flash for 0.5 seconds
        restore_row8_display()
    end)
end

-- Function to restore normal row 8 button display
function restore_row8_display()
    if not my_grid_one then return end

    -- Restore all row 8 buttons to their normal state
    for _, button in ipairs(BUTTONS) do
        if button.y == 8 then
            local brightness = 0
            -- Check if this button is currently armed
            if (button.name == arm_control) then
                brightness = 15
            else
                brightness = 4
            end
            my_grid_one:led(button.x, button.y, brightness)
        end
    end

    my_grid_one:refresh()
end

-- Manual Euclidean pattern application for advanced users
-- Allows direct specification of all parameters
function apply_manual_euclidean(row, events, length, rotation, strategy_name)
    if row < 1 or row > 8 then
        print("ERROR: Row must be 1-8, got: " .. tostring(row))
        return false
    end

    length = length or row_states[row]["last_step"]
    rotation = rotation or 0
    events = events or 1
    strategy_name = strategy_name or "manual"

    -- Validate parameters
    if events > length then
        print("WARNING: Events (" .. events .. ") > length (" .. length .. "), capping events")
        events = length
    end

    if events < 0 then events = 0 end
    if length <= 0 then
        print("ERROR: Length must be positive, got: " .. tostring(length))
        return false
    end

    -- Generate and apply the pattern
    local pattern = generate_euclidean_rhythm(events, length, rotation)

    -- Clear the row first
    for step = 1, 16 do
        unconditional_set_grid(step, row, 0)
    end

    -- Apply the pattern
    for step = 1, math.min(length, 16) do
        if pattern[step] then
            unconditional_set_grid(step, row, pattern[step])
        end
    end

    print("Manual Euclidean applied: " .. events .. "/" .. length ..
          " rotation:" .. rotation .. " strategy:" .. strategy_name .. " row:" .. row)

    return true
end

-- Convenience functions for common Euclidean patterns using current event count
function apply_euclidean_with_current_events(row, rotation, name)
    rotation = rotation or 0
    name = name or "pattern"
    local events = get_euclidean_events()
    local length = row_states[row]["last_step"]
    return apply_manual_euclidean(row, events, length, rotation, name)
end

-- Quick preset functions for ARM_EUCLIDIAN_EVENTS_BUTTON event counts
function set_euclidean_sparse()
    return set_euclidean_events(2)  -- 2 events
end

function set_euclidean_light()
    return set_euclidean_events(4)  -- 4 events
end

function set_euclidean_medium()
    return set_euclidean_events(6)  -- 6 events
end

function set_euclidean_dense()
    return set_euclidean_events(9)  -- 9 events
end

function set_euclidean_max()
    return set_euclidean_events(16) -- 16 events
end

-- Classic pattern presets (maintain original event counts)
function apply_euclidean_kick(row)
    -- Classic 4-on-the-floor pattern
    return apply_manual_euclidean(row, 4, 16, 0, "kick")
end

function apply_euclidean_snare(row)
    -- Backbeat snare pattern
    return apply_manual_euclidean(row, 2, 8, 2, "snare")
end

function apply_euclidean_hihat(row)
    -- Dense hi-hat pattern
    return apply_manual_euclidean(row, 7, 16, 1, "hihat")
end

function apply_euclidean_tresillo(row)
    -- Classic 3/8 tresillo pattern
    return apply_manual_euclidean(row, 3, 8, 0, "tresillo")
end

-- probably not used (but does get called becuase lots of prints)
midi_gates_usb_device_port.event = function(data)
    -- print("---------------------- midi_gates_usb_device_port IN ---------------------------------------")
end



captured_midi_note_in = -1


----

--  m = midi.connect() -- Connect to the first available MIDI device

-- 2. Set Up an Event Handler

-- You need to define a function that will be triggered when a MIDI event (like a Note On) is received:

-- m.event = function(data)
--   local msg = midi.to_msg(data) -- Convert raw MIDI data to a structured table

--   if msg.type == "note_on" then
--     print("Note: " .. msg.note .. " Velocity: " .. msg.vel)
--   elseif msg.type == "note_off" then
--     print("Note Off: " .. msg.note)
--   end
-- end



------


-- On MIDI note receive -  on midi note input - on midi in
-- Capture MIDI IN
midi_keyboard_usb_device_port.event = function(data)
    --  print("Got a midi_keyboard_usb_device_port.event. The data[1] is: " .. data[1] .. " data[2] is: " .. data[2] .. " data[3]: is " .. data[3])

    -- print("Got a midi_keyboard_usb_device_port.event. The data[1] is: " .. data[1] .. " the_current_tick_count_since_start is: " .. the_current_tick_count_since_start .. " transport_is_active:  " .. tostring(transport_is_active))


    if data[1] == 254 and data[2] == nil and data[3] == nil then
        -- Do nothing! Filter out Active Sensing messages from Yamaha keyboard.
    else
        print("Got a (noteish) midi_keyboard_usb_device_port.event. The data[1] is: " ..
        data[1] ..
        " the_current_tick_count_since_start is: " ..
        the_current_tick_count_since_start .. " transport_is_active:  " .. tostring(transport_is_active))



        -- TODO MIGHT BE BETTER TO USE THIS WHITE LIST INSTEAD OF BLACK LIST ABOVE.
        -- If NOTE ON (MIDI specification states that note off can either be a note off event OR a zero velocity note on event - so we must handle that.)
        -- if data[1] == 144 and data[3] ~= 0 then


        local midi_msg = midi.to_msg(data) -- Convert raw MIDI data to a structured table

        if (midi_msg.type == "note_on" and midi_msg.vel ~= 0) then
            print("Note: " .. midi_msg.note .. " Velocity: " .. midi_msg.vel)

            normal_midi_note_is_on = true
            normal_midi_note_is_off = false
            normal_midi_note_in = midi_msg.note -- data[2]
            captured_midi_note_in = midi_msg.note -- data[2] --


            OnMidiNoteInEvent(C_MIDI_NOTE_ON, midi_msg.note, midi_msg.vel, midi_msg.ch)
        elseif (midi_msg.type == "note_off" or midi_msg.vel == 0) then
            normal_midi_note_is_on = false
            normal_midi_note_is_off = true
            normal_midi_note_in = midi_msg.note --data[2]
            captured_midi_note_in = -1  -- We only want to have a captured note (one at a time) whilst the note is held down.
            -- Also, we ONLY want note off to reset this.

            OnMidiNoteInEvent(C_MIDI_NOTE_OFF, midi_msg.note, midi_msg.vel, midi_msg.ch)
        else
            if midi_msg.type == "cc" then
                print("midi cc " .. midi_msg.cc .. " = " .. midi_msg.val)

                -- Look for sustain pedal on.
                if midi_msg.cc == 64 and midi_msg.val == 127 then
                    -- init the midi sequence
                    init_keyboard_midi_note_events()
                    AllMidiNotesOff() -- panic.
                end
            else
                -- hopefully we have filtered out 254 above but there might be other stuff.
                print("other midi data: ")
                print(data[1])
                print(data[2])
                print(data[3])
                -- print("midi type: " .. midi_msg.type and midi_msg.type or "type is nil")
            end
        end
    end

    --if normal_midi_note_is_on == true then
    --  print ("NOTE ON: " .. normal_midi_note_in)


    -- To quote dan_dirks, "any MIDI note number divided by 12 is how the pitch is expressed in voltage (assuming volt per octave)"
    -- https://llllllll.co/t/frequencies-and-cv-converting-back-and-forth-in-lua-math-math-math/50984

    -- crow.output[1].volts = captured_midi_note_in / 12
    --crow.output[1].slew = tick_count / 10


    --end


    --if normal_midi_note_is_off == true then
    --  print ("NOTE OFF: " .. normal_midi_note_in)
    --end
end -- end test for 254




-- bug here
function set_sequence(x, y, midi_note)
    sequence_button_x = x
    sequence_button_y = y
    sequence_button_midi = midi_note

    if x ~= 0 and y ~= 0 then
        sequence_button_is_pressed = true
    else
        sequence_button_is_pressed = false
    end
end

function random_dense_grid(x, y)
    -- x and y should be the button pressed

    -- Depending on which column 1-16 is pressed on the row we want to change, we make the pattern more or less dense.
    -- i.e.
    -- <- sparse RANDOM PATTERN CREATION dense ->

    print("Hello from random_dense_grid: x: " .. x .. " y: " .. y)

    on_bias = x / 16 -- more bias towards an on note with a higher x button pressed


    print("on_bias: " .. on_bias)



    for j = 1, 16 do
        -- on the current step...
        -- chance of that step becoming 1 (on) (higher chance if we pressed button 16)

        dice = math.random() -- we want a real number between 0 and 1.

        print("dice: " .. dice)

        if on_bias > dice then
            unconditional_set_grid(j, y, 1)
        else
            unconditional_set_grid(j, y, 0)
        end
    end
end

function reset_row_states(row)
    row_states[row]["first_step"] = first_step
    row_states[row]["last_step"] = last_step
    -- row_states[row]["current_step"] = first_step
end

function preset_grid(x, y)
    reset_row_states(y)

    -- Any button pressed on this row (1)
    if y == 1 then
        print("Setting preset for row: " .. x)

        if x == 1 then
            grid_one_state[1][y]  = 1
            grid_one_state[2][y]  = 0
            grid_one_state[3][y]  = 0
            grid_one_state[4][y]  = 0

            grid_one_state[5][y]  = 1
            grid_one_state[6][y]  = 0
            grid_one_state[7][y]  = 0
            grid_one_state[8][y]  = 0

            grid_one_state[9][y]  = 1
            grid_one_state[10][y] = 0
            grid_one_state[11][y] = 0
            grid_one_state[12][y] = 0

            grid_one_state[13][y] = 1
            grid_one_state[14][y] = 0
            grid_one_state[15][y] = 0
            grid_one_state[16][y] = 0
        else
            -- TODO if x==2 then just reset_row_states not the pattern?
            random_dense_grid(x, y)
        end


        -- Any button pressed on this row (2)
    elseif y == 2 then
        print("Setting preset for row: " .. x)

        if x == 1 then
            grid_one_state[1][y]  = 0
            grid_one_state[2][y]  = 0
            grid_one_state[3][y]  = 1
            grid_one_state[4][y]  = 0

            grid_one_state[5][y]  = 0
            grid_one_state[6][y]  = 0
            grid_one_state[7][y]  = 1
            grid_one_state[8][y]  = 0

            grid_one_state[9][y]  = 0
            grid_one_state[10][y] = 0
            grid_one_state[11][y] = 1
            grid_one_state[12][y] = 0

            grid_one_state[13][y] = 0
            grid_one_state[14][y] = 0
            grid_one_state[15][y] = 1
            grid_one_state[16][y] = 0
        elseif x == 9 then
            -- Euclidean 3/8 tresillo pattern
            euclidean_preset(1, y)
        end
    elseif y == 3 then
        print("Setting preset for row: " .. x)

        if x == 1 then
            grid_one_state[1][y]  = 1
            grid_one_state[2][y]  = 1
            grid_one_state[3][y]  = 0
            grid_one_state[4][y]  = 1

            grid_one_state[5][y]  = 1
            grid_one_state[6][y]  = 1
            grid_one_state[7][y]  = 0
            grid_one_state[8][y]  = 1

            grid_one_state[9][y]  = 1
            grid_one_state[10][y] = 1
            grid_one_state[11][y] = 0
            grid_one_state[12][y] = 1

            grid_one_state[13][y] = 1
            grid_one_state[14][y] = 1
            grid_one_state[15][y] = 0
            grid_one_state[16][y] = 1
        else
            random_dense_grid(x, y)
        end
    elseif y == 4 then
        print("Setting preset for row: " .. x)

        if x == 1 then
            grid_one_state[1][y]  = 0
            grid_one_state[2][y]  = 0
            grid_one_state[3][y]  = 0
            grid_one_state[4][y]  = 0

            grid_one_state[5][y]  = 0
            grid_one_state[6][y]  = 0
            grid_one_state[7][y]  = 0
            grid_one_state[8][y]  = 0

            grid_one_state[9][y]  = 0
            grid_one_state[10][y] = 0
            grid_one_state[11][y] = 0
            grid_one_state[12][y] = 0

            grid_one_state[13][y] = 0
            grid_one_state[14][y] = 0
            grid_one_state[15][y] = 1
            grid_one_state[16][y] = 0
        else
            random_dense_grid(x, y)
        end
    elseif y == 5 then
        print("Setting preset for row: " .. x)

        if x == 1 then
            grid_one_state[1][y]  = 1
            grid_one_state[2][y]  = 0
            grid_one_state[3][y]  = 0
            grid_one_state[4][y]  = 0

            grid_one_state[5][y]  = 0
            grid_one_state[6][y]  = 0
            grid_one_state[7][y]  = 0
            grid_one_state[8][y]  = 0

            grid_one_state[9][y]  = 0
            grid_one_state[10][y] = 0
            grid_one_state[11][y] = 0
            grid_one_state[12][y] = 0

            grid_one_state[13][y] = 0
            grid_one_state[14][y] = 0
            grid_one_state[15][y] = 1
            grid_one_state[16][y] = 0
        else
            random_dense_grid(x, y)
        end
    elseif y == 6 then
        print("Setting preset for row: " .. x)

        if x == 1 then
            grid_one_state[1][y]  = 1
            grid_one_state[2][y]  = 0
            grid_one_state[3][y]  = 0
            grid_one_state[4][y]  = 0

            grid_one_state[5][y]  = 0
            grid_one_state[6][y]  = 1
            grid_one_state[7][y]  = 0
            grid_one_state[8][y]  = 0

            grid_one_state[9][y]  = 0
            grid_one_state[10][y] = 0
            grid_one_state[11][y] = 0
            grid_one_state[12][y] = 0

            grid_one_state[13][y] = 0
            grid_one_state[14][y] = 0
            grid_one_state[15][y] = 1
            grid_one_state[16][y] = 0
        else
            random_dense_grid(x, y)
        end
    elseif y == 7 then
        print("Setting preset for row: " .. x)

        if x == 1 then
            grid_one_state[1][y]  = 1
            grid_one_state[2][y]  = 1
            grid_one_state[3][y]  = 1
            grid_one_state[4][y]  = 0

            grid_one_state[5][y]  = 0
            grid_one_state[6][y]  = 0
            grid_one_state[7][y]  = 0
            grid_one_state[8][y]  = 0

            grid_one_state[9][y]  = 0
            grid_one_state[10][y] = 0
            grid_one_state[11][y] = 0
            grid_one_state[12][y] = 0

            grid_one_state[13][y] = 0
            grid_one_state[14][y] = 0
            grid_one_state[15][y] = 0
            grid_one_state[16][y] = 0
        else
            random_dense_grid(x, y)
        end
    end
end

function get_interesting_note_value(x, y, x_pressed)
    last_note = mozart_state[x][y]

    print(" last_note " .. last_note)

    print(" direction " .. direction)

    if direction == 1 then
        new_note = last_note + x + x_pressed

        print(" new_note " .. new_note)

        if new_note >= MOZART_BASE_MIDI_NOTE + MOZART_RANDOM_MAX_DELTA then
            new_note = last_note - x - x_pressed
            direction = 0
        end
    else
        new_note = last_note - x - x_pressed

        if new_note <= MOZART_BASE_MIDI_NOTE then
            new_note = last_note + x + x_pressed
            direction = 1
        end
    end


    print(" new_note " .. new_note)
    return new_note
end

function preset_mozart(x_button_pressed, y)
    -- Set the notes on row y.
    -- Use the value to determine the algorithm.

    -- if x == 1 do set all rows to MIDI note A4 (treat all the sequence rows the same.)
    -- else do some other patterns TODO improve this.

    for x = 1, 16 do
        if x_button_pressed == 1 then
            unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE - 24, 0) -- same note low low
        elseif x_button_pressed == 2 then
            unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE - 12, 0) -- same note low
        elseif x_button_pressed == 3 then
            unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE - 0, 0) -- same note
        else
            unconditional_set_mozart(x, y, get_interesting_note_value(x, y, x_button_pressed), 0)
        end
    end
end

function cycle_ratchet(x, y)
    -- We look at the current value of the grid_one_state and increment / Cycle around to produce a rest, normal and various ratchets
    if grid_one_state[x][y] == 0 then
        unconditional_set_grid(x, y, 1) -- this is not a ratchet, just a normal hit.
    elseif grid_one_state[x][y] == 1 then
        unconditional_set_grid(x, y, 2)
    elseif grid_one_state[x][y] == 2 then
        unconditional_set_grid(x, y, 3)
    elseif grid_one_state[x][y] == 3 then
        unconditional_set_grid(x, y, 4)
    elseif grid_one_state[x][y] == 4 then
        unconditional_set_grid(x, y, 5)
    elseif grid_one_state[x][y] == 5 then
        unconditional_set_grid(x, y, 0) -- This is a rest
    end
end

function do_mozart_down(x, y)
    print(" doing " .. ARM_MOZART_DOWN_BUTTON)
    unconditional_set_mozart(x, y, mozart_state[x][y] - 1, 1)
end

function do_mozart_up(x, y)
    print(" doing " .. ARM_MOZART_UP_BUTTON)
    unconditional_set_mozart(x, y, mozart_state[x][y] + 1, 1)
end

function toggle_sequence_grid(x, y)
    -- Is this used?
    -- This TOGGLES the grid states i.e. because z=1 push on/off push off/on etc.
    if grid_one_state[x][y] ~= 0 then -- "on" might be 1 or something else if its a ratchet etc.
        unconditional_set_grid(x, y, 0)
        held_x = 0
        held_y = 0
    else
        unconditional_set_grid(x, y, 1)
        held_x = x
        held_y = y
    end
end

function put_slide_on(x, y)
    slide_state[x][y] = 1
end

function take_slide_off(x, y)
    slide_state[x][y] = 0
end

function undo_grid()
    ----------
    -- UNDO --
    ----------
    --print ("Pressed 1,8: UNDO")


    -- Only do this if we know we can pop from undo
    if (lifo_populated(undo_grid_lifo)) then
        --print ("undo_grid_lifo is populated")

        -- In order to Undo we:


        -- local tally = refresh_grid_and_screen()
        -- print ("grid_one_state BEFORE push_grid_redo is:")
        -- print (grid_one_state)
        -- print ("tally is:" .. tally)

        push_grid_redo()

        -- local tally = refresh_grid_and_screen()
        -- print ("grid_one_state BEFORE pop_grid_undo is:")
        -- print (grid_one_state)
        -- print ("tally is:" .. tally)



        pop_grid_undo()

        -- local tally = refresh_grid_and_screen()
        -- print ("grid_one_state AFTER pop_grid_undo is:")
        -- print (grid_one_state)
        --print ("grid_one_state: " .. get_tally(grid_one_state))



        -- print ("grid_one_state is:")
        -- print (grid_one_state)
    else
        print("undo_grid_lifo is NOT populated")
    end
end

function redo_grid()
    -- REDO
    -- print ("Pressed 2,8: REDO")
    -- local tally = refresh_grid_and_screen()
    -- print ("grid_one_state BEFORE push_grid_undo is:")
    -- print (grid_one_state)
    -- print ("tally is:" .. tally)

    -- Only do this if we know we can pop from undo
    if (lifo_populated(redo_grid_lifo)) then
        -- print ("redo_grid_lifo is populated")

        push_grid_undo()

        -- local tally = refresh_grid_and_screen()
        -- print ("grid_one_state BEFORE pop_grid_redo is:")
        -- print (grid_one_state)
        -- print ("tally is:" .. tally)
        pop_grid_redo()

        --  refresh_grid_and_screen()

        -- local tally = refresh_grid_and_screen()
        -- print ("grid_one_state AFTER pop_grid_redo is:")
        -- print (grid_one_state)
        -- print ("tally is:" .. tally)

        --print ("grid_one_state: " .. get_tally(grid_one_state))
    else
        print("redo_grid_lifo is NOT populated")
    end
end

function undo_mozart()
    ----------
    -- UNDO MOZART--
    ----------

    -- Only do this if we know we can pop from undo
    if (lifo_populated(undo_mozart_lifo)) then
        push_mozart_redo()
        pop_mozart_undo()
    else
        print("undo_mozart_lifo is NOT populated")
    end
end

function redo_mozart()
    ----------
    -- REDO MOZART--
    ----------

    if (lifo_populated(redo_mozart_lifo)) then
        push_mozart_undo()
        pop_mozart_redo()
    else
        print("redo_grid_lifo is NOT populated")
    end
end

function print_table(t, indent)
    indent = indent or ""
    for k, v in pairs(t) do
        local key = tostring(k)
        if type(v) == "table" then
            print(indent .. key .. " = {")
            print_table(v, indent .. "  ")
            print(indent .. "}")
        else
            print(indent .. key .. " = " .. tostring(v))
        end
    end
end

function on_sequence_button_press_down(x, y, z)
    -- Every time we change state of sequence rows (non control rows), record the new state in the undo_grid_lifo
    push_grid_undo()
    push_mozart_undo()   -- TODO check this is not too much.

    toggle_sequence_grid(x, y)

    -- So we save the table to file
    -- (don't bother with control rows)
    --print ("Before set grids_are_dirty = true")
    grids_are_dirty = true
end

-- MAIN GRID LOOP
-- We capture monome grid key presses - Grid Key Presses
-- Main Grid button loop

my_grid_one.key = function(x, y, z)
    -- x is the column
    -- y is the row
    -- z == 1 means key down, z == 0 means key up

    print("Hello from ----------- my_grid_one.key = function -----------------")
    print("Captured value for monome grid row,column " ..
    x .. "," .. y .. " is " .. z .. " the value before change was: " .. grid_one_state[y][y])

    print("arm_control is: " ..
    arm_control ..
    " captured_midi_note_in is: " ..
    captured_midi_note_in ..
    " preset_mozart_button is: " .. preset_mozart_button .. " midi_note_key_pressed is: " .. midi_note_key_pressed)


    -- First lets capture the combination of buttons pressed (up to three groups i.e. one sequence button, one row7 and one row8 (control))

    if z == 1 then
        print("z is 1. You pressed a monome grid key down")
        if y <= TOTAL_SEQUENCE_ROWS then
            print("You pressed a Sequence Row button down")
            -- This holds the sequence button
            set_sequence(x, y, mozart_state[x][y]) -- bug here
        elseif y == 7 then
            print("Row7 On")
            arm_row7 = grid_button_function_name(x, y)
            if my_grid_one then my_grid_one:led(x, y, 12) end -- just show that the button is pressed
        elseif y == 8 then
            print("Control On")
            arm_control = grid_button_function_name(x, y)
            if my_grid_one then my_grid_one:led(x, y, 12) end
        else
            print("Error")
        end
    else
        print("Key Up")
        if y <= TOTAL_SEQUENCE_ROWS then
            print("Sequence Row Up")
            -- This releases the sequence button
            set_sequence(0, 0, 0)
        elseif y == 7 then
            print("Row7 Reset")
            arm_row7 = NO_FEATURE
            if my_grid_one then my_grid_one:led(x, y, 0) end
        elseif y == 8 then
            print("Control Reset")
            arm_control = NO_FEATURE
            if my_grid_one then my_grid_one:led(x, y, 0) end
        else
            print("Error")
        end
    end

    operation_matix_string = "x:" ..
    sequence_button_x ..
    " y:" ..
    sequence_button_x ..
    " z:" ..
    z ..
    " sequence_button_is_pressed: " ..
    tostring(sequence_button_is_pressed) ..
    " midi:" .. sequence_button_midi .. " arm_row7:" .. arm_row7 .. " arm_control:" .. arm_control


    print("Operation matrix is: " .. operation_matix_string)
    print("Before deciding what to do.. ")
    -- Now we have a matrix of buttons, now decide and process.

    if sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == NO_FEATURE then
        on_sequence_button_press_down(x, y, z)
    elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_PRESET_GRID_BUTTON then
        preset_grid(x, y)
    elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_PRESET_MOZART_BUTTON then
        preset_mozart(x, y)
    elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_RANDOMISE_GRID_BUTTON then
        randomize_grid(x, y)
    elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_RANDOMISE_MOZART_BUTTON then
        randomize_mozart(x, y)
    elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_RATCHET_BUTTON then
        cycle_ratchet(x, y)
    elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_MOZART_DOWN_BUTTON then
        do_mozart_down(x, y)
    elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_MOZART_UP_BUTTON then
        do_mozart_up(x, y)
    elseif sequence_button_is_pressed == false and arm_row7 == NO_FEATURE and arm_control == UNDO_GRID_BUTTON then
        undo_grid()
    elseif sequence_button_is_pressed == false and arm_row7 == NO_FEATURE and arm_control == REDO_GRID_BUTTON then
        redo_grid()
    elseif sequence_button_is_pressed == false and arm_row7 == NO_FEATURE and arm_control == UNDO_MOZART_BUTTON then
        undo_mozart()
    elseif sequence_button_is_pressed == false and arm_row7 == NO_FEATURE and arm_control == REDO_MOZART_BUTTON then
        redo_mozart()
    elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_SLIDE_OFF_BUTTON then
        take_slide_off(x, y)
    elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_SLIDE_ON_BUTTON then
        put_slide_on(x, y)
    elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_EUCLIDIAN_ROTATION_BUTTON then
        set_euclidian_rotation(x, y)
    elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_EUCLIDIAN_LENGTH_BUTTON then
        set_euclidian_length(x, y)
    elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_EUCLIDIAN_EVENTS_BUTTON then
        set_euclidean_events(x, y)
        print("ARM_EUCLIDIAN_EVENTS_BUTTON: Set events to " .. x .. " and generated pattern for row " .. y)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_01 and arm_control == NO_FEATURE then
        print("button" .. 1)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FIFTH * 0), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_02 and arm_control == NO_FEATURE then
        print("button" .. 2)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FIFTH * 1), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_03 and arm_control == NO_FEATURE then
        print("button" .. 3)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FIFTH * 2), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_04 and arm_control == NO_FEATURE then
        print("button" .. 4)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FIFTH * 3), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_05 and arm_control == NO_FEATURE then
        print("button" .. 5)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FOURTH * 1), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_06 and arm_control == NO_FEATURE then
        print("button" .. 6)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FOURTH * 2), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_07 and arm_control == NO_FEATURE then
        print("button" .. 7)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FOURTH * 3), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_08 and arm_control == NO_FEATURE then
        print("button" .. 8)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FOURTH * 4), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_09 and arm_control == NO_FEATURE then
        print("button" .. 9)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MAJOR_THIRD * 1), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_10 and arm_control == NO_FEATURE then
        print("button" .. 10)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MAJOR_THIRD * 2), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_11 and arm_control == NO_FEATURE then
        print("button" .. 11)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MAJOR_THIRD * 3), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_12 and arm_control == NO_FEATURE then
        print("button" .. 12)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MAJOR_THIRD * 4), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_13 and arm_control == NO_FEATURE then
        print("button" .. 13)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MINOR_THIRD * 1), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_14 and arm_control == NO_FEATURE then
        print("button" .. 14)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MINOR_THIRD * 2), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_15 and arm_control == NO_FEATURE then
        print("button" .. 15)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MINOR_THIRD * 3), 1)
    elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_16 and arm_control == NO_FEATURE then
        print("button" .. 16)
        unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MINOR_THIRD * 4), 1)
    else
        print("WARNING! No action found for the following combination of buttons: " .. operation_matix_string .. " )")
    end -- end of grid_button_function_name tests


    if y == 7 or y == 8 then
        last_action_method = grid_button_function_name(x, y):gsub("Button", ""):gsub("Arm", ""):gsub("Preset", "Pre")
        :gsub("Mozart", "Mz"):gsub("Grid", "Grd"):gsub("Randomise", "Rnd")                                                                                                             -- used in display
    end

    -- Always do this else results are not shown to user.
    refresh_grid_and_screen()
end -- End of my_grid_one.key function definition
-- /////////////////////////////////////////////////



-- Main loop for the second grid. This gets called every time a grid button is pressed.
my_grid_two.key = function(x, y, z)
    -- x is the column
    -- y is the row
    -- z == 1 means key down, z == 0 means key up

    print("Hello from ----------- my_grid_two.key = function -----------------")
    print("Captured value for monome grid two row,column " .. x .. "," .. y .. " is " .. z .. "")
    -- the value before change was: " .. grid_two_state[y][y])


    if z == 1 then
        if my_grid_two then my_grid_two:led(x, y, 12) end
    else
        -- 1) Turn the LED off to give feedback to the user
        if my_grid_two then my_grid_two:led(x, y, 0) end

        -- 2) Get the mozart_pointer for the button we just pressed off
        local mozart_pointer = scroll_state[x][y]


        print("here is the pointer for x " .. x .. " y " .. y)
        print("lane " .. mozart_pointer.current_midi_lane)
        print("bar " .. mozart_pointer.midi_bar_count)
        print("step " .. mozart_pointer.midi_step_count)
        print("note " .. mozart_pointer.midi_note_number)

        -- 3) Turn off the keyboard_midi_note
        keyboard_midi_note_events[mozart_pointer.current_midi_lane][mozart_pointer.midi_bar_count][mozart_pointer.midi_step_count][mozart_pointer.midi_note_number][1].is_active = 0



        -- 4) turn off this scroll_state[midi_step_count][count_of_active_midi_on].is_active = true


        scroll_state[x][y].is_active = 0

        print("note should be turned off soon " .. mozart_pointer.midi_note_number)

        -- By now we should have removed the note from the mozart_state / grid (difference?)
    end


    if my_grid_two then my_grid_two:refresh() end
end -- End of function for my_grid_two
-- //////////////////////////////////////////////


function set_euclidian_rotation(x, y)
    -- x is the rotation step
    -- y is the row

    -- We can set the first step (y) for the sequence row (x) as long as it is less than the last step of that row.
    if x <= row_states[y]["last_step"] then
        -- Save state for undo before making changes
        push_grid_undo()

        print("Setting first_step of row " .. y .. " to: " .. x)
        row_states[y]["first_step"] = x

        -- Use advanced Euclidean generation with current event count
        generate_euclidean_with_rotation(y, x)

        -- Mark grids as dirty so they get saved
        grids_are_dirty = true

        -- Show comprehensive info for user feedback
        local events = get_euclidean_events()
        local length = row_states[y]["last_step"]
        local rotation = x - 1
        print("ARM_EUCLIDIAN_ROTATION: Generated " .. events .. "/" .. length .. " Euclidean pattern, rotation=" .. rotation .. ", row=" .. y)
    else
        print("No can do. Rotation step of row " .. y .. " would be after last step. " .. x)
    end
end

function set_euclidian_length(x, y)
    if x >= row_states[y]["first_step"] then
        -- Save state for undo before making changes
        push_grid_undo()

        print("Setting euclidian_length (last_step) of row" .. y .. " to: " .. x)
        row_states[y]["last_step"] = x

        -- Auto-generate Euclidean rhythm with current event count and length
        local events = get_euclidean_events()
        apply_euclidean_to_row_with_length(y, events, 0)
        print("ARM_EUCLIDIAN_LENGTH: Generated " .. events .. "/" .. x .. " Euclidean pattern for row " .. y)

        -- Mark grids as dirty so they get saved
        grids_are_dirty = true
    else
        print("No can do. Euclidian length of row " .. y .. " would be before first step. " .. x)
    end
end



function get_tally(input_grid)
    -- A helper debug function to show the state of a grid
    -- A grid is a table with known dimensions
    -- Used for debugging and to prompt creation of tables if this gives an error.
    local tally = "id:" .. input_grid["id"] .. " colsXrows:"
    for col = 1, COLS do
        for row = 1, ROWS do
            -- This line throws an error if the table hasn't been dimensioned to col X row or is_active is missing.
            local cell_value = input_grid[col][row]
            if type(cell_value) == "table" then
                -- Handle MozartPointer objects - use is_active field
                tally = tally .. (cell_value.is_active or 0)
            else
                -- Handle numeric values
                tally = tally .. cell_value
            end
        end
    end
    return tally
end

function lifo_populated(input_lifo)
    local count = 0
    -- We consider the lifo populated if it contains a single key (a grid state table)
    for key, value in pairs(input_lifo) do
        --print(key, " -- ", value)
        count = count + 1
        -- No need to loop through whole table.
        if count > 0 then
            break
        end
    end

    if count > 0 then
        return true
    else
        return false
    end
end

function lifo_size(input_lifo)
    local count = 0
    -- We expect only tables in the lifo. Other keys will confuse this count.
    for key, value in pairs(input_lifo) do
        --print(key, " -- ", value)
        count = count + 1
    end

    return count
end

function get_copy_of_grid(input_grid)
    -- For creating copies of a grid for Undo and probably other things.
    local output_grid = create_a_grid() -- this returns a grid with the dimensions we expect
    -- copy all the key values except the ID
    output_grid["id"] = math.random(1, 99999999999999)
    -- output_grid["gspc"] = input_grid["gspc"]



    for col = 1, COLS do
        for row = 1, ROWS do
            --print ("col:" .. col .. " row:" .. row)
            output_grid[col][row] = input_grid[col][row]
        end
    end
    return output_grid
end

function display_tempo_status()
    screen.clear()
    screen.update()

    screen.move(1, 7)
    screen.text(tempo_status_string_1)

    screen.move(1, 14)
    screen.text(tempo_status_string_2)


    screen.move(1, 21)
    screen.text(tempo_status_string_3)

    screen.move(1, 28)
    screen.text(tempo_status_string_4)

    screen.move(1, 35)
    screen.text(tempo_status_string_5)

    screen.move(1, 42)
    screen.text(co2_ppm_status_string)


    screen.move(1, 49)
    screen.text(version_string)

    --screen.font_size(10)
    screen.move(1, 56)

    screen.text(string.format("%.4f", current_tempo))
    screen.update()
end

function refresh_grid_and_screen()
    --print ("Hello from refresh_grid_and_screen for grid at:")
    --print (grid_one_state)


    local tally = ""

    screen.clear()
    screen.move(1, 1)


    if (tempo_wow_is_good == 1 and tempo_flutter_is_good == 1) then
        screen.move(1, 7)


        screen.text(midiNoteToName(last_midi_note_on_out))
        screen.move(1, 14)
        screen.text(last_midi_on_velocity_out)

        screen.move(1, 21)
        screen.text(midiNoteToName(last_midi_note_off_out))
        screen.move(1, 28)

        screen.move(1, 35)
        screen.text("1:" .. g_count_of_active_midi_on)
        screen.move(1, 42)
        screen.text("0:" .. g_count_of_active_midi_off)
        -- screen.text(string.format("%X", total_wow_tempo_ticks * 255))

        screen.move(1, 49)
        screen.text("Bar")
        screen.move(1, 56)
        screen.text(midi_bar_count)




        -- Show min stability info
        -- screen.move(1,7)
        -- screen.text("W")
        -- screen.move(1,14)
        -- screen.text(wow_tempo_episodes)

        -- screen.move(1,21)
        -- screen.text("F")
        -- screen.move(1,28)
        -- screen.text(flutter_tempo_episodes)

        -- screen.move(1,35)
        -- screen.text("w")
        -- screen.move(1,42)
        -- screen.text(total_wow_tempo_ticks)
        -- -- screen.text(string.format("%X", total_wow_tempo_ticks * 255))

        -- screen.move(1,49)
        -- screen.text("t")
        -- screen.move(1,56)
        -- screen.text(total_flutter_tempo_ticks)


        -- NOTE This is only for display purposes.
        for col = 1, COLS do
            for row = 1, TOTAL_SEQUENCE_ROWS do -- don't want to set (or display) non sequence rows in this place
                tally = tally .. grid_one_state[col][row]

                screen.move(10 + (col * 7), row * 7)
                --screen.text("table[" .. row .. "]["..col.."] is: " ..grid_one_state[row][column])


                -- Show the scrolling of the steps with the sequence rows of LEDS. (Others will be used for other controls)
                -- note: row 7 has a dual use (sequence and set midi note when a row 8 button is presssed.)
                --if (current_step == col and row <= TOTAL_SEQUENCE_ROWS) then

                -- POLYR
                if (row_states[row]["current_step"] == col and row <= TOTAL_SEQUENCE_ROWS) then
                    -- This is the scrolling cursor
                    screen.text("*")

                    if (grid_one_state[col][row] >= 2) then -- ratchet
                        -- If current step and key is on, highlight it.
                        if my_grid_one then my_grid_one:led(col, row, 12) end
                    elseif (grid_one_state[col][row] == 1) then
                        -- If current step and key is on, highlight it.
                        if my_grid_one then my_grid_one:led(col, row, 9) end
                    else
                        -- Else use scrolling brightness
                        if my_grid_one then my_grid_one:led(col, row, 4) end
                    end
                else
                    if (grid_one_state[col][row] >= 2) then
                        if my_grid_one then my_grid_one:led(col, row, 8) end -- ratchet
                    elseif (grid_one_state[col][row] == 1) then
                        -- Not current step but Grid square is On
                        if my_grid_one then my_grid_one:led(col, row, 5) end
                    else
                        -- Not current step and key is off
                        if my_grid_one then my_grid_one:led(col, row, 0) end
                    end
                    -- Show the stored value on screen
                    screen.text(grid_one_state[col][row])
                end
            end -- end rows loop
        end -- end cols loop
    else
        display_tempo_status()
    end -- stable tempo check

    current_tempo = clock.get_tempo()


    if need_to_start_midi == true then
        end_of_line_text = "Pending CLK.."
    else
        end_of_line_text = output_text
    end

    if midi_step_count <= 4 then
        conductor_text = "1.."
    elseif midi_step_count > 4 and midi_step_count <= 8 then
        conductor_text = "2["
    elseif midi_step_count > 8 and midi_step_count <= 12 then
        conductor_text = "3 ]"
    elseif midi_step_count > 12 and midi_step_count <= 16 then
        conductor_text = "4^^"
    end

    -- print (conductor_text)


    -- status_text = conductor_text .. " " .. current_tempo .. " BPM. Step " .. midi_step_count .. " " .. end_of_line_text
    -- current_tempo no decimal points
    -- pad current step with a 0 so the display doesn't move about
    -- https://www.cprogramming.com/tutorial/printf-format-strings.html


    midi_status_text = "  MIDI IN " ..
    midiNoteToName(last_midi_note_in) ..
    " " ..
    string.format("%.3d", last_midi_velocity_in) ..
    " " .. string.format("%.1d", last_midi_on_off_in) .. " " .. string.format("%.2d", last_midi_channel_in)




    screen.move(1, 56)
    screen.text(midi_status_text)



    status_text = string.format("%.2f", current_tempo) ..
    " " ..
    string.format("%.2d", midi_step_count) ..
    " " ..
    last_action_method ..
    " " ..
    string.format("%.1d", last_x) ..
    "," ..
    string.format("%.1d", last_y) ..
    " " .. last_grid_value .. "-" .. last_mozart_value .. " " .. conductor_text .. " " .. end_of_line_text .. " "



    screen.move(1, 63)
    screen.text(status_text)



    screen.update() -- better to have this here than in the loop above because otherwise we get screen flickering

    if my_grid_one then my_grid_one:refresh() end
    if my_grid_two then my_grid_two:refresh() end
    -- print ("Bye from refresh_grid_and_screen tally is:" .. tally)

    return tally
end

-- NOTE redraw gets called by the norns implicitly sometimes and explicitly by us too.
-- Do NOT rename this function else we might have problems navigating to param screens etc.
function redraw()
    screen.clear()

    if (greetings_done == false) then
        --print("i will do greetings becuase not done yet")
        clock.run(greetings)
    else
        --print ("i will print table because greetings done")
        refresh_grid_and_screen()
    end
end
