-- SimonSaysSeeq on Norns
-- Left Button Stop. Right Start
-- Licenced under the AGPL.

version = "1.0.0"

version_string = "SimonSaysSeeq Norns v" .. version

NO_FEATURE = "NO_FEATURE"

function get_script_path()
  local info = debug.getinfo(1,'S');
  local script_path = info.source:match[[^@?(.*[\/])[^\/]-$]]
  return script_path
end


print ("Current script path is " .. get_script_path())


--- 

-- file_path = io.open("/home/we/dust/data/SimonSaysSeeqNorns/simon_says_seeq_web_data_1.txt", "r")

-- print ("my file contents is: START" .. file .. "END")


local open = io.open

local function read_file(path)
    local file = open(path, "rb") -- r read mode and b binary mode
    if not file then return nil end
    local content = file:read "*a" -- *a or *all reads the whole file
    file:close()
    return content
end


function file_exists(name)
  local f=io.open(name,"r")
  if f~=nil then io.close(f) return true else return false end
end

 


-- # See gml.noaa.gov/ccgg/trends/ for additional details.


local co2_ppm_daily_latest_value = tonumber(read_file("/home/we/dust/data/SimonSaysSeeqNorns/simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_daily_latest.csv"));

if (co2_ppm_daily_latest_value) then 
  print ("here is the co2_ppm_daily_latest_value we got from the file: " .. co2_ppm_daily_latest_value);
  we_have_last_daily_co2_ppm_value = true
else
  print ("We do NOT have a daily co2 ppm ");
  we_have_last_daily_co2_ppm_value = false
end



local all_days_path = "/home/we/dust/data/SimonSaysSeeqNorns/simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_all_daily.csv"

