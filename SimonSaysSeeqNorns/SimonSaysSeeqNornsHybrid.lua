-- SimonSaysSeeq on Norns - Hybrid Lua/Rust Version
-- Left Button Stop. Right Start
-- Licensed under the AGPL.
version = "1.8.0-hybrid"

version_string = "SimonSaysSeeq Norns Hybrid v" .. version

-- Load Rust core library (required for hybrid version)
local core_status, core = pcall(require, 'simon_says_seeq_core')
if not core_status then
    error("FATAL: Rust core library 'simon_says_seeq_core' not found! This is the hybrid version which requires the Rust core. Please build and install the Rust library first, or use the pure Lua version instead.")
end

-- Initialize Rust sequencer core
local sequencer_core = core.new_sequencer()
print("Rust core initialized successfully")

-- Detect if we're running on Norns or standalone
local is_norns = (_path and _path.dust) or (norns and norns.version)
print("DEBUG: is_norns =", is_norns)
print("DEBUG: clock before init =", clock)

-- Initialize modules for standalone operation only
if not is_norns then
    print("Running in standalone mode - initializing module stubs")
    
    -- Initialize clock module for standalone operation
    if not clock then
        print("DEBUG: Creating clock module...")
        clock = {}
        clock.transport = {}
        print("DEBUG: clock after creation =", clock)
        print("DEBUG: clock.transport =", clock.transport)
        
        -- Basic clock functionality
        local current_tempo = 120
        local sync_counter = 0
        
        function clock.get_tempo()
            return current_tempo
        end
        
        function clock.sync(beat_fraction)
            -- Simple sync implementation - just sleep for the appropriate time
            local sleep_time = (60 / current_tempo) * beat_fraction
            -- In a real implementation, this would be more sophisticated
            sync_counter = sync_counter + 1
            if sync_counter % 100 == 0 then
                -- Occasionally yield to prevent tight loops
                -- This is a placeholder - in real use, you'd want proper timing
            end
        end
        
        function clock.run(func, ...)
            -- Simple coroutine runner
            local co = coroutine.create(func)
            local ok, err = coroutine.resume(co, ...)
            if not ok then
                print("Clock coroutine error: " .. tostring(err))
            end
        end
        
        print("Initialized standalone clock module")
        print("DEBUG: Final clock =", clock)
    else
        print("DEBUG: clock already exists =", clock)
    end

    -- Initialize other Norns modules for standalone operation
    if not util then
        util = {}
        function util.clamp(value, min, max)
            if min and max then
                return math.max(min, math.min(max, value))
            elseif min then
                return math.max(min, value)
            else
                return value
            end
        end
        print("Initialized standalone util module")
    end

    if not params then
        params = {}
        local param_values = {}
        function params.set(key, value)
            param_values[key] = value
        end
        function params.get(key)
            return param_values[key] or 0
        end
        function params.delta(key, delta)
            param_values[key] = (param_values[key] or 0) + delta
        end
        print("Initialized standalone params module")
    end

    if not screen then
        screen = {}
        function screen.clear() end
        function screen.move(x, y) end
        function screen.text(text) end
        function screen.level(level) end
        function screen.update() end
        function screen.font_face(face) end
        function screen.font_size(size) end
        print("Initialized standalone screen module")
    end

    if not midi then
        midi = {}
        midi.devices = {}
        print("Initialized standalone midi module")
    end

    if not grid then
        grid = {}
        grid.devices = {}
        print("Initialized standalone grid module")
    end
else
    print("Running on Norns - using native modules")
end

NO_FEATURE = "NO_FEATURE"

-- Global timing variables
the_current_tick_count_since_start = 0
the_current_tick_count_since_step = 0
tick_count = 0
transport_is_active = false

-- MIDI and sequencer state
current_midi_lane = 1
midi_bar_count = 1
midi_step_count = 1
first_step = 1
last_step = 16

-- Constants
TOTAL_SEQUENCE_ROWS = 8
MIN_BAR = 1
MAX_BAR = 4
MIDI_KEYBOARD_CHANNEL = 1

-- Grid and display state
my_grid_one = nil
my_grid_two = nil
grid_one_state = {}
row_settings = {}