if (file_exists(all_days_path)) then 
  print ("Yes all days path file exists");
  -- create a table out of the csv data.
  -- note the last match is greedy with *
  co2_ppm_list = {}
  for line in io.lines(all_days_path) do
      local year, month, day, something, the_co2_ppm_value = line:match("%s*(.-),%s*(.-),%s*(.-),%s*(.-),%s*(.*)")
      co2_ppm_list[#co2_ppm_list + 1] = { year = year, month = month, day = day, something = something, the_co2_ppm_value = the_co2_ppm_value }
  end
  
  -- for i,v in ipairs(co2_ppm_list) do
  --  -- print(i, v.year .. "-" .. v.month .. "-" .. v.day .. " is " .. v.the_co2_ppm_value)

  --   print(i, v.the_co2_ppm_value)

  -- end
  we_have_all_daily_co2_ppm_value = true
else
  print ("We do NOT have ALL daily co2 ppm ");
  we_have_all_daily_co2_ppm_values = false
end




-- TODO - add total_wow_tempo_ticks to display


-- To measure / display tempo instability
-- Two windows for averaging the tempo. 
-- WOW - Big instabillity in Tempo 
wow_window_size = 192 -- This is 16 steps at 12 ticks per step.
wow_window_tick_position = 0 -- how far through the averaging window we are.
wow_tempo_sum = 0 -- sum of tempo through the window so far
total_wow_tempo_ticks = 0 -- ticks we are unstable for
wow_tempo_episodes = 0 -- number of unstable episodes.
wow_average_tempo = 0 -- the average tempo over the window
tempo_wow_is_good = 1 -- All good at the moment, (no wow) (assume it is to start with.)
wow_threshold = 3.0 -- the difference between current and average tempo (in bpm) that triggers an episode

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
sequence_button_x = 0
sequence_button_midi = 0
sequence_button_is_pressed = false


arm_row7 = NO_FEATURE
arm_control = NO_FEATURE

print ("Current matrix is " .. sequence_button_x .. " " .. sequence_button_x .. " " .. sequence_button_midi .. " " .. arm_row7 .. " " .. arm_control)



-- clock is currently external so cant script clock 
--co2_ppm_simon_bday = 318
--default_tempo = 20
-- if (we_have_last_daily_co2_ppm_value) then
--   co2_ppm_delta = co2_ppm_daily_latest_value - co2_ppm_simon_bday
--   print ("co2_ppm_delta is " .. co2_ppm_delta)
--   current_tempo = default_tempo + co2_ppm_delta
-- else
--   current_tempo = default_tempo -- will be almost immediatly changed to the clock
-- end


-- print ("Initial tempo (current_tempo) is " .. current_tempo)


local volts = 0
local slew = 0

-- on/off for stepped sequence 
transport_is_active = true


-- Dimensions of Grid
COLS = 16
ROWS = 8

GATE_12 = 12
GATE_11 = 11
GATE_10 = 10
GATE_9 = 9
GATE_8 = 8
GATE_7 = 7

first_step = 1
last_step = COLS

arm_feature = NO_FEATURE
preset_grid_button = 0
arm_clock_button = 0
preset_mozart_button = 0


arm_first_step_button = 0
arm_last_step_button = 0


arm_put_slide_on = 0
arm_take_slide_off = 0


arm_swing_button = 0

swing_mode = 1

TOTAL_SEQUENCE_ROWS = 7 -- was 6
--MIN_GATE_ROW = 7 -- Not used?
--MAX_GATE_ROW = 12 -- Not used?

GRID_STATE_FILE = "/home/we/SimonSaysSeeq-grid.tbl"

MOZART_STATE_FILE = "/home/we/SimonSaysSeeq-mozart.tbl"

SLIDE_STATE_FILE = "/home/we/SimonSaysSeeq-slide.tbl"

last_action_method = ""
last_x = 0
last_y = 0
last_grid_value = 0
last_mozart_value = 0
last_slide_value = 0

held_x = 0
held_y = 0



end_of_line_text = ""
output_text = ""

Tab = require "lib/tabutil"

MIDI_CHANNEL_GATES = 1

-- This works well with Flame MGTV factory default settings. 
--  http://flame.fortschritt-musik.de/pdf/Manual_Flame_MGTV_module_v100_eng.pdf
LOWEST_MIDI_NOTE_NUMBER_FOR_GATE = 47 -- at least 1 will be added to this.

MIDI_NOTE_ON_VELOCITY = 127
MIDI_NOTE_OFF_VELOCITY = 0

MOZART_BASE_MIDI_NOTE = 24 -- C1  -- 33 = A1
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

audio_clock_file = _path.dust.."audio/SimonSaysSeeqAudio/modular-pulse.wav"

-- audio_clock_file = _path.dust.."audio/SimonSaysSeeqAudio/hermit_leaves.wav"

-- audio_clock_file = _path.dust.."audio/x0x/606/606-CH.wav"

-- PRESET_GRID_BUTTON = {x=1, y=2}

BUTTONS = {}

--7th Row -- Probably not used because now using for gates
ROW7_BUTTON_01 = "Button1"
ROW7_BUTTON_02 = "Button2"
ROW7_BUTTON_03 = "Button3"
ROW7_BUTTON_04 = "Button4"
table.insert(BUTTONS, {name = ROW7_BUTTON_01, x = 1, y = 7})
table.insert(BUTTONS, {name = ROW7_BUTTON_02, x = 2, y = 7})
table.insert(BUTTONS, {name = ROW7_BUTTON_03, x = 3, y = 7})
table.insert(BUTTONS, {name = ROW7_BUTTON_04, x = 4, y = 7})

ROW7_BUTTON_05 = "Button5"
ROW7_BUTTON_06 = "Button6"
ROW7_BUTTON_07 = "Button7"
ROW7_BUTTON_08 = "Button8"
table.insert(BUTTONS, {name = ROW7_BUTTON_05, x = 5, y = 7})
table.insert(BUTTONS, {name = ROW7_BUTTON_06, x = 6, y = 7})
table.insert(BUTTONS, {name = ROW7_BUTTON_07, x = 7, y = 7})
table.insert(BUTTONS, {name = ROW7_BUTTON_08, x = 8, y = 7})

ROW7_BUTTON_09 = "Button9"
ROW7_BUTTON_10 = "Button10"
ROW7_BUTTON_11 = "Button11"
ROW7_BUTTON_12 = "Button12"
table.insert(BUTTONS, {name = ROW7_BUTTON_09, x = 9, y = 7})
table.insert(BUTTONS, {name = ROW7_BUTTON_10, x = 10, y = 7})
table.insert(BUTTONS, {name = ROW7_BUTTON_11, x = 11, y = 7})
table.insert(BUTTONS, {name = ROW7_BUTTON_12, x = 12, y = 7})

ROW7_BUTTON_13 = "Button13"
ROW7_BUTTON_14 = "Button14"
ROW7_BUTTON_15 = "Button15"
ROW7_BUTTON_16 = "Button16"
table.insert(BUTTONS, {name = ROW7_BUTTON_13, x = 13, y = 7})
table.insert(BUTTONS, {name = ROW7_BUTTON_14, x = 14, y = 7})
table.insert(BUTTONS, {name = ROW7_BUTTON_15, x = 15, y = 7})
table.insert(BUTTONS, {name = ROW7_BUTTON_16, x = 16, y = 7})




-- 8th Row
UNDO_GRID_BUTTON = "UndoGridButton"
REDO_GRID_BUTTON = "RedoGridButton"
UNDO_MOZART_BUTTON = "UndoMozartButton"
REDO_MOZART_BUTTON = "RedoMozartButton"
table.insert(BUTTONS, {name = UNDO_GRID_BUTTON, x = 1, y = 8})
table.insert(BUTTONS, {name = REDO_GRID_BUTTON, x = 2, y = 8})
table.insert(BUTTONS, {name = UNDO_MOZART_BUTTON, x = 3, y = 8})
table.insert(BUTTONS, {name = REDO_MOZART_BUTTON, x = 4, y = 8})

--table.insert(BUTTONS, {name = "ArmMidiCommand", x = 5, y = 8})
--table.insert(BUTTONS, {name = "ArmSwing", x = 6, y = 8})
--table.insert(BUTTONS, {name = "DoMidiStop", x = 7, y = 8})
--table.insert(BUTTONS, {name = "DoMidiStart", x = 8, y = 8})


ARM_FIRST_STEP_BUTTON = "ArmFirstStep"
ARM_LAST_STEP_BUTTON = "ArmLastStep"
ARM_LAG_BUTTON = "ArmLag"
ARM_RATCHET_BUTTON = "ArmRatchet"
table.insert(BUTTONS, {name = ARM_FIRST_STEP_BUTTON, x = 5, y = 8}) -- note Lag is processed through ratchet
table.insert(BUTTONS, {name = ARM_LAST_STEP_BUTTON, x = 6, y = 8})
table.insert(BUTTONS, {name = ARM_LAG_BUTTON, x = 7, y = 8})
table.insert(BUTTONS, {name = ARM_RATCHET_BUTTON, x = 8, y = 8})

ARM_RANDOMISE_GRID_BUTTON = "RandomiseGrid"
ARM_RANDOMISE_MOZART_BUTTON = "RandomiseMozart"
ARM_PRESET_GRID_BUTTON = "PresetGrid"
ARM_PRESET_MOZART_BUTTON = "PresetMozart"
table.insert(BUTTONS, {name = ARM_RANDOMISE_GRID_BUTTON, x = 9, y = 8})
table.insert(BUTTONS, {name = ARM_RANDOMISE_MOZART_BUTTON, x = 10, y = 8})
table.insert(BUTTONS, {name = ARM_PRESET_GRID_BUTTON, x = 11, y = 8})
table.insert(BUTTONS, {name = ARM_PRESET_MOZART_BUTTON, x = 12, y = 8})

ARM_MOZART_DOWN_BUTTON = "ArmMozartDown"
ARM_MOZART_UP_BUTTON = "ArmMozartUp"
ARM_SLIDE_OFF_BUTTON = "ArmSlideOff"
ARM_SLIDE_ON_BUTTON = "ArmSlideOn"
table.insert(BUTTONS, {name = ARM_MOZART_DOWN_BUTTON, x = 13, y = 8})
table.insert(BUTTONS, {name = ARM_MOZART_UP_BUTTON, x = 14, y = 8})
table.insert(BUTTONS, {name = ARM_SLIDE_OFF_BUTTON, x = 15, y = 8})
table.insert(BUTTONS, {name = ARM_SLIDE_ON_BUTTON, x = 16, y = 8})


function reset_step_counters()
  current_step = first_step
  total_step_count = 1
end  


reset_step_counters()


tick_text = "."

tick_count = 0



blip_count = 0

PPQN24_GATES_ARE_ENABLED = true -- kind of duplicated setting


greetings_done = false



my_grid = grid.connect()

grid_state_dirty = false

print (my_grid)

-- Note the *ports*
-- midi.devices are:
-- 1	 -- 	table: 0x52be78
--   id	 -- 	1
--   dev	 -- 	userdata: 0x3f15a0
--   name	 -- 	virtual
-- 3	 -- 	table: 0x53fe78
--   id	 -- 	3
--   name	 -- 	Teensy MIDI
--   dev	 -- 	userdata: 0x554ea0
--   port	 -- 	1
-- 4	 -- 	table: 0x554ca0
--   id	 -- 	4
--   name	 -- 	USB MIDI Interface
--   dev	 -- 	userdata: 0x5557a8
--   port	 -- 	3
-- 5	 -- 	table: 0x52c338
--   id	 -- 	5
--   name	 -- 	Teensy MIDI 2
--   dev	 -- 	userdata: 0x555d08
--   port	 -- 	2




-- TODO make parameters so we can change in the UI
MIDI_GATES_PORT = 1
NORMAL_MIDI_PORT = 2


midi_gates_device = midi.connect(MIDI_GATES_PORT)
normal_midi_device = midi.connect(NORMAL_MIDI_PORT)


 -- defaults
 normal_midi_note_on = false
 normal_midi_note_off = false
 normal_midi_note_in = -1


 captured_normal_midi_note_in = -1
 midi_note_key_pressed = -1

 direction = 1 


-- psudo random for our grid ids
math.randomseed( os.time() )
math.random() -- call a few times so it gets more random (apparently)
math.random() 
math.random()

-- engine.name = 'PolyPerc'





function Set (list)
  local set = {}
  for _, l in ipairs(list) do set[l] = true end
  return set
end

-- swing 8ths
-- SWING_STEPS = Set { 3, 7, 11, 15 }

-- swing 16
-- we are not doing swing in code
SWING_STEPS = Set { 2, 4, 6, 8, 10, 12, 14, 16 }

-- Fonts: Note, we can use the Foundry app to view all the fonts.
-- Tried to find a fixed font (so strings don't jump around), but currently using the default font
-- Best approach probably is not to have long strings and instead place short strings at specific locations on the screen.

-- 2 at 8 works 
-- not 29, 40
-- Most of these fonts don't look good on Norns Shield
  -- 1 04B_03 (norns default) 
  -- 2 ALEPH 
  -- 3 Roboto Thin 
  -- 4 Roboto Light 
  -- 5 Roboto Regular 
  -- 6 Roboto Medium 
  -- 7 Roboto Bold 
  -- 8 Roboto Black 
  -- 9 Roboto Thin Italic 
  -- 10 Roboto Light Italic 
  -- 11 Roboto Italic 
  -- 12 Roboto Medium Italic 
  -- 13 Roboto Bold Italic 
  -- 14 Roboto Black Italic 
  -- 15 VeraBd 
  -- 16 VeraBI 
  -- 17 VeraIt 
  -- 18 VeraMoBd 
  -- 19 VeraMoBI 
  -- 20 VeraMoIt 
  -- 21 VeraMono 
  -- 22 VeraSeBd 
  -- 23 VeraSe 
  -- 24 Vera 
  -- 25 bmp/tom-thumb 
  -- 26 creep 
  -- 27 ctrld-fixed-10b 
  -- 28 ctrld-fixed-10r 
  -- 29 ctrld-fixed-13b 
  -- 30 ctrld-fixed-13b-i 
  -- 31 ctrld-fixed-13r 
  -- 32 ctrld-fixed-13r-i 
  -- 33 ctrld-fixed-16b 
  -- 34 ctrld-fixed-16b-i 
  -- 35 ctrld-fixed-16r 
  -- 36 ctrld-fixed-16r-i 
  -- 37 scientifica-11 
  -- 38 scientificaBold-11 
  -- 39 scientificaItalic-11 
  -- 40 ter-u12b 
  -- 41 ter-u12n 
  -- 42 ter-u14b 
  -- 43 ter-u14n 
  -- 44 ter-u14v 
  -- 45 ter-u16b 
  -- 46 ter-u16n 
  -- 47 ter-u16v 
  -- 48 ter-u18b 
  -- 49 ter-u18n 
  -- 50 ter-u20b 
  -- 51 ter-u20n 
  -- 52 ter-u22b 
  -- 53 ter-u22n 
  -- 54 ter-u24b 
  -- 55 ter-u24n 
  -- 56 ter-u28b 
  -- 57 ter-u28n 
  -- 58 ter-u32b 
  -- 59 ter-u32n 
  -- 60 unscii-16-full.pcf 
  -- 61 unscii-16.pcf 
  -- 62 unscii-8-alt.pcf 
  -- 63 unscii-8-fantasy.pcf 
  -- 64 unscii-8-mcr.pcf 
  -- 65 unscii-8.pcf 
  -- 66 unscii-8-tall.pcf 
  -- 67 unscii-8-thin.pcf


  -- See notes above
-- screen.font_face(1)
-- screen.font_size(7)


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






------------- TICK FUNCTION - THIS IS THE MAIN TIMING LOOP ---------------------------
function tick()
  while true do

   -- In clock sync, 1 refers to a quarter note so if we clock.sync(1) we will count 4 beats per bar
   -- if we clock.sync(1/4) we will count 16 beats per bar. (16 steps in the sequence)
   -- if we clock.sync(1/24) this is 24PPQN Pulses Per Quarter Note, I.e. standard MIDI clock

     
    --swing_amount = 0

current_tempo = clock.get_tempo()

-- Check for big differences in tempo from average 
if (math.abs(wow_average_tempo - current_tempo) > wow_threshold ) then

  -- UNSTABLE WOW
  -- Track the total number of ticks we have wow since we started the clock
  total_wow_tempo_ticks = total_wow_tempo_ticks + 1

  -- Moving from stable to unstable
  if (tempo_wow_is_good == 1 ) then
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
  if (tempo_flutter_is_good == 1 ) then
    flutter_tempo_episodes = flutter_tempo_episodes + 1

    -- As soon as we are stable start a new tempo windown
    -- init_flutter_window()
  end  

  -- New state is unstable
  tempo_flutter_is_good = 0

else
  tempo_flutter_is_good = 1


end

if (tempo_wow_is_good == 0 or tempo_flutter_is_good == 0 ) then
  tempo_status_string_1 = "Current Tempo (UNSTABLE): " .. string.format("%.2f",current_tempo) 
  tempo_is_stable = 0
else 
  tempo_status_string_1 = "Current Tempo: " .. string.format("%.2f",current_tempo) 
  tempo_is_stable = 1
end  


  tempo_status_string_2 = "Wow Av Tempo: " .. string.format("%.2f",wow_average_tempo) 
  tempo_status_string_3 = "Flutter Av Tempo: " .. string.format("%.2f",flutter_average_tempo)
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


    -- elseif swing_mode == 13 then
    --   swing_amount = 1/56
    -- elseif swing_mode == 14 then
    --   swing_amount = 1/70  
    -- elseif swing_mode == 15 then
    --   swing_amount = 1/80
    -- elseif swing_mode == 16 then
    --   swing_amount = 1/96
    -- end 


    --print ("tick says: current_step is: " .. current_step .. " tick_count is: " .. tick_count .. " blip_count is: " .. blip_count)

    clock.sync(1/48) -- Run at twice 24 PPQN so the even we can send gate on (for clock) and on the odd we can send gate off.

    -- if swing_mode == 1 then
    --    -- No swing, normal clock
    --   clock.sync(1/48) -- Run at twice 24 PPQN so the even we can send gate on (for clock) and on the odd we can send gate off.
    -- else   
    --   -- print ("swing_mode is: " .. swing_mode)


    --   -- note probably better to check on odd / even. in any case they must be balanced. what if changge pattern length?

    --   if SWING_STEPS[current_step] then
    --     print ("swinging step " .. current_step .. " amount is: + " .. swing_amount)
    --     clock.sync(1/48 + swing_amount)
    --   else
    --     -- Non swing step
    --     print ("swinging step " .. current_step .. " amount is: - " .. swing_amount)
    --     clock.sync(1/48 - swing_amount)
    --   end    
    -- end  


  





        if PPQN24_GATES_ARE_ENABLED == true then

          if run_conditional_clocks == true then

        -- 24 PPQN clock -- This is a 50 50 duty cycle
        if tick_count % 2 == 0 then
         
         if (enable_audio_clock_out == 1) then
          softcut.position(1,0) -- at 0 seconds there is the transient click BUT no click is produced.doesn't do much, so try at 5 seconds 1000 HZ tone, but needless to say it doesn't work
          softcut.play(1,1)
         end

        else

         -- softcut.position(1, 1)-- at this this position (1 second) there should be no sound
         if (enable_audio_clock_out == 1) then
           softcut.play(1,0)
         end

        end  


      end -- End conditional clocks check 

    end -- End check for 24 PPQN clocks




  if transport_is_active then 
    -- Every 12 ticks we want to advance the sequencer (if transport is active) 

    if blip_count == 0 then

      -- print("i would process the step here " .. blip_count)
      -- process_step() 
    end  

        -- Less frequently triggered gates

        if tick_count % (192 * 1) == 0 then
            clock.run(process_clock_gate, GATE_12)
            --print("tick_count is: " .. tick_count .. " GATE_12 ")
        end 

        if tick_count % (192 * 2)  == 0 then
          clock.run(process_clock_gate, GATE_11)
          --print("tick_count is: " .. tick_count .. " GATE_11 ")
        end 

        if tick_count % (192 * 4)  == 0 then
          clock.run(process_clock_gate, GATE_10)
          --print("tick_count is: " .. tick_count .. " GATE_10 ")
        end 

        if tick_count % (192 * 8)  == 0 then
          clock.run(process_clock_gate, GATE_9)
          --print("tick_count is: " .. tick_count .. " GATE_9 ")
        end 

        if tick_count % (192 * 16)  == 0 then
          clock.run(process_clock_gate, GATE_8)
          --print("tick_count is: " .. tick_count .. " GATE_8 ")
        end 

        -- Note: make sure reset of tick_count is at least this otherwise we won't go in here
        if tick_count % (192 * 32)  == 0 then
          clock.run(process_clock_gate, GATE_7)
          --print("tick_count is: " .. tick_count .. " GATE_7 ")
        end 


    if tick_count % 12 == 0 then
  

      --  print("tick_count is: " .. tick_count .. " blip_count is: " .. blip_count)


      process_step() 

      -- Always advance the step based on tick_count mod 12.    
      current_step = util.wrap(current_step + 1, first_step, last_step)
      -- print ("Advanced step to: " .. current_step)
      
      total_step_count = total_step_count + 1
        
      -- by setting a differnt value per step, we can control when it will count down to zero and hense trigger the processing of the subsequent step.
      if current_step == 3 then
        blip_count = 6
      else   
        blip_count = 12
      end

      redraw()

    end

    blip_count = blip_count - 1

  end   

  -- So tick_count doesn't get too big over the course of a long running session. (would end up slowing down modulus calcs?)
  tick_count = tick_count + 1


  wow_window_tick_position = wow_window_tick_position + 1
  wow_tempo_sum = wow_tempo_sum + current_tempo 


  flutter_window_tick_position = flutter_window_tick_position + 1
  flutter_tempo_sum = flutter_tempo_sum + current_tempo 

     
  if wow_window_tick_position == wow_window_size then -- 
    -- This means we calculate the average tempo over a fixed period of 192 ticks however we start the window again as soon as we have a stable tempo 

    screen.clear()
    screen.move(1,10)
    
   -- Don't floor because we don't want to go down one bpm if we're just under
    wow_average_tempo = wow_tempo_sum / wow_window_tick_position

    screen.text("Average Wow Tempo" .. wow_average_tempo)

    init_wow_window()
  end  


  if flutter_window_tick_position == flutter_window_size then -- 
    -- This means we calculate the average tempo over a fixed period of 192 ticks however we start the window again as soon as we have a stable tempo 

    screen.clear()
    screen.move(1,20)
    
    -- Don't floor because we don't want to go down one bpm if we're just under
    flutter_average_tempo = flutter_tempo_sum / flutter_window_tick_position

    screen.text("Average Flutter Tempo" .. flutter_average_tempo)

    init_flutter_window()
  end  



    if (tick_count == 192 * 64) then -- Reset so we don't have too many numbers on which we do modulus calculations

      -- This means we calculate the average tempo over a fixed period of 192 ticks  

      screen.clear()

      screen.move(1,10)
      screen.text(version_string)

      init_tick_count()

   

    end  

  end
end




function init_tick_count()
  tick_count = 0
end  


-- Note: This effictively gets called multiple times at boot 
function greetings()

  -- presumption of success but still seems to get run multiple times.
  greetings_done = true
  
  screen.clear()

  screen.move(1,10)
  screen.text(version_string)
  
  screen.move(1,20)
  
  
  grid_text = "Unknown"
  
  if (not my_grid) then
    grid_text = "Grid NOT CONNECTED"
  else
    grid_text = "Grid: " .. tostring(my_grid.name)
  end 
  
  screen.text(grid_text)
  
  screen.move(1,30)   
  screen.text(my_grid.cols .. " X " .. my_grid.rows)

  local y_position = 40


-- local do_print_midi = false

-- if do_print_midi then

  print ("midi.devices are:")

  for key, value in pairs(midi.devices) do

    local midi_text = ""

    print(key, " -- ", value)
    for sub_key, sub_value in pairs(value) do
      print("  " .. sub_key, " -- ", sub_value)
      
      if sub_key == "port" then
        midi_text = midi_text .. " P" .. sub_value 

       if sub_value == MIDI_GATES_PORT then
        midi_text = midi_text .. "GTES"
       end

       if sub_value == NORMAL_MIDI_PORT then
        midi_text = midi_text .. "MZRT"
       end


      end

      if sub_key == "name" then
        midi_text = midi_text .. " " .. sub_value
      end

    end

    screen.move(1,y_position)  
    screen.text(midi_text)
    print ("midi_text is: " .. midi_text)
    y_position = y_position + 10
    -- screen.update()
  
  end -- end loop of midi devices

  screen.update()
  clock.sleep(4)
  --print("now awake")
  greetings_done = true

  print_audio_file_info(audio_clock_file)
  
end




function process_step()
  --print ("process_step current_step is:  " .. current_step)

  
  local ratchet_mode = 1 -- default is 1 but it will be set

  --engine.hz(400) -- just to give some audible sign for debugging timing

  if need_to_start_midi == true then
  
    if current_step == first_step then

     --engine.hz(800) -- just to give some audible sign for debugging timing

      -- we only want to start midi clock at the right time!

      if (enable_midi_clock_out == 1 ) then
        print ("Send MIDI Start current_step is: " .. current_step)
        midi_gates_device:start()
        normal_midi_device:start()
      else
        print ("NOT Send MIDI Start (disabled) current_step is: " .. current_step)
      end
      run_conditional_clocks = true -- so our 24PPQN etc stays on when midi clock is on 
      need_to_start_midi = false

    else
      if (enable_midi_clock_out == 1 ) then
      print ("Waiting to MIDI Start current_step is: " .. current_step)
      else
        print (" MIDI Clock out disabled")
      end  
    end

   end -- End check midi start

 

  
  -- For each sequence row...
  for sequence_row = 1, TOTAL_SEQUENCE_ROWS do
    -- on the current step...
    ratchet_mode = grid_state[current_step][sequence_row]

    -- process step should run independently
    clock.run(process_ratchet, sequence_row, ratchet_mode)

    -- Sent appropriate midi note out as cv
    
    -- To quote dan_dirks, "any MIDI note number divided by 12 is how the pitch is expressed in voltage (assuming volt per octave)"
    -- https://llllllll.co/t/frequencies-and-cv-converting-back-and-forth-in-lua-math-math-math/50984

    -- Send the midi note number as CV we have previously captured (this currently sends even if the step is not active)

    -- We have 4 outputs on crow to output eurorack CV
    -- Here we check the slide and set the voltage to the pitch accordingly.
    if sequence_row >= 3 and sequence_row <= 6 then
      conditional_change_crow_output(current_step, sequence_row)
    end  


  end -- end for



  
end -- end function


function conditional_change_crow_output(current_step, sequence_row)

  crow_output = sequence_row - 2



    -- only change slew and voltage if the sequence step is active
    if grid_state[current_step][sequence_row]  ~= 0 then

      if slide_state[current_step][sequence_row] == 1 then
        crow.output[crow_output].slew = 0.1
      else
        crow.output[crow_output].slew = 0
      end  

    -- Row 3 special case for the CO2 PPM data
      if (sequence_row == 3) then
        print("hello from row 6 total_step_count is " .. total_step_count)

        co2_ppm_value_to_use = co2_ppm_list[total_step_count].the_co2_ppm_value

        voltage_to_set = co2_ppm_value_to_use / 50

        print (voltage_to_set)

        crow.output[crow_output].volts =  voltage_to_set      
      else -- use the notes from the grid
        crow.output[crow_output].volts = mozart_state[current_step][sequence_row] / 12 
      end

    end


    --if (sequence_row ==5) then
    --print("hello from row 5 voltage just set was " .. mozart_state[current_step][sequence_row] / 12 ) 
    --end


  
end -- end function  



function process_ratchet (output, ratchet_mode)
-- This is independent of the main clock. Thus its run with clock.run(process_ratchet, output, ratchet_mode)
  if ratchet_mode == 1 then -- could be 1 or 2 (ratchet) or...
    -- direct relation between value on grid at count of send_gates we will get

    gate_on(output)
    clock.sync(1/64)
    gate_off(output)

  elseif ratchet_mode == 2 then
    gate_on(output)
    clock.sync(1/64)
    gate_off(output)

    clock.sync(1/8)
    
    gate_on(output)
    clock.sync(1/64)
    gate_off(output)

  elseif ratchet_mode == 3 then

    gate_on(output)
    clock.sync(1/64)
    gate_off(output)

    clock.sync(1/12)
    
    gate_on(output)
    clock.sync(1/64)
    gate_off(output)

    clock.sync(1/12)

    gate_on(output)
    clock.sync(1/64)
    gate_off(output)


  elseif ratchet_mode == 4 then
    gate_on(output)
    clock.sync(1/64)
    gate_off(output)

    clock.sync(1/32)
    
    gate_on(output)
    clock.sync(1/64)
    gate_off(output)

    clock.sync(1/32)
    
    gate_on(output)
    clock.sync(1/64)
    gate_off(output)

    clock.sync(1/32)
    
    gate_on(output)
    clock.sync(1/64)
    gate_off(output)

  elseif ratchet_mode == 5 then
    gate_on(output)
    clock.sync(1/64)
    gate_off(output)

    clock.sync(1/8)
    
    gate_on(output)
    clock.sync(1/64)
    gate_off(output)

    clock.sync(1/32)
    
    gate_on(output)
    clock.sync(1/62)
    gate_off(output)

    clock.sync(1/32)
    
    gate_on(output)
    clock.sync(1/64)
    gate_off(output)

    -- this is a "lag" (pause before playing) 
    -- This applies lag to a single step.
    -- If we want to apply swing we'd need to look at a swing setting of the track
    -- Then if the track is swung we'd need to process each step's lag differently 
    -- depending on the type of swing
elseif ratchet_mode == 6 then
  clock.sync(1/32) 
  gate_on(output)
  clock.sync(1/64)
  gate_off(output)
-- Lag  
elseif ratchet_mode == 7 then
  clock.sync(1/16)
  gate_on(output)
  clock.sync(1/64)
  gate_off(output)
-- Lag  
elseif ratchet_mode == 8 then
  clock.sync(1/8)  
  gate_on(output)
  clock.sync(1/64)
  gate_off(output)
  -- Lag  
elseif ratchet_mode == 9 then
  clock.sync(1/6)
  gate_on(output)
  clock.sync(1/64)
  gate_off(output)

  end -- end non zero

 
  
end  

-- MUST be run as clock.run(process_clock_gate, output) - so syncs are independent
function process_clock_gate (output)

 -- if (enable_analog_clock_out == 1) then
    gate_on(output)
    clock.sync(1/64)
    gate_off(output)
 -- end 

end 

 
function gate_on(output)
       --print ("A ON LOWEST_MIDI_NOTE_NUMBER_FOR_GATE" .. LOWEST_MIDI_NOTE_NUMBER_FOR_GATE .. " MIDI_NOTE_ON_VELOCITY " .. MIDI_NOTE_ON_VELOCITY .. " sequence_row + MIDI_CHANNEL_GATES " .. sequence_row + MIDI_CHANNEL_GATES)
  midi_gates_device:note_on (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + output, MIDI_NOTE_ON_VELOCITY, MIDI_CHANNEL_GATES)
end 
  
function gate_off(output)
  --print ("A OFF LOWEST_MIDI_NOTE_NUMBER_FOR_GATE" .. LOWEST_MIDI_NOTE_NUMBER_FOR_GATE .. " MIDI_NOTE_OFF_VELOCITY " .. MIDI_NOTE_OFF_VELOCITY .. " sequence_row + MIDI_CHANNEL_GATES " .. sequence_row + MIDI_CHANNEL_GATES)
  midi_gates_device:note_off (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + output, MIDI_NOTE_OFF_VELOCITY, MIDI_CHANNEL_GATES)
end 



function clock.transport.start() 
  -- This function is maybe called
  -- 1) Via code attached to the Norns Right Button
  -- 2) Via the system when midi start is detected. ? check this.

  print("====================== transport.start says Hello ========================")

  init_tick_count()

  transport_is_active = true

  screen.clear()


 
  init_wow_window()

  init_flutter_window()

  init_wow_and_flutter_counters()
  
  screen.move(1,63)
  screen.text("Transport Start")
  screen.update()