-- Tempo analysis variables
current_tempo = 120
wow_average_tempo = 120
flutter_average_tempo = 120
wow_threshold = 5.0
flutter_threshold = 1.0
tempo_wow_is_good = 1
tempo_flutter_is_good = 1
wow_tempo_episodes = 0
flutter_tempo_episodes = 0
total_wow_tempo_ticks = 0
total_flutter_tempo_ticks = 0
wow_window_size = 192
flutter_window_size = 48
wow_window_tick_position = 0
flutter_window_tick_position = 0
wow_tempo_sum = 0
flutter_tempo_sum = 0
tempo_is_stable = 1

-- Status strings for display
tempo_status_string_1 = ""
tempo_status_string_2 = ""
tempo_status_string_3 = ""
tempo_status_string_4 = ""
tempo_status_string_5 = ""

-- CO2 data variables
we_have_last_daily_co2_ppm_value = false
co2_ppm_daily_latest_value = 0
co2_ppm_status_string = "CO2 PPM: UNKNOWN"
no_of_co2_ppm_records = 0
total_tick_co2_count = 1
total_step_co2_count = 1

-- MIDI device ports
midi_gates_usb_device_port = nil
midi_keyboard_usb_device_port = nil
enable_midi_clock_out = 1
need_to_start_midi = false
run_conditional_clocks = false

-- Swing and timing
swing_mode = 1
swing_amount = 0

-- Gate constants
GATE_7 = 7
GATE_8 = 8
GATE_9 = 9
GATE_10 = 10
GATE_11 = 11
GATE_12 = 12

-- MIDI note events table (fallback for Lua-only mode)
keyboard_midi_note_events = {}

-- Scroll and grid state
scroll_state = {}
mozart_state = {}
slide_state = {}
held_state = {}

-- Global counters
g_count_of_active_midi_on = 0
g_count_of_active_midi_off = 0

-- Utility functions
function get_script_path()
    local info = debug.getinfo(1, 'S');
    local script_path = info.source:match [[^@?(.*[\/])[^\/]-$]]
    return script_path
end

function file_exists(name)
    local f = io.open(name, "r")
    if f ~= nil then
        io.close(f)
        return true
    else 
        return false 
    end
end

function validate_co2_value(raw_value)
    local co2_numeric = tonumber(raw_value)
    if co2_numeric and co2_numeric > 0 and co2_numeric < 10000 and co2_numeric == co2_numeric then
        return co2_numeric
    else
        return nil
    end
end

-- MIDI utility functions
function SanityCheckMidiNote(note)
    if note < 0 then return 0 end
    if note > 127 then return 127 end
    return note
end

function SanityCheckMidiVelocity(velocity)
    if velocity < 0 then return 0 end
    if velocity > 127 then return 127 end
    return velocity
end

function SanityCheckMidiChannel(channel)
    if channel < 1 then return 1 end
    if channel > 16 then return 16 end
    return channel
end

function SendMidiKeyboardNoteOn(note, velocity, channel)
    note = SanityCheckMidiNote(note)
    velocity = SanityCheckMidiVelocity(velocity)
    channel = SanityCheckMidiChannel(channel)
    
    if midi_keyboard_usb_device_port then
        if velocity > 0 then
            midi_keyboard_usb_device_port:note_on(note, velocity, channel)
        else
            midi_keyboard_usb_device_port:note_off(note, velocity, channel)
        end
    end
end

function AllMidiNotesOff()
    if midi_keyboard_usb_device_port then
        for note = 0, 127 do
            midi_keyboard_usb_device_port:note_off(note, 0, MIDI_KEYBOARD_CHANNEL)
        end
    end
end

-- High-performance MIDI processing using Rust core
function PlayMidiHybrid()
    if not core or not sequencer_core then
        -- Fallback to Lua implementation
        return PlayMidiLua()
    end
    
    -- Use Rust core for fast MIDI processing
    local active_notes = sequencer_core:process_midi_tick(current_tempo)
    
    local count_of_active_midi_on = 0
    local count_of_active_midi_off = 0
    
    -- Process the returned active MIDI notes
    for i = 1, #active_notes do
        local note_data = active_notes[i]
        local note = note_data[1]
        local velocity = note_data[2] 
        local status = note_data[3]
        
        if status == 144 then -- Note ON
            count_of_active_midi_on = count_of_active_midi_on + 1
            SendMidiKeyboardNoteOn(note, velocity, MIDI_KEYBOARD_CHANNEL)
        elseif status == 128 then -- Note OFF
            count_of_active_midi_off = count_of_active_midi_off + 1
            SendMidiKeyboardNoteOn(note, 0, MIDI_KEYBOARD_CHANNEL)
        end
    end
    
    g_count_of_active_midi_on = count_of_active_midi_on
    g_count_of_active_midi_off = count_of_active_midi_off
end

-- Fallback Lua MIDI processing (simplified version)
function PlayMidiLua()
    local count_of_active_midi_on = 0
    local count_of_active_midi_off = 0
    
    -- Simplified MIDI processing for fallback mode
    -- This would contain the original complex Lua logic
    -- For brevity, implementing a basic version here
    
    g_count_of_active_midi_on = count_of_active_midi_on
    g_count_of_active_midi_off = count_of_active_midi_off
end

-- Tempo analysis with hybrid approach
function analyze_tempo_stability()
    if core and sequencer_core then
        -- Let Rust handle heavy tempo calculations
        local wow_stable, flutter_stable = sequencer_core:analyze_tempo_stability(current_tempo)
        tempo_wow_is_good = wow_stable and 1 or 0
        tempo_flutter_is_good = flutter_stable and 1 or 0
        return wow_stable, flutter_stable
    else
        -- Fallback Lua tempo analysis
        local tempo_diff_wow = math.abs(wow_average_tempo - current_tempo)
        local tempo_diff_flutter = math.abs(flutter_average_tempo - current_tempo)
        
        if tempo_diff_wow > wow_threshold then
            total_wow_tempo_ticks = total_wow_tempo_ticks + 1
            if tempo_wow_is_good == 1 then
                wow_tempo_episodes = wow_tempo_episodes + 1
            end
            tempo_wow_is_good = 0
        else
            tempo_wow_is_good = 1
        end
        
        if tempo_diff_flutter > flutter_threshold then
            total_flutter_tempo_ticks = total_flutter_tempo_ticks + 1
            if tempo_flutter_is_good == 1 then
                flutter_tempo_episodes = flutter_tempo_episodes + 1
            end
            tempo_flutter_is_good = 0
        else
            tempo_flutter_is_good = 1
        end
    end
end

-- Initialize timing counters
function init_tick_count()
    tick_count = 0
end

function init_midi_step_count()
    midi_step_count = first_step
end

function init_midi_bar_count()
    midi_bar_count = MIN_BAR
end

function InitStepCountSinceStep()
    the_current_tick_count_since_step = 0
end

function IncrementStepCountSinceStep()
    the_current_tick_count_since_step = the_current_tick_count_since_step + 1
end

-- Row settings management
function create_row_settings()
    for row = 1, TOTAL_SEQUENCE_ROWS do
        row_settings[row] = {
            current_step = 1,
            first_step = 1,
            last_step = 16,
            ratchet_mode = 1
        }
    end
end