end

function request_midi_start()
  print("request_midi_start")
  need_to_start_midi = true
end  
 




function clock.transport.stop()

  -- This function is maybe called
  -- 1) Via code attached to the Norns Right Button
  -- 2) Via the system when midi stop is detected. ? check this.


  print("================= transport.stop says Hello =======================")

  print("total_flutter_tempo_ticks since last start: " .. total_flutter_tempo_ticks)
  print("flutter_tempo_episodes since last start: " .. flutter_tempo_episodes)

  reset_step_counters()


--  screen.clear()

  transport_is_active = false
  --screen.move(80,80)
  --screen.text("Transport STOP")
  --screen.update()




  display_tempo_status()




end



function request_midi_stop()
  print("request_midi_stop")
  need_to_start_midi = false
  -- can stop the midi clock at any time.

if (enable_midi_clock_out == 1) then
  midi_gates_device:stop ()
  normal_midi_device:stop ()
end 


  run_conditional_clocks = false
end  

function enc(n,d)
  if n==3 then
     params:delta("clock_tempo", d)
  end
end 


-- Norns (Shield) key presses - (This is not the monome grid )
function key(n,z)
  print("key pressed.  n:" .. n ..  " z:" .. z )
  
  -- since MIDI and Link offer their own start/stop messages,
  -- we'll only need to manually start if using internal or crow clock sources:
 -- if params:string("clock_source") == "internal" then

    -- STOP left button pressed 
    if n == 2 and z == 1 then

      -- try moving pointer to quiet part of

    -- softcut.tape_play_stop ()

     -- if transport_is_active then -- currently running so Stop     
        clock.transport.stop()
        request_midi_stop()

        
     -- else -- Not currently running so reset. 
       -- effectively we press this again.
        reset_step_counters()

     -- end
      
      screen_dirty = true
    end
    
    -- START Right button pressed
    if n == 3 and z == 1 then

      -- try moving pointer to loud part of sample

    --  softcut.tape_play_start ()


      if not transport_is_active then
        clock.transport.start()
        request_midi_start() -- Just send MIDI start instead of requesting?

        

      end
      
      screen_dirty = true
    end
    
 -- end