-- Main timing loop - HYBRID VERSION
function tick()
    while true do
        current_tempo = clock.get_tempo()
        
        -- Use hybrid MIDI processing
        PlayMidiHybrid()
        
        -- Hybrid tempo analysis
        analyze_tempo_stability()
        
        -- Update status strings
        if tempo_wow_is_good == 0 or tempo_flutter_is_good == 0 then
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
        
        if we_have_last_daily_co2_ppm_value then
            co2_ppm_status_string = "CO2 PPM: " .. co2_ppm_daily_latest_value
        else
            co2_ppm_status_string = "CO2 PPM: UNKNOWN"
        end
        
        -- Swing calculation
        if swing_mode == 1 then
            swing_amount = 0
        else
            swing_amount = (swing_mode / 18) * (1 / 480)
        end
        
        clock.sync(1 / 48) -- Run at twice 24 PPQN
        
        if transport_is_active then
            -- Gate processing for different divisions
            if tick_count % (192 * 1) == 0 then
                clock.run(process_clock_gate, GATE_12)
            end
            if tick_count % (192 * 2) == 0 then
                clock.run(process_clock_gate, GATE_11)
            end
            if tick_count % (192 * 4) == 0 then
                clock.run(process_clock_gate, GATE_10)
            end
            if tick_count % (192 * 8) == 0 then
                clock.run(process_clock_gate, GATE_9)
            end
            if tick_count % (192 * 16) == 0 then
                clock.run(process_clock_gate, GATE_8)
            end
            if tick_count % (192 * 32) == 0 then
                clock.run(process_clock_gate, GATE_7)
            end
            
            -- CO2 counter increment
            if no_of_co2_ppm_records > 0 then
                total_tick_co2_count = util.wrap(total_tick_co2_count + 1, 1, no_of_co2_ppm_records)
            end
            
            -- Step processing
            if tick_count % 12 == 0 then
                InitStepCountSinceStep()
                process_step()
                
                -- Grid LED updates
                if my_grid_two then
                    for i = 1, 8 do
                        my_grid_two:led(midi_step_count, i, 0)
                    end
                    my_grid_two:refresh()
                end
                
                -- Advance MIDI step
                midi_step_count = util.wrap(midi_step_count + 1, first_step, last_step)
                if midi_step_count == 1 then
                    midi_bar_count = util.wrap(midi_bar_count + 1, MIN_BAR, MAX_BAR)
                end
                
                -- Advance row steps
                for row = 1, TOTAL_SEQUENCE_ROWS do
                    row_settings[row]["current_step"] = util.wrap(
                        row_settings[row]["current_step"] + 1,
                        row_settings[row]["first_step"], 
                        row_settings[row]["last_step"]
                    )
                end
                
                -- CO2 step counter
                if no_of_co2_ppm_records > 0 then
                    total_step_co2_count = util.wrap(total_step_co2_count + 1, 1, no_of_co2_ppm_records)
                end
                
                redraw()
            end
        end
        
        -- Update counters
        tick_count = tick_count + 1
        the_current_tick_count_since_start = the_current_tick_count_since_start + 1
        
        -- Tempo window processing
        wow_window_tick_position = wow_window_tick_position + 1
        wow_tempo_sum = wow_tempo_sum + current_tempo
        
        flutter_window_tick_position = flutter_window_tick_position + 1
        flutter_tempo_sum = flutter_tempo_sum + current_tempo
        
        -- Calculate averages
        if wow_window_tick_position == wow_window_size then
            if wow_window_tick_position > 0 then
                wow_average_tempo = wow_tempo_sum / wow_window_tick_position
            else
                wow_average_tempo = current_tempo
            end
            init_wow_window()
        end
        
        if flutter_window_tick_position == flutter_window_size then
            if flutter_window_tick_position > 0 then
                flutter_average_tempo = flutter_tempo_sum / flutter_window_tick_position
            else
                flutter_average_tempo = current_tempo
            end
            init_flutter_window()
        end
        
        -- Reset tick count periodically
        if tick_count == 192 * 64 then
            init_tick_count()
        end
        
        IncrementStepCountSinceStep()
    end
end

-- Initialize tempo windows
function init_wow_window()
    wow_window_tick_position = 0
    wow_tempo_sum = 0
end

function init_flutter_window()
    flutter_window_tick_position = 0
    flutter_tempo_sum = 0
end

-- Process step function
function process_step()
    if need_to_start_midi == true then
        if midi_step_count == first_step then
            if enable_midi_clock_out == 1 then
                print("Send MIDI Start midi_step_count is: " .. midi_step_count)
                if midi_gates_usb_device_port then
                    midi_gates_usb_device_port:start()
                end
                if midi_keyboard_usb_device_port then
                    midi_keyboard_usb_device_port:start()
                end
            end
            run_conditional_clocks = true
            need_to_start_midi = false
        end
    end
    
    -- Process each sequence row
    for sequence_row = 1, TOTAL_SEQUENCE_ROWS do
        local ratchet_mode = 1
        if grid_one_state[row_settings[sequence_row]["current_step"]] then
            ratchet_mode = grid_one_state[row_settings[sequence_row]["current_step"]][sequence_row] or 1
        end
        
        clock.run(process_ratchet, sequence_row, ratchet_mode)
        
        -- CV output for crow (rows 3-6)
        if sequence_row >= 3 and sequence_row <= 6 then
            conditional_change_crow_output(row_settings[sequence_row]["current_step"], sequence_row)
        end
    end
end

-- Placeholder functions for compatibility
function process_ratchet(sequence_row, ratchet_mode)
    -- Ratchet processing logic
end

function process_clock_gate(gate_type)
    -- Clock gate processing logic
end

function conditional_change_crow_output(step, row)
    -- Crow CV output logic
end