end









function grid_button_function_name (x,y)

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
    print ("x was: " .. x .. " y was: " .. y) 
  end  

  return ret

end -- end function definition


-- Originally copied from https://github.com/monome/softcut-studies/blob/master/1-basics.lua
function print_audio_file_info(file)
  if util.file_exists(file) == true then
    local ch, samples, samplerate = audio.file_info(file)
    local duration = samples/samplerate
    print("loading file: "..file)
    print("  channels:\t"..ch)
    print("  samples:\t"..samples)
    print("  sample rate:\t"..samplerate.."hz")
    print("  duration:\t"..duration.." sec")
  else print "ERROR read_wav(): file not found" end
end



function init()

  -- clear buffer
  softcut.buffer_clear()
  -- read file into buffer
  -- buffer_read_mono (file, start_src, start_dst, dur, ch_src, ch_dst)
  softcut.buffer_read_mono(audio_clock_file,0,0,-1,1,1,1,1)
  


  -- softcut.buffer_read_stereo(audio_clock_file, 0, 0, -1)


  -- audio.tape_play_open (audio_clock_file)


  -- enable voice 1
  softcut.enable(1,1)
  -- set voice 1 to buffer 1
  softcut.buffer(1,1)
  -- set voice 1 level to 1.0
  softcut.level(1,1.0)
  
  
  
  -- voice 1  loop
   softcut.loop(1,0) -- loop off
  -- set voice 1 loop start to 1
  softcut.loop_start(1,0)
  -- set voice 1 loop end to 2
  softcut.loop_end(1,5)
  -- set voice 1 position to 0
  
  softcut.fade_time(1,0)
  
  softcut.position(1,0.0)

  -- set voice 1 rate to 1.0
  softcut.rate(1,1.0)
  
  
  
  audio:rev_off ()
  audio:comp_off ()

  -- enable voice 1 play
  softcut.play(1,1)



--  params:set("clock_source",4)
  
    -- Last In First Out (LIFO) tables for Undo and Redo of grid state functionality
  undo_grid_lifo = {}
  redo_grid_lifo = {}

  undo_mozart_lifo = {}
  redo_mozart_lifo = {}




  print ("before init_grid_state_table")
  init_grid_state_table()
    

  print ("before init_mozart_state_table")
  init_mozart_state_table()

  print ("before init_slide_state_table")
  init_slide_state_table()


  print ("before init_held_state_table")
  init_held_state_table()



  refresh_grid_and_screen()

  print("hello")
  -- my_grid:all(2)
  my_grid:refresh() -- refresh the LEDs
    
    
  print("my_grid follows: ")
  print(my_grid)
  print("my_grid.name is: " .. my_grid.name)
  print("my_grid.cols is: " .. my_grid.cols)
  print("my_grid.rows is: " .. my_grid.rows)
  

  print ("midi.devices are:")
  for key, value in pairs(midi.devices) do
    print(key, " -- ", value)
    for sub_key, sub_value in pairs(value) do
      print("  " .. sub_key, " -- ", sub_value)
    end
  end -- end loop of midi devices


  -- Set the starting tempo. Can be changed with right knob
  -- TODO store and retreive this
  params:set("clock_tempo",80)
  
  --midi_start_on_bar_id = clock.run(midi_start_on_bar)

  -- print ("******START***************")
  -- print(grid_button_function_name (11,8))
  -- print(grid_button_function_name (2,1))
  -- print ("========END============")

-- here
--crow.output[1].scale = {0,7,2,9}

current_tempo = clock.get_tempo()
flutter_average_tempo = clock.get_tempo() -- just for initial value

init_wow_and_flutter_counters()
init_wow_window()
init_flutter_window()

   print("init says: Starting main sequencer timing called tick.")
   clock.run(tick)       -- start the sequencer


  print_audio_file_info(audio_clock_file)


  end -- end init




 -- Periodically check if we need to save the grid state to file.
 -- TODO - if we're not saving this when running then might as well just save it when we stop (rather than have a loop)
  clock.run(function()
    while true do
      clock.sleep(5)
        if (grid_state_dirty == true) then

           if (transport_is_active == false) then -- only save if we're stopped. (not sure we really need this) for sure we don't want to write to disk when playing

            Tab.save(grid_state, GRID_STATE_FILE)

            -- for now do mozart at the same time.
            Tab.save(mozart_state, MOZART_STATE_FILE)

            -- for now do mozart at the same time.
            Tab.save(slide_state, SLIDE_STATE_FILE)


            grid_state_dirty = false

            print("I saved tables.")

            screen.move(SCREEN_INFO_X,SCREEN_INFO_Y)
            screen.text("I saved tables.")
            screen.update() 

           end
        end
    end
  end)




function load_grid_state()
  
  grid_state = Tab.load (GRID_STATE_FILE)


  
  -- grid state popularity counter
  grid_state["gspc"]=0 -- Reset this (apart from anything this assures the key is there)


  print("Result of table load is:")
  print (grid_state)
  print (get_tally(grid_state))

  return grid_state
end
  


function load_mozart_state()
  


  mozart_state = Tab.load (MOZART_STATE_FILE) 


  mozart_state["gspc"]=0 -- Reset this (apart from anything this assures the key is there)

  print("Result of table load is:")
  print (mozart_state)
  print (get_tally(mozart_state))

  return mozart_state
end


function load_slide_state()

  slide_state = Tab.load (SLIDE_STATE_FILE) 

  slide_state["gspc"]=0 -- Reset this (apart from anything this assures the key is there)

  print("Result of table load is:")
  print (slide_state)
  print (get_tally(slide_state))

  return slide_state
end


-- a general grid. This is used for grid, mozart, slide etc.
function create_a_grid()
  local local_grid = {}
  local_grid["id"]=math.random(1,99999999999999) -- an ID for debugging purposes
  local_grid["gspc"]=0 -- we might increment this to see how popular it is 
  for col = 1, COLS do 
    local_grid[col] = {} -- create a table for each col
    for row = 1, ROWS do
        local_grid[col][row] = 0
    end
  end
  return local_grid
end 


function init_grid_state_table()
  
  print ("Hello from init_grid_state_table")
  
  -- Try to load the table
  local status, err = pcall(load_grid_state)

  if status then
    print ("load grid state seems ok. grid_state is:")
    print (grid_state)
  else
    print ("Seems we got an error - setting grid_state to nil so we will create it and save it: " .. err)
    grid_state = nil
  end  
  
  -- if it doesn't exist
  if grid_state == nil then
    print ("No table, I will generate a structure and save that")

    grid_state = create_a_grid()



    Tab.save(grid_state, GRID_STATE_FILE)
    grid_state = Tab.load (GRID_STATE_FILE)  
  else
    print ("I already have a grid_state table, no need to generate one")
  end

  -- We want to make sure rown 8 are all off. 
  -- Note: row 7 has a kind of dual function but 8 is all control.
  for y = 8, 8 do
    for x = 1, 16 do
      print("turn off x:" .. x .. " y:" .. y)
      unconditional_set_grid_non_seq_button(x, y, 0)
    end 
  end

  
 print ("grid tally is: " .. get_tally(grid_state))

 print ("clock.get_tempo() is: " .. clock.get_tempo())
 
 


  -- Push Undo so we can get back to initial state
  push_grid_undo()

  
  print ("Bye from init_grid_state_table")
  

end -- end init_grid_state_table


----

function init_mozart_state_table()
  
  print ("Hello from init_mozart_state_table")
  
  -- Try to load the table
  local status, err = pcall(load_mozart_state)

  if status then
    print ("load mozart state seems ok. mozart_state is:")
    print (mozart_state)
  else
    print ("Seems we got an error - setting mozart_state to nil so we will create it and save it: " .. err)
    mozart_state = nil
  end  
  
  -- if it doesn't exist
  if mozart_state == nil then
    print ("No table, I will generate a structure and save that")

    mozart_state = create_a_grid()



    Tab.save(mozart_state, MOZART_STATE_FILE)
    mozart_state = Tab.load (MOZART_STATE_FILE)  
  else
    print ("I already have a mozart_state table, no need to generate one")
  end
  
 print ("tally is: " .. get_tally(mozart_state))

 print ("clock.get_tempo() is: " .. clock.get_tempo())
 
   -- Push Undo so we can get back to initial state
   push_mozart_undo()

  print ("Bye from init_mozart_state_table")
  

end -- end init_mozart_state_table




function init_slide_state_table()
  
  print ("Hello from init_slide_state_table")
  
  -- Try to load the table
  local status, err = pcall(load_slide_state)

  if status then
    print ("load slide state seems ok. slide_state is:")
    print (slide_state)
  else
    print ("Seems we got an error - setting slide_state to nil so we will create it and save it: " .. err)
    slide_state = nil
  end  
  
  -- if it doesn't exist
  if slide_state == nil then
    print ("No table, I will generate a structure and save that")

    slide_state = create_a_grid()



    Tab.save(slide_state, SLIDE_STATE_FILE)
    slide_state = Tab.load (SLIDE_STATE_FILE)  
  else
    print ("I already have a slide_state table, no need to generate one")
  end
  
 print ("tally is: " .. get_tally(slide_state))

 print ("clock.get_tempo() is: " .. clock.get_tempo())
 
 


  
  print ("Bye from init_slide_state_table")
  

end -- end init_slide_state_table

--------------

function init_held_state_table()
  
  print ("Hello from init_held_state_table")
  
  -- Don't want to load or save - always create new
  held_state = create_a_grid()

  
 --print ("tally is: " .. get_tally(held_state))

  
  print ("Bye from init_held_state_table")
  

end -- end init_held_state_table







--------------
-----

function push_grid_undo()

  --print("push_grid_undo says hello. Store Undo LIFO")
  -- TODO check memory / count of states? - if this gets very large, truncate from the other side

  -- When we push to the undo_grid_lifo, we want to *copy* the grid_state (not reference) so that any subsequent changes to grid_state are not saved on the undo_grid_lifo 
  -- Inserts in the last position of the table (push)
  table.insert (undo_grid_lifo, get_copy_of_grid(grid_state))

  --print ("undo_grid_lifo size is: ".. lifo_size(undo_grid_lifo))

end  

function push_mozart_undo()
  table.insert (undo_mozart_lifo, get_copy_of_grid(mozart_state))
end  





function pop_grid_undo()

  if lifo_size(undo_grid_lifo) > 1 then

    -- 2) Pop from the undo_grid_lifo to the current state.
    -- Removes from the last element of the table (pop)
    local undo_state = table.remove (undo_grid_lifo)

    grid_state = get_copy_of_grid(undo_state)

    --print ("undo_grid_lifo size is: ".. lifo_size(undo_grid_lifo))
    
    -- Thus if A through G are all the states we've seen, and E is the current state, we'd have the following:
    
    --    ABCDEFG
    --        *
    -- grid_state: E
    --
    -- undo_grid_lifo       redo_grid_lifo
    --    D                F 
    --    C                G 
    --    B
    --    A 

  else 
    print ("Not poping last undo_grid_lifo because its size is 1 or less ")  
  end