-- Transport control functions
function transport_start()
    print("Transport START")
    transport_is_active = true
    need_to_start_midi = true
    AllMidiNotesOff()
end

function transport_stop()
    print("Transport STOP")
    transport_is_active = false
    need_to_start_midi = false
    AllMidiNotesOff()
    
    if enable_midi_clock_out == 1 then
        if midi_gates_usb_device_port then
            midi_gates_usb_device_port:stop()
        end
        if midi_keyboard_usb_device_port then
            midi_keyboard_usb_device_port:stop()
        end
    end
end

-- Set transport functions on clock module
print("DEBUG: About to set transport functions, clock =", clock)
print("DEBUG: clock.transport =", clock and clock.transport)
clock.transport.start = transport_start
clock.transport.stop = transport_stop
print("DEBUG: Transport functions set successfully")

-- User interface functions
function enc(n, delta)
    -- Encoder handling - keep existing Norns UI logic
    if n == 1 then
        -- Tempo adjustment
        params:delta("clock_tempo", delta)
    elseif n == 2 then
        -- Step length adjustment
        first_step = util.clamp(first_step + delta, 1, 16)
    elseif n == 3 then
        -- Last step adjustment
        last_step = util.clamp(last_step + delta, first_step, 16)
    end
    redraw()
end

function key(n, z)
    -- Key handling - keep existing Norns key logic
    if n == 2 and z == 1 then
        -- Left key - stop
        transport_stop()
    elseif n == 3 and z == 1 then
        -- Right key - start
        transport_start()
    end
end

-- Grid functions
function grid_button_function_name(x, y, z)
    -- Grid button handling - keep existing grid logic
    if z == 1 then
        -- Button press
        if grid_one_state[x] then
            grid_one_state[x][y] = (grid_one_state[x][y] or 0) + 1
            if grid_one_state[x][y] > 4 then
                grid_one_state[x][y] = 0
            end
        end
    end
    
    if my_grid_one then
        my_grid_one:refresh()
    end
end

-- Display function
function redraw()
    screen.clear()
    screen.level(15)
    screen.move(1, 10)
    screen.text(version_string)
    
    screen.move(1, 20)
    screen.text(tempo_status_string_1)
    
    screen.move(1, 30)
    screen.text("Step: " .. midi_step_count .. "/" .. last_step)
    
    screen.move(1, 40)
    screen.text("Bar: " .. midi_bar_count)
    
    screen.move(1, 50)
    screen.text(co2_ppm_status_string)
    
    if core then
        screen.move(1, 60)
        screen.text("Mode: Hybrid (Rust+Lua)")
    else
        screen.move(1, 60)
        screen.text("Mode: Lua Fallback")
    end
    
    screen.update()
end

-- Initialization
function init()
    print("DEBUG: init() function called!")
    print("DEBUG: clock in init =", clock)
    print("Initializing " .. version_string)
    
    -- Initialize MIDI
    midi_keyboard_usb_device_port = midi.connect(1)
    midi_gates_usb_device_port = midi.connect(2)
    
    -- Initialize grids
    my_grid_one = grid.connect(1)
    my_grid_two = grid.connect(2)
    
    if my_grid_one then
        my_grid_one.key = grid_button_function_name
    end
    
    -- Initialize data structures
    create_row_settings()
    
    for x = 1, 16 do
        grid_one_state[x] = {}
        for y = 1, 8 do
            grid_one_state[x][y] = 0
        end
    end
    
    for x = 1, 16 do
        scroll_state[x] = {}
        for y = 1, 8 do
            scroll_state[x][y] = {}
        end
    end
    
    -- Initialize timing
    init_tick_count()
    init_midi_step_count()
    init_midi_bar_count()
    InitStepCountSinceStep()
    init_wow_window()
    init_flutter_window()
    
    -- Set up parameters
    params:add_separator("SimonSaysSeeq Hybrid")
    params:add_option("enable_rust_core", "Enable Rust Core", {"Off", "On"}, core and 2 or 1)
    
    -- Start the main timing loop
    print("DEBUG: About to call clock.run(tick), clock =", clock)
    clock.run(tick)
    
    print("Initialization complete")
    print("Rust core status: " .. (core and "Available" or "Not available"))
    
    redraw()
end

-- Cleanup
function cleanup()
    print("Cleaning up " .. version_string)
    AllMidiNotesOff()
    transport_is_active = false
end