end 

function pop_mozart_undo()

  if lifo_size(undo_mozart_lifo) > 1 then
    local undo_state = table.remove (undo_mozart_lifo)
    mozart_state = get_copy_of_grid(undo_state)

  else 
    print ("Not poping last undo_mozart_lifo because its size is 1 or less ")  
  end
end 





function push_grid_redo()
    -- 1) Push the current state to the redo_grid_lifo so we can get back to it.
    -- Similarly we want to *copy* the grid_state (not reference) 
    -- so any subsequent changes to the grid_state are not reflected in the redo_grid_lifo
    table.insert (redo_grid_lifo, get_copy_of_grid(grid_state))
    
    --print ("redo_grid_lifo size is: ".. lifo_size(redo_grid_lifo))
end  


function push_mozart_redo()
  table.insert (redo_mozart_lifo, get_copy_of_grid(mozart_state))
end  




function pop_grid_redo()

  if lifo_size(redo_grid_lifo) > 1 then

      -- 2) Pop the redo_grid_lifo into the current state.

      -- TODO need to copy this?
      local redo_state = table.remove (redo_grid_lifo) 
      grid_state = get_copy_of_grid(redo_state)

      --print ("redo_grid_lifo size is: ".. lifo_size(redo_grid_lifo))
    else 
      print ("Not poping last redo_grid_lifo because its size is 1 or less ")  
    end

end  



function pop_mozart_redo()
  if lifo_size(redo_mozart_lifo) > 1 then
      local redo_state = table.remove (redo_mozart_lifo) 
      mozart_state = get_copy_of_grid(redo_state)
    else 
      print ("Not poping last redo_mozart_lifo because its size is 1 or less ")  
    end
end  

function have_held ()
  if held_x ~= 0 and held_y ~= 0 then
    print ("have_held is true")
    return true
  else 
    print ("have_held is false")
    return false
  end
end  


function unconditional_set_mozart(x, y, midi_note_number, set_grid_on)
 

  print ("unconditional_set_mozart says: Got x: " .. x .. " y: " ..  y ..  " midi_note_number: " .. midi_note_number)

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

  print ("unconditional_set_mozart says: Just set x: " .. x .. " y: " ..  y ..  " to midi note: " .. midi_note_number)

end



function unconditional_set_grid_non_seq_button(x, y, integer)  
  if y > TOTAL_SEQUENCE_ROWS then

    if integer < 0 then
      integer = 0
    end  
  
    if integer > 1 then
      integer = 1
    end
  
    grid_state[x][y] = integer

  else
    print ("Error: unconditional_set_grid_non_seq_button will not set state of sequence button ")
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

  grid_state[x][y] = integer

  last_x = x
  last_y = y 
  last_grid_value = integer

else
  print ("Error: unconditional_set_grid will not set state of non sequence button ")
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


-- probably not used (but does get called becuase lots of prints)
midi_gates_device.event = function(data)
  -- print("---------------------- midi_gates_device IN ---------------------------------------")
end



captured_normal_midi_note_in = -1


 


-- Capture MIDI IN
normal_midi_device.event = function(data)

    -- Something is sending lots of midi messages. is it the USB to DIN adapter?
  if data[1] == 254 then
    -- Ignore this MIDI message 
    --print("Noisy MIDI normal_midi_device")

  else  
    --for key, value in pairs(data) do
    --  print(key, " -- ", value)
    --end


  -- If NOTE ON (MIDI specification states that note off can either be a note off event OR a zero velocity note on event - so we must handle that.)
  if data[1] == 144 and data[3] ~= 0 then
    normal_midi_note_on = true
    normal_midi_note_off = false
    normal_midi_note_in = data[2]
    captured_normal_midi_note_in = data[2] -- store this so we can act on a later step press
    

    -- set_mozart_and_grid_based_on_held_key(captured_normal_midi_note_in)



  end

  -- NOTE OFF  
  if data[1] == 128 or data[3] == 0 then
    normal_midi_note_on = false
    normal_midi_note_off = true
    normal_midi_note_in = data[2]
    captured_normal_midi_note_in = -1 -- We only want to have a captured note (one at a time) whilst the note is held down.
                                      -- Also, we ONLY want note off to reset this.    
  end 



  if normal_midi_note_on == true then
    print ("NOTE ON: " .. captured_normal_midi_note_in)


    -- To quote dan_dirks, "any MIDI note number divided by 12 is how the pitch is expressed in voltage (assuming volt per octave)"
    -- https://llllllll.co/t/frequencies-and-cv-converting-back-and-forth-in-lua-math-math-math/50984

   -- crow.output[1].volts = captured_normal_midi_note_in / 12
    --crow.output[1].slew = tick_count / 10


  end  


  if normal_midi_note_off == true then
    print ("NOTE OFF: " .. normal_midi_note_in)
  end 

end -- end test for 254

end   


function set_sequence(x,y,midi_note)

  sequence_button_x = x
  sequence_button_y = y
  sequence_button_midi = midi_note

  if x ~= 0 and y ~= 0 and midi_note ~= 0 then
    sequence_button_is_pressed = true
  else 
    sequence_button_is_pressed = false
  end


end  



-- here


function random_dense_grid(x, y)
  -- x and y should be the button pressed

  -- Depending on which column 1-16 is pressed on the row we want to change, we make the pattern more or less dense.
  -- i.e.
  -- <- sparse RANDOM PATTERN CREATION dense -> 

  print ("Hello from random_dense_grid: x: " .. x .. " y: " .. y) 

  on_bias = x / 16 -- more bias towards an on note with a higher x button pressed


  print ("on_bias: " .. on_bias ) 
  
 

  for j = 1, 16 do
    -- on the current step...
    -- chance of that step becoming 1 (on) (higher chance if we pressed button 16)
    
    dice = math.random() -- we want a real number between 0 and 1.

    print ("dice: " .. dice ) 

    if on_bias > dice then 
      unconditional_set_grid(j, y, 1)
    else 
      unconditional_set_grid(j, y, 0)
    end 
  end

end





function preset_grid (x,y)

    -- Any button pressed on this row (1)
    if y == 1 then

      print ("Setting preset for row: " .. x) 

      if x == 1 then

        grid_state[1][y] = 1
        grid_state[2][y] = 0
        grid_state[3][y] = 0
        grid_state[4][y] = 0

        grid_state[5][y] = 1
        grid_state[6][y] = 0
        grid_state[7][y] = 0
        grid_state[8][y] = 0

        grid_state[9][y]  = 1
        grid_state[10][y] = 0
        grid_state[11][y] = 0
        grid_state[12][y] = 0

        grid_state[13][y] = 1
        grid_state[14][y] = 0
        grid_state[15][y] = 0
        grid_state[16][y] = 0
      
      else
        random_dense_grid(x, y)
      end
            

     -- Any button pressed on this row (2)  
    elseif y == 2 then

      print ("Setting preset for row: " .. x) 

      if x == 1 then

        grid_state[1][y] = 0
        grid_state[2][y] = 0
        grid_state[3][y] = 1
        grid_state[4][y] = 0

        grid_state[5][y] = 0
        grid_state[6][y] = 0
        grid_state[7][y] = 1
        grid_state[8][y] = 0

        grid_state[9][y]  = 0
        grid_state[10][y] = 0
        grid_state[11][y] = 1
        grid_state[12][y] = 0

        grid_state[13][y] = 0
        grid_state[14][y] = 0
        grid_state[15][y] = 1
        grid_state[16][y] = 0

      else
        random_dense_grid(x, y)
      end

    elseif y == 3 then

      print ("Setting preset for row: " .. x) 

      if x == 1 then

        grid_state[1][y] = 1
        grid_state[2][y] = 1
        grid_state[3][y] = 0
        grid_state[4][y] = 1

        grid_state[5][y] = 1
        grid_state[6][y] = 1
        grid_state[7][y] = 0
        grid_state[8][y] = 1

        grid_state[9][y]  = 1
        grid_state[10][y] = 1
        grid_state[11][y] = 0
        grid_state[12][y] = 1

        grid_state[13][y] = 1
        grid_state[14][y] = 1
        grid_state[15][y] = 0
        grid_state[16][y] = 1

      else
        random_dense_grid(x, y)
      end

    elseif y == 4 then

      print ("Setting preset for row: " .. x) 

      if x == 1 then

        grid_state[1][y] = 0
        grid_state[2][y] = 0
        grid_state[3][y] = 0
        grid_state[4][y] = 0

        grid_state[5][y] = 0
        grid_state[6][y] = 0
        grid_state[7][y] = 0
        grid_state[8][y] = 0

        grid_state[9][y]  = 0
        grid_state[10][y] = 0
        grid_state[11][y] = 0
        grid_state[12][y] = 0

        grid_state[13][y] = 0
        grid_state[14][y] = 0
        grid_state[15][y] = 1
        grid_state[16][y] = 0

      else
        random_dense_grid(x, y)
      end  

    elseif y == 5 then

      print ("Setting preset for row: " .. x) 

      if x == 1 then

        grid_state[1][y] = 1
        grid_state[2][y] = 0
        grid_state[3][y] = 0
        grid_state[4][y] = 0

        grid_state[5][y] = 0
        grid_state[6][y] = 0
        grid_state[7][y] = 0
        grid_state[8][y] = 0

        grid_state[9][y]  = 0
        grid_state[10][y] = 0
        grid_state[11][y] = 0
        grid_state[12][y] = 0

        grid_state[13][y] = 0
        grid_state[14][y] = 0
        grid_state[15][y] = 1
        grid_state[16][y] = 0

      else
        random_dense_grid(x, y)
      end   

    elseif y == 6 then

      print ("Setting preset for row: " .. x) 

      if x == 1 then

        grid_state[1][y] = 1
        grid_state[2][y] = 0
        grid_state[3][y] = 0
        grid_state[4][y] = 0

        grid_state[5][y] = 0
        grid_state[6][y] = 1
        grid_state[7][y] = 0
        grid_state[8][y] = 0

        grid_state[9][y]  = 0
        grid_state[10][y] = 0
        grid_state[11][y] = 0
        grid_state[12][y] = 0

        grid_state[13][y] = 0
        grid_state[14][y] = 0
        grid_state[15][y] = 1
        grid_state[16][y] = 0

      else
        random_dense_grid(x, y)
      end   


    elseif y == 7 then

      print ("Setting preset for row: " .. x) 

      if x == 1 then

        grid_state[1][y] = 1
        grid_state[2][y] = 1
        grid_state[3][y] = 1
        grid_state[4][y] = 0

        grid_state[5][y] = 0
        grid_state[6][y] = 0
        grid_state[7][y] = 0
        grid_state[8][y] = 0

        grid_state[9][y]  = 0
        grid_state[10][y] = 0
        grid_state[11][y] = 0
        grid_state[12][y] = 0

        grid_state[13][y] = 0
        grid_state[14][y] = 0
        grid_state[15][y] = 0
        grid_state[16][y] = 0

      else
        random_dense_grid(x, y)
      end   






    end


end

function get_interesting_note_value(x, y, x_pressed)

  last_note = mozart_state[x][y] 

  print (" last_note ".. last_note)

  print (" direction ".. direction)

  if direction == 1 then
    
    new_note = last_note + x + x_pressed

    print (" new_note ".. new_note)

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


  print (" new_note ".. new_note)
  return new_note

end





function preset_mozart(x_button_pressed,y)
-- Set the notes on row y.
-- Use the value to determine the algorithm. 

-- if x == 1 do set all rows to MIDI note A4 (treat all the sequence rows the same.)
-- else do some other patterns TODO improve this.

for x = 1, 16 do
  if x_button_pressed == 1 then
    unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE, 0) -- same note
  else
    unconditional_set_mozart(x, y, get_interesting_note_value(x, y, x_button_pressed), 0)
  end  
 
end


end  

function cycle_ratchet(x,y)
    -- We look at the current value of the grid_state and increment / Cycle around to produce a rest, normal and various ratchets
  if grid_state[x][y] == 0 then
    unconditional_set_grid(x,y,1) -- this is not a ratchet, just a normal hit.
  elseif grid_state[x][y] == 1 then
    unconditional_set_grid(x,y,2) 
  elseif grid_state[x][y] == 2 then
    unconditional_set_grid(x,y,3) 
  elseif grid_state[x][y] == 3 then
    unconditional_set_grid(x,y,4) 
  elseif grid_state[x][y] == 4 then
    unconditional_set_grid(x,y,5) 
  elseif grid_state[x][y] == 5 then
    unconditional_set_grid(x,y,0) -- This is a rest
  end

end


function do_mozart_down(x,y)
  print (" doing ".. ARM_MOZART_DOWN_BUTTON)
    unconditional_set_mozart(x, y, mozart_state[x][y] - 1, 1)
end  

function do_mozart_up(x,y)
  print (" doing ".. ARM_MOZART_UP_BUTTON)
    unconditional_set_mozart(x, y, mozart_state[x][y] + 1, 1)

end 

function toggle_sequence_grid(x,y)
  -- Is this used?
  -- This TOGGLES the grid states i.e. because z=1 push on/off push off/on etc.
  if grid_state[x][y] ~= 0 then -- "on" might be 1 or something else if its a ratchet etc.
    unconditional_set_grid(x,y,0)
    held_x = 0
    held_y = 0
  else 
    unconditional_set_grid(x,y,1)
    held_x = x
    held_y = y 
  end

end  

 
function put_slide_on(x,y)
  slide_state[x][y] = 1
end  


function take_slide_off(x,y)
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
      -- print ("grid_state BEFORE push_grid_redo is:")
      -- print (grid_state)
      -- print ("tally is:" .. tally)

      push_grid_redo()

      -- local tally = refresh_grid_and_screen()
      -- print ("grid_state BEFORE pop_grid_undo is:")
      -- print (grid_state)
      -- print ("tally is:" .. tally)



      pop_grid_undo()

      -- local tally = refresh_grid_and_screen()
      -- print ("grid_state AFTER pop_grid_undo is:")
      -- print (grid_state)
      --print ("grid_state: " .. get_tally(grid_state))


  
      -- print ("grid_state is:")
      -- print (grid_state)

    else
      print ("undo_grid_lifo is NOT populated")
    end

  end

function redo_grid()

   -- REDO  
      -- print ("Pressed 2,8: REDO")
      -- local tally = refresh_grid_and_screen()
      -- print ("grid_state BEFORE push_grid_undo is:")
      -- print (grid_state)
      -- print ("tally is:" .. tally)

          -- Only do this if we know we can pop from undo 
          if (lifo_populated(redo_grid_lifo)) then
            -- print ("redo_grid_lifo is populated")
    
            push_grid_undo()
    
            -- local tally = refresh_grid_and_screen()
            -- print ("grid_state BEFORE pop_grid_redo is:")
            -- print (grid_state)
            -- print ("tally is:" .. tally)
            pop_grid_redo()
    
          --  refresh_grid_and_screen()
    
            -- local tally = refresh_grid_and_screen()
            -- print ("grid_state AFTER pop_grid_redo is:")
            -- print (grid_state)
            -- print ("tally is:" .. tally)
    
            --print ("grid_state: " .. get_tally(grid_state))
          else
            print ("redo_grid_lifo is NOT populated")
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
        print ("undo_mozart_lifo is NOT populated")
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
        print ("redo_grid_lifo is NOT populated")
      end  

end  

function on_sequence_button_press_down (x,y,z)

      -- Every time we change state of sequence rows (non control rows), record the new state in the undo_grid_lifo
      push_grid_undo()
      push_mozart_undo() -- TODO check this is not too much.

      toggle_sequence_grid(x,y)
  
      -- So we save the table to file
      -- (don't bother with control rows)
      --print ("Before set grid_state_dirty = true")
      grid_state_dirty = true


end 




-- MAIN GRID LOOP
-- Here we capture monome grid key presses - Grid Key Presses
-- Main Grid button loop

my_grid.key = function(x,y,z)
-- x is the column
-- y is the row
-- z == 1 means key down, z == 0 means key up

print("Hello from ----------- my_grid.key = function -----------------")
print(x .. ","..y .. " z is " .. z.. " value before change " .. grid_state[y][y])

-- print("arm_control is: ".. arm_control .. " captured_normal_midi_note_in is: " ..  captured_normal_midi_note_in .. " preset_mozart_button is: " .. preset_mozart_button .. " midi_note_key_pressed is: " .. midi_note_key_pressed)


-- First lets capture the combination of buttons pressed (up to three groups i.e. one sequence button, one row7 and one row8 (control))

if z == 1 then
  print("Key Down")
  if y <= TOTAL_SEQUENCE_ROWS then
    print("Sequence Row Down")
    -- This holds the sequence button
    set_sequence(x,y,mozart_state[x][y])
  elseif y == 7 then
    print("Row7 On")
    arm_row7 = grid_button_function_name(x,y)
    my_grid:led(x,y,12) -- just show that the button is pressed
  elseif y == 8 then
    print("Control On")
    arm_control = grid_button_function_name(x,y)
    my_grid:led(x,y,12) 
  else
    print("Error")
  end  
else
  print("Key Up")
  if y <= TOTAL_SEQUENCE_ROWS then
    print("Sequence Row Up")
    -- This releases the sequence button
    set_sequence(0,0,0)
  elseif y == 7 then
    print("Row7 Reset")
    arm_row7 = NO_FEATURE
    my_grid:led(x,y,0) 
  elseif y == 8 then
    print("Control Reset")
    arm_control = NO_FEATURE
    my_grid:led(x,y,0) 
  else
    print("Error")
  end
end   

operation_matix_string = "x:" .. sequence_button_x .. " y:" .. sequence_button_x .. " midi:" .. sequence_button_midi .. " arm_row7:" .. arm_row7 .. " arm_control:" .. arm_control


print ("Operation matrix is: " ..  operation_matix_string)

-- Now we have a matrix of buttons, now process.

if sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == NO_FEATURE then
  on_sequence_button_press_down(x,y,z)
elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_PRESET_GRID_BUTTON then
  preset_grid(x,y)
elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_PRESET_MOZART_BUTTON then
  preset_mozart(x,y)
elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_RANDOMISE_GRID_BUTTON then 
  randomize_grid (x,y)
elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_RANDOMISE_MOZART_BUTTON then  
  randomize_mozart (x,y)
elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_RATCHET_BUTTON then  
  cycle_ratchet(x,y)
elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_MOZART_DOWN_BUTTON then
  do_mozart_down(x,y)
elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_MOZART_UP_BUTTON then
  do_mozart_up(x,y)
elseif sequence_button_is_pressed == false and arm_row7 == NO_FEATURE and arm_control == UNDO_GRID_BUTTON then
  undo_grid()
elseif sequence_button_is_pressed == false and arm_row7 == NO_FEATURE and arm_control == REDO_GRID_BUTTON then
  redo_grid()
elseif sequence_button_is_pressed == false and arm_row7 == NO_FEATURE and arm_control == UNDO_MOZART_BUTTON then
  undo_mozart()
elseif sequence_button_is_pressed == false and arm_row7 == NO_FEATURE and arm_control == REDO_MOZART_BUTTON then
  redo_mozart()
elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_SLIDE_OFF_BUTTON then
  take_slide_off(x,y)  
elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_SLIDE_ON_BUTTON then
  put_slide_on(x,y)  
elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_FIRST_STEP_BUTTON then
  set_first_step(x)
elseif sequence_button_is_pressed == true and arm_row7 == NO_FEATURE and arm_control == ARM_LAST_STEP_BUTTON then
  set_last_step(x)
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_01 and arm_control == NO_FEATURE then
  print("button" .. 1)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FIFTH * 0),1) 
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_02 and arm_control == NO_FEATURE then
  print("button" .. 2)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FIFTH * 1),1) 
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_03 and arm_control == NO_FEATURE then
  print("button" .. 3)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FIFTH * 2),1)
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_04 and arm_control == NO_FEATURE then
  print("button" .. 4)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FIFTH * 3),1)
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_05 and arm_control == NO_FEATURE then
  print("button" .. 5)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FOURTH * 1),1)
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_06 and arm_control == NO_FEATURE then
  print("button" .. 6)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FOURTH * 2),1)
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_07 and arm_control == NO_FEATURE then
  print("button" .. 7)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FOURTH * 3),1)
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_08 and arm_control == NO_FEATURE then
  print("button" .. 8)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_PERFECT_FOURTH * 4),1)
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_09 and arm_control == NO_FEATURE then
  print("button" .. 9)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MAJOR_THIRD * 1),1)
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_10 and arm_control == NO_FEATURE then
  print("button" .. 10)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MAJOR_THIRD * 2),1)
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_11 and arm_control == NO_FEATURE then
  print("button" .. 11)             
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MAJOR_THIRD * 3),1) 
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_12 and arm_control == NO_FEATURE then
  print("button" .. 12)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MAJOR_THIRD * 4),1) 
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_13 and arm_control == NO_FEATURE then
  print("button" .. 13)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MINOR_THIRD * 1),1)
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_14 and arm_control == NO_FEATURE then
  print("button" .. 14)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MINOR_THIRD * 2),1)
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_15 and arm_control == NO_FEATURE then
  print("button" .. 15)
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MINOR_THIRD * 3),1)
elseif sequence_button_is_pressed == true and arm_row7 == ROW7_BUTTON_16 and arm_control == NO_FEATURE then
  print("button" .. 16) 
  unconditional_set_mozart(x, y, MOZART_BASE_MIDI_NOTE + (MOZART_INTERVAL_MINOR_THIRD * 4),1)
else
  print("(No action for this combination of buttons: " ..  operation_matix_string .. " )") 
end -- end of grid_button_function_name tests


if y == 7 or y == 8 then
  last_action_method = grid_button_function_name(x,y):gsub( "Button", ""):gsub( "Arm", ""):gsub( "Preset", "Pre"):gsub( "Mozart", "Mz"):gsub( "Grid", "Grd"):gsub( "Randomise", "Rnd") -- used in display
end

  -- Always do this else results are not shown to user.
  refresh_grid_and_screen()


end -- End of my_grid.key function definition

-- //////////////////////////////////////////////
-------------------------/////////////////////////
-------------------------/////////////////////////

function set_first_step(step)
  if step <= last_step then
    print ("Setting first_step to: " .. step)
    first_step = step 
  else 
    print ("No can do. First step would be after last step. " .. step)
  end
end  

function set_last_step(step)
  if step >= first_step then
    print ("Setting last_step to: " .. step)
    last_step = step 
  else 
    print ("No can do. Last step would be before first step. " .. step) 
  end
end  


function get_tally(input_grid)
  -- A helper debug function to show the state of a grid
  -- A grid is a table with known dimensions
  -- Used for debugging
  local tally = "id:" ..input_grid["id"] .. " gspc:" .. input_grid["gspc"] .. " colsXrows:"
  for col = 1,COLS do 
    for row = 1,ROWS do
      tally = tally .. input_grid[col][row]
    end 
  end
  return tally
end  


function lifo_populated(input_lifo)
  local count  = 0
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
  local count  = 0
  -- We expect only tables in the lifo. Other keys will confuse this count.
  for key, value in pairs(input_lifo) do
    --print(key, " -- ", value)
    count = count + 1
  end

  return count
end







function get_copy_of_grid(input_grid)
  -- For creating copies of a grid for Undo and probably other things.
  --print ("input_grid is" .. get_tally(input_grid))
  local output_grid = create_a_grid() -- this returns a grid with the dimensions we expect
  -- copy all the key values except the ID 
  output_grid["id"] = math.random(1,99999999999999)
  output_grid["gspc"] = input_grid["gspc"]
  


  for col = 1,COLS do 
    for row = 1,ROWS do
      --print ("col:" .. col .. " row:" .. row)
      output_grid[col][row] = input_grid[col][row]
    end 
  end
  return output_grid
end  


function display_tempo_status()

  screen.clear()
  screen.update()
  
  screen.move(1,7) 
  screen.text(tempo_status_string_1)

  screen.move(1,14) 
  screen.text(tempo_status_string_2)


  screen.move(1,21) 
  screen.text(tempo_status_string_3)

  screen.move(1,28) 
  screen.text(tempo_status_string_4)

  screen.move(1,35) 
  screen.text(tempo_status_string_5)

  screen.move(1,42) 
  screen.text(co2_ppm_status_string)


  screen.move(1,49) 
  screen.text(version_string)

  --screen.font_size(10)
  screen.move(1,56) 
  
  screen.text(string.format("%.4f",current_tempo) )
  screen.update() 

 


end  


function refresh_grid_and_screen()
  
  --print ("Hello from refresh_grid_and_screen for grid at:")
  --print (grid_state)
  

  local tally = ""

  screen.clear()
  screen.move(1,1)


  if (tempo_wow_is_good == 1 and tempo_flutter_is_good == 1) then

    -- Show min stability info
    screen.move(1,7)
    screen.text("W")
    screen.move(1,14)
    screen.text(wow_tempo_episodes)
  
    screen.move(1,21)
    screen.text("F")
    screen.move(1,28)
    screen.text(flutter_tempo_episodes)

    screen.move(1,35)
    screen.text("w")
    screen.move(1,42)
    screen.text(total_wow_tempo_ticks)
    -- screen.text(string.format("%X", total_wow_tempo_ticks * 255))

    screen.move(1,49)
    screen.text("t")
    screen.move(1,56)
    screen.text(total_flutter_tempo_ticks)


  for col = 1,COLS do 
    for row = 1,TOTAL_SEQUENCE_ROWS do -- don't want to set (or display) non sequence rows here
      tally = tally .. grid_state[col][row]

      screen.move(10 + (col * 7),row * 7)
      --screen.text("table[" .. row .. "]["..col.."] is: " ..grid_state[row][column])
      

      -- Show the scrolling of the steps with the sequence rows of LEDS. (Others will be used for other controls)
      -- note: row 7 has a dual use (sequence and set midi note when a row 8 button is presssed.)
      if (current_step == col and row <= TOTAL_SEQUENCE_ROWS) then
          -- This is the scrolling cursor
          screen.text("*")
        
        if (grid_state[col][row] >= 2) then -- ratchet 
          -- If current step and key is on, highlight it.
          my_grid:led(col,row,12) 
        elseif (grid_state[col][row] == 1) then 
          -- If current step and key is on, highlight it.
          my_grid:led(col,row,9) 
        else
          -- Else use scrolling brightness
          my_grid:led(col,row,4)
        end
      else
        if (grid_state[col][row] >= 2) then
          my_grid:led(col,row,8) -- ratchet
        elseif (grid_state[col][row] == 1) then
            -- Not current step but Grid square is On
          my_grid:led(col,row,5)
        else 
            -- Not current step and key is off
          my_grid:led(col,row,0)
        end
        -- Show the stored value on screen
        screen.text(grid_state[col][row])
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

  if current_step <= 4 then
    conductor_text = "1.."
  elseif current_step > 4 and current_step <= 8 then
    conductor_text = "2["
  elseif current_step > 8 and current_step <= 12 then
    conductor_text = "3 ]"
  elseif  current_step > 12 and current_step <= 16 then 
    conductor_text = "4^^"
  end  
  
  -- print (conductor_text)


-- status_text = conductor_text .. " " .. current_tempo .. " BPM. Step " .. current_step .. " " .. end_of_line_text
-- current_tempo no decimal points
-- pad current step with a 0 so the display doesn't move about
-- https://www.cprogramming.com/tutorial/printf-format-strings.html

  status_text = string.format("%.2f",current_tempo) .. " " .. string.format("%.2d", current_step) .. " " .. last_action_method  .. " " .. string.format("%.1d", last_x) .. "," .. string.format("%.1d", last_y) .. " " .. last_grid_value .. "-" .. last_mozart_value .. " " .. conductor_text .. " "  .. end_of_line_text .. " "  
  
  
  
  screen.move(1,63)   
  screen.text(status_text)



  screen.update() -- better to have this here than in the loop above because otherwise we get screen flickering

  my_grid:refresh()
  
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
