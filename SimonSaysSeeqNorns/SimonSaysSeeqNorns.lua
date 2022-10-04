-- SimonSaysSeeq on Norns
-- Left Button Stop. Right Start
-- Licenced under the AGPL.

version = 0.2

local volts = 0
local slew = 0

-- on/off for stepped sequence 
transport_active = true


-- Dimensions of Grid
COLS = 16
ROWS = 8


first_step = 1
last_step = COLS

arm_ratchet = 0
preset_grid_button = 0
arm_clock_button = 0
preset_mozart_button = 0


arm_first_step_button = 0
arm_last_step_button = 0


arm_put_slide_on = 0
arm_take_slide_off = 0


arm_swing_button = 0

swing_mode = 1

TOTAL_SEQUENCE_ROWS = 6

GRID_STATE_FILE = "/home/we/SimonSaysSeeq-grid.tbl"

MOZART_STATE_FILE = "/home/we/SimonSaysSeeq-mozart.tbl"

SLIDE_STATE_FILE = "/home/we/SimonSaysSeeq-slide.tbl"

end_of_line_text = ""
output_text = ""

Tab = require "lib/tabutil"

MIDI_CHANNEL_GATES = 1

-- This works well with Flame MGTV factory default settings. 
--  http://flame.fortschritt-musik.de/pdf/Manual_Flame_MGTV_module_v100_eng.pdf
LOWEST_MIDI_NOTE_NUMBER_FOR_GATE = 47 -- at least 1 will be added to this.

MIDI_NOTE_ON_VELOCITY = 127
MIDI_NOTE_OFF_VELOCITY = 0


need_to_start_midi = true -- Check gate clock situation.
run_conditional_clocks = false


-- audio_clock_file = _path.dust.."code/softcut-studies/lib/whirl1.aif"

--audio_clock_file = _path.dust.."audio/SimonSaysSeeqAudio/E-RM_multiclock_sample.wav"


audio_clock_file = _path.dust.."audio/SimonSaysSeeqAudio/0-1-2-3-4-5.wav"

-- audio_clock_file = _path.dust.."audio/SimonSaysSeeqAudio/hermit_leaves.wav"

-- audio_clock_file = _path.dust.."audio/x0x/606/606-CH.wav"

PRESET_GRID_BUTTON = {x=1, y=2}

BUTTONS = {}

--7th Row
table.insert(BUTTONS, {name = "Button1", x = 1, y = 7})
table.insert(BUTTONS, {name = "Button2", x = 2, y = 7})
table.insert(BUTTONS, {name = "Button3", x = 3, y = 7})
table.insert(BUTTONS, {name = "Button4", x = 4, y = 7})

table.insert(BUTTONS, {name = "Button5", x = 5, y = 7})
table.insert(BUTTONS, {name = "Button6", x = 6, y = 7})
table.insert(BUTTONS, {name = "Button7", x = 7, y = 7})
table.insert(BUTTONS, {name = "Button8", x = 8, y = 7})

table.insert(BUTTONS, {name = "Button9", x = 9, y = 7})
table.insert(BUTTONS, {name = "Button10", x = 10, y = 7})
table.insert(BUTTONS, {name = "Button11", x = 11, y = 7})
table.insert(BUTTONS, {name = "Button12", x = 12, y = 7})

table.insert(BUTTONS, {name = "Button13", x = 13, y = 7})
table.insert(BUTTONS, {name = "Button14", x = 14, y = 7})
table.insert(BUTTONS, {name = "Button15", x = 15, y = 7})
table.insert(BUTTONS, {name = "Button16", x = 16, y = 7})


-- 8th Row
table.insert(BUTTONS, {name = "Undo", x = 1, y = 8})
table.insert(BUTTONS, {name = "Redo", x = 2, y = 8})
table.insert(BUTTONS, {name = "ArmFirstStep", x = 3, y = 8})
table.insert(BUTTONS, {name = "ArmLastStep", x = 4, y = 8})

table.insert(BUTTONS, {name = "ArmMidiCommand", x = 5, y = 8})
table.insert(BUTTONS, {name = "ArmSwing", x = 6, y = 8})
table.insert(BUTTONS, {name = "DoMidiStop", x = 7, y = 8})
table.insert(BUTTONS, {name = "DoMidiStart", x = 8, y = 8})

table.insert(BUTTONS, {name = "Ratchet2", x = 9, y = 8})
table.insert(BUTTONS, {name = "Ratchet3", x = 10, y = 8})
table.insert(BUTTONS, {name = "Ratchet4", x = 11, y = 8})
table.insert(BUTTONS, {name = "Ratchet5", x = 12, y = 8})

table.insert(BUTTONS, {name = "PresetGrid", x = 13, y = 8})
table.insert(BUTTONS, {name = "PresetMozart", x = 14, y = 8})
table.insert(BUTTONS, {name = "ArmSlideOff", x = 15, y = 8})
table.insert(BUTTONS, {name = "ArmSlideOn", x = 16, y = 8})






current_step = first_step


tick_text = "."

tick_count = 0

PPQN24_GATES_ARE_ENABLED = true


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
SWING_STEPS = Set { 2, 4, 6, 8, 10, 12, 14, 16 }


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


screen.font_face(15)
screen.font_size(7)


function tick()
  while true do

   -- In clock sync, 1 refers to a quarter note so if we clock.sync(1) we will count 4 beats per bar
   -- if we clock.sync(1/4) we will count 16 beats per bar. (16 steps in the sequence)
   -- if we clock.sync(1/24) this is 24PPQN Pulses Per Quarter Note, I.e. standard MIDI clock

   
 
     
    swing_amount = 0


    -- some kind of swing amount between zero and nearly 1/192 
    swing_amount = (swing_mode / 18) * (1 / 192) 

    if swing_mode == 1 then
      swing_amount = 0
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

    if swing_mode == 1 then
       -- No swing, normal clock
      clock.sync(1/48) -- Run at twice 24 PPQN so the even we can send gate on (for clock) and on the odd we can send gate off.
    else   
      print ("swing_mode is: " .. swing_mode)
      if SWING_STEPS[current_step] then
        print ("swinging step " .. current_step)


        clock.sync(1/48 + swing_amount)


      else
        -- Non swing step
        --clock.sync(1/48)

        print ("Not swinging step " .. current_step)

        clock.sync(1/48 - swing_amount)
  
      end    
    end  











  
    if PPQN24_GATES_ARE_ENABLED == true then

      if run_conditional_clocks == true then

        -- 24 PPQN clock -- This is a 50 50 duty cycle
        if tick_count % 2 == 0 then
          gate_on(12)


         --audio.tape_play_start()
         
          softcut.position(1,5) -- at 0 seconds there is the transient click BUT no click is produced.doesn't do much, so try at 5 seconds 1000 HZ tone, but needless to say it doesn't work
        --  softcut.level(1,1)
        --  softcut.play(1,1)


         --engine.hz(440)


        else
          gate_off(12)

         -- print("POS 1")

          softcut.position(1, 1)-- at this this position (1 second) there should be no sound

          -- softcut.play(1,1)

           -- audio.tape_play_stop()

          -- softcut.level(1,0)

          --engine.hz(1000)

        end  

      -- this is a narrower width

        if tick_count % 2 == 0 then
          clock.run(process_clock_gate, 11)
        end 

        --------

        -- This seem to overload Flame sometimes.
    
        -- if tick_count % 6 == 0 then
        --   clock.run(process_clock_gate, 10)
        -- end 

        -- if tick_count % 12 == 0 then
        --   clock.run(process_clock_gate, 9)
        -- end  

        -- if tick_count % 24 == 0 then
        --   clock.run(process_clock_gate, 8)
        -- end 

        -- if tick_count % 48 == 0 then
        --   clock.run(process_clock_gate, 8)
        -- end 

        -- if tick_count % 96 == 0 then
        --   clock.run(process_clock_gate, 8)
        -- end 

        -- if tick_count % 192 == 0 then -- since tick_count never reaches 192, this only fires at 0
        --   -- print("Modulus 192 tick_count is: " .. tick_count)
        --   clock.run(process_clock_gate, 7)
        -- end 

      end -- End conditional clocks check 

    end -- End check for 24 PPQN clocks

    -- Every 12 ticks we want to advance the sequencer (if transport is active) 
    if tick_count % 12 == 0 then

      -- print("tick_count is: " .. tick_count)

      if transport_active then 

        --softcut.play(1,1)
        do_and_advance_step() 
      end

      redraw()

    end   

    -- So tick_count doesn't get too big over the course of a long running session. (would end up slowing down modulus calcs?)
    tick_count = tick_count + 1

    if tick_count == 192 then -- Reset so we don't have too many numbers on which we do modulus calculations
      tick_count = 0
    end  

  end
end




function grid_state_popularity_watcher()
  -- get some idea of how much a particular grid is used
  -- the idea is to use this on a fast forward backward undo / redo that just scrolls through the popular states
  -- (i.e. states that have been used for a long time)
  -- WIP The idea is to have a separate undo / redo that only uses hi popularity states
  while true do
    clock.sync(1) -- do this every beat
    grid_state["gspc"] = grid_state["gspc"] + 1
    --print ("grid_state.gspc is: " .. grid_state["gspc"])
  end
end



-- Note: This effictively gets called multiple times at boot 
function greetings()

  -- presumption of success but still seems to get run multiple times.
  greetings_done = true
  
  screen.clear()

  screen.move(1,10)
  screen.text("SimonSaysSeeq on Norns v" .. version)
  
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




function do_and_advance_step()
  --print ("do_and_advance_step current_step is:  " .. current_step)

  
  local ratchet_mode = 1 -- default is 1 but it will be set

  --engine.hz(400) -- just to give some audible sign for debugging timing

  if need_to_start_midi == true then
  
    if current_step == first_step then

     --engine.hz(800) -- just to give some audible sign for debugging timing

      -- we only want to start midi clock at the right time!
      print ("Send MIDI Start current_step is: " .. current_step)
      midi_gates_device:start()
      normal_midi_device:start()
      run_conditional_clocks = true -- so our 24PPQN etc stays on when midi clock is on 
      need_to_start_midi = false

    else
      print ("Waiting to MIDI Start current_step is: " .. current_step)
    end

   end -- End check midi start



  
  -- For each sequence row...
  for output = 1, TOTAL_SEQUENCE_ROWS do
    -- on the current step...
    ratchet_mode = grid_state[current_step][output]

    -- process step should run independently
    clock.run(process_ratchet, output, ratchet_mode)

    -- Sent appropriate midi note out as cv
    
    -- To quote dan_dirks, "any MIDI note number divided by 12 is how the pitch is expressed in voltage (assuming volt per octave)"
    -- https://llllllll.co/t/frequencies-and-cv-converting-back-and-forth-in-lua-math-math-math/50984

    -- Send the midi note number as CV we have previously captured (this currently sends even if the step is not active)

    -- We have 4 outputs on crow
    -- Here we check the slide and set the voltage to the pitch accordingly.

  

    if output == 3 then
      if slide_state[current_step][output] == 1 then
        crow.output[1].slew = 0.1
      else
        crow.output[1].slew = 0
      end  
      crow.output[1].volts = mozart_state[current_step][output] / 12 
      output_text = output .. mozart_state[current_step][output] -- show on display 
      

    elseif output == 4 then
      if slide_state[current_step][output] == 1 then
        crow.output[2].slew = 0.1
      else
        crow.output[2].slew = 0
      end  
      crow.output[2].volts = mozart_state[current_step][output] / 12
      
      output_text = output_text .. " " .. output ..  mozart_state[current_step][output] 



    elseif output == 5 then
      if slide_state[current_step][output] == 1 then
        crow.output[3].slew = 0.1
      else
        crow.output[3].slew = 0
      end  
      crow.output[3].volts = mozart_state[current_step][output] / 12  
      
      output_text = output_text .. " " ..  output ..  mozart_state[current_step][output] 

    elseif output == 6 then
      if slide_state[current_step][output] == 1 then
        crow.output[4].slew = 0.1
      else
        crow.output[4].slew = 0
      end  
      crow.output[4].volts = mozart_state[current_step][output] / 12  

      output_text = output_text .. " " ..  output ..  mozart_state[current_step][output] 

    end  


  end -- end for

  -- advance step
  current_step = util.wrap(current_step + 1, first_step, last_step)

  
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

  end -- end non zero

 
  
end  

-- MUST be run as clock.run(process_clock_gate, output) - so syncs are independent
function process_clock_gate (output)

  gate_on(output)
  clock.sync(1/64)
  gate_off(output)
 
end 

 
function gate_on(output)
       --print ("A ON LOWEST_MIDI_NOTE_NUMBER_FOR_GATE" .. LOWEST_MIDI_NOTE_NUMBER_FOR_GATE .. " MIDI_NOTE_ON_VELOCITY " .. MIDI_NOTE_ON_VELOCITY .. " sequence_row + MIDI_CHANNEL_GATES " .. sequence_row + MIDI_CHANNEL_GATES)
  midi_gates_device:note_on (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + output, MIDI_NOTE_ON_VELOCITY, MIDI_CHANNEL_GATES)
end 
  
function gate_off(output)
  --print ("A OFF LOWEST_MIDI_NOTE_NUMBER_FOR_GATE" .. LOWEST_MIDI_NOTE_NUMBER_FOR_GATE .. " MIDI_NOTE_OFF_VELOCITY " .. MIDI_NOTE_OFF_VELOCITY .. " sequence_row + MIDI_CHANNEL_GATES " .. sequence_row + MIDI_CHANNEL_GATES)
  midi_gates_device:note_off (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + output, MIDI_NOTE_OFF_VELOCITY, MIDI_CHANNEL_GATES)
end 



function clock.transport.start() -- when is this called?
  print("transport.start")

  screen.clear()

  transport_active = true
  
  screen.move(90,63)
  screen.text("Start")
  screen.update()
end

function request_midi_start()
  print("request_midi_start")
  need_to_start_midi = true
end  
 


function clock.transport.stop()
  print("transport.stop")

  screen.clear()

  transport_active = false
  screen.move(80,80)
  screen.text("STOP")
  screen.update()
end



function request_midi_stop()
  print("request_midi_stop")
  need_to_start_midi = false
  -- can stop the midi clock at any time.
  midi_gates_device:stop ()
  normal_midi_device:stop ()

  gate_off(6)
  gate_off(7)
  gate_off(8)
  gate_off(9)
  gate_off(10)
  gate_off(11) 


  run_conditional_clocks = false
end  

function enc(n,d)
  if n==3 then
     params:delta("clock_tempo", d)
  end
end 


-- Norns (Shield) key presses - (not the monome grid )
function key(n,z)
  print("key pressed.  n:" .. n ..  " z:" .. z )
  
  -- since MIDI and Link offer their own start/stop messages,
  -- we'll only need to manually start if using internal or crow clock sources:
  if params:string("clock_source") == "internal" then

    -- left button pressed 
    if n == 2 and z == 1 then

      -- try moving pointer to quiet part of

    -- softcut.tape_play_stop ()

      if transport_active then -- currently running so Stop     
        clock.transport.stop()
        request_midi_stop()

        
      else -- Not currently running so reset. 
        current_step = first_step -- effectively we press this again.

      end
      
      screen_dirty = true
    end
    
    -- Right button pressed
    if n == 3 and z == 1 then

      -- try moving pointer to loud part of sample

    --  softcut.tape_play_start ()


      if not transport_active then
        clock.transport.start()
        request_midi_start() -- Just send MIDI start instead of requesting?

        

      end
      
      screen_dirty = true
    end
    
  end
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

  print ("grid_button_function_name says Bye. I will return: " .. ret)
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
  -- softcut.buffer_read_mono(audio_clock_file,0,1,-1,1,1)
  


  softcut.buffer_read_stereo(audio_clock_file, 1, 1, -1)


  -- audio.tape_play_open (audio_clock_file)


  -- enable voice 1
  softcut.enable(1,1)
  -- set voice 1 to buffer 1
  softcut.buffer(1,1)
  -- set voice 1 level to 1.0
  softcut.level(1,1.0)
  
  
  
  -- voice 1  loop
   softcut.loop(1,1) -- This MUST be on ??
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
  undo_lifo = {}
  redo_lifo = {}

 
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
  my_grid:all(2)
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
  params:set("clock_tempo",136)
  
  --midi_start_on_bar_id = clock.run(midi_start_on_bar)

  -- print ("******START***************")
  -- print(grid_button_function_name (11,8))
  -- print(grid_button_function_name (2,1))
  -- print ("========END============")

-- here
--crow.output[1].scale = {0,7,2,9}




   clock.run(tick)       -- start the sequencer


  --  clock.run(function()  -- redraw the screen and grid at 15fps (maybe refresh grid at a different rate?)
  --   while true do
  --     clock.sleep(1/15)
  --     redraw()
  --   end
  -- end)


  print_audio_file_info(audio_clock_file)


  end -- end init



 


 clock.run(grid_state_popularity_watcher) 

 




 -- Periodically check if we need to save the grid state to file.
 -- TODO - if we're not saving this when running then might as well just save it when we stop (rather than have a loop)
  clock.run(function()
    while true do
      clock.sleep(5)
        if (grid_state_dirty == true) then

           if (transport_active == false) then -- only save if we're stopped. (not sure we really need this)

            Tab.save(grid_state, GRID_STATE_FILE)

            -- for now do mozart at the same time.
            Tab.save(mozart_state, MOZART_STATE_FILE)

            -- for now do mozart at the same time.
            Tab.save(slide_state, SLIDE_STATE_FILE)


            grid_state_dirty = false
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



 
-- grid is for storing our gates / ratches
function create_grid()
  local fresh_grid = {}
  fresh_grid["id"]=math.random(1,99999999999999) -- an ID for debugging purposes
  fresh_grid["gspc"]=0 -- we can increment this to see how popular this grid is
  for col = 1, COLS do 
    fresh_grid[col] = {} -- create a table for each col
    for row = 1, ROWS do
      if col == row then -- eg. if coordinate is (3,3)
        fresh_grid[col][row] = 1
      else -- eg. if coordinate is (3,2)
        fresh_grid[col][row] = 0
      end
    end
  end
  return fresh_grid
end  


-- mozart is for storing our midi notes
function create_mozart()
  local fresh_mozart = {}
  fresh_mozart["id"]=math.random(1,99999999999999) -- an ID for debugging purposes
  fresh_mozart["gspc"]=0 -- we can increment this to see how popular this mozart is
  for col = 1, COLS do 
    fresh_mozart[col] = {} -- create a table for each col
    for row = 1, ROWS do
      if col == row then -- eg. if coordinate is (3,3)
        fresh_mozart[col][row] = 1
      else -- eg. if coordinate is (3,2)
        fresh_mozart[col][row] = 0
      end
    end
  end
  return fresh_mozart
end 

-- slide is for storing our slide / gliss.
function create_slide()
  local fresh_slide = {}
  fresh_slide["id"]=math.random(1,99999999999999) -- an ID for debugging purposes
  fresh_slide["gspc"]=0 -- we can increment this to see how popular this mozart is
  for col = 1, COLS do 
    fresh_slide[col] = {} -- create a table for each col
    for row = 1, ROWS do
      if col == row then -- eg. if coordinate is (3,3)
        fresh_slide[col][row] = 1
      else -- eg. if coordinate is (3,2)
        fresh_slide[col][row] = 0
      end
    end
  end
  return fresh_slide
end 


-- held is for storing grid buttons that are held down 
function create_held()
  local fresh_held = {}
  fresh_held["id"]=math.random(1,99999999999999) -- an ID for debugging purposes
  for col = 1, COLS do 
    fresh_held[col] = {} -- create a table for each col
    for row = 1, ROWS do
        fresh_held[col][row] = 0
    end
  end
  return fresh_held
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

    grid_state = create_grid()



    Tab.save(grid_state, GRID_STATE_FILE)
    grid_state = Tab.load (GRID_STATE_FILE)  
  else
    print ("I already have a grid_state table, no need to generate one")
  end
  
 print ("grid tally is: " .. get_tally(grid_state))

 print ("clock.get_tempo() is: " .. clock.get_tempo())
 
 


  -- Push Undo so we can get back to initial state
  push_undo()

  
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

    mozart_state = create_mozart()



    Tab.save(mozart_state, MOZART_STATE_FILE)
    mozart_state = Tab.load (MOZART_STATE_FILE)  
  else
    print ("I already have a mozart_state table, no need to generate one")
  end
  
 print ("tally is: " .. get_tally(mozart_state))

 print ("clock.get_tempo() is: " .. clock.get_tempo())
 

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

    slide_state = create_slide()



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
  held_state = create_held()

  
 --print ("tally is: " .. get_tally(held_state))

  
  print ("Bye from init_held_state_table")
  

end -- end init_held_state_table







--------------
-----

function push_undo()

  --print("push_undo says hello. Store Undo LIFO")
  -- TODO check memory / count of states? - if this gets very large, truncate from the other side

  -- When we push to the undo_lifo, we want to *copy* the grid_state (not reference) so that any subsequent changes to grid_state are not saved on the undo_lifo 
  -- Inserts in the last position of the table (push)
  table.insert (undo_lifo, get_copy_of_grid(grid_state))

  --print ("undo_lifo size is: ".. lifo_size(undo_lifo))

end  

function pop_undo()

  if lifo_size(undo_lifo) > 1 then

    -- 2) Pop from the undo_lifo to the current state.
    -- Removes from the last element of the table (pop)
    local undo_state = table.remove (undo_lifo)

    grid_state = get_copy_of_grid(undo_state)

    --print ("undo_lifo size is: ".. lifo_size(undo_lifo))
    
    -- Thus if A through G are all the states we've seen, and E is the current state, we'd have the following:
    
    --    ABCDEFG
    --        *
    -- grid_state: E
    --
    -- undo_lifo       redo_lifo
    --    D                F 
    --    C                G 
    --    B
    --    A 

  else 
    print ("Not poping last undo_lifo because its size is 1 or less ")  
  end
end 




function push_redo()
    -- 1) Push the current state to the redo_lifo so we can get back to it.
    -- Similarly we want to *copy* the grid_state (not reference) 
    -- so any subsequent changes to the grid_state are not reflected in the redo_lifo
    table.insert (redo_lifo, get_copy_of_grid(grid_state))
    
    --print ("redo_lifo size is: ".. lifo_size(redo_lifo))
end  


function pop_redo()

  if lifo_size(redo_lifo) > 1 then

      -- 2) Pop the redo_lifo into the current state.

      -- TODO need to copy this?
      local redo_state = table.remove (redo_lifo) 
      grid_state = get_copy_of_grid(redo_state)

      --print ("redo_lifo size is: ".. lifo_size(redo_lifo))
    else 
      print ("Not poping last redo_lifo because its size is 1 or less ")  
    end

end  


function set_mozart_and_grid_based_on_held_key(midi_note_number)
  for col = 1,COLS do 
    for row = 1,ROWS do
      -- if a step is held, assign the captured midi note to it.
      if held_state[col][row] == 1 then
        mozart_state[col][row] = midi_note_number -- Note we don't have any note off

        -- As we have just "put a midi note on the step", make the step on.
        grid_state[col][row] = 1
      end  
    end 
  end
end  


function conditional_set_mozart(x, y, z, midi_note_number)

  if z == 1 then 
    midi_note_key_pressed = midi_note_number
    set_mozart_and_grid_based_on_held_key(midi_note_number)
  else
    midi_note_key_pressed = -1
  end 

  -- As we have just "put a midi note on the step", make the step on.
  grid_state[x][y] = 1 -- this not working, something is overriding it

end

-- probably not used
midi_gates_device.event = function(data)
  print("---------------------- midi_gates_device IN ---------------------------------------")
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
    
    -- Now check if a step button is pressed down


    --loop through held_state



    -- for col = 1,COLS do 
    --   for row = 1,ROWS do
    --     -- if a step is held, assign the captured midi note to it.
    --     if held_state[col][row] == 1 then
    --       mozart_state[col][row] = captured_normal_midi_note_in -- Note we don't have any note off

    --       -- As we have just "put a midi note on the step", make the step on.
    --       grid_state[col][row] = 1
    --     end  
    --   end 
    -- end


        -- replaces the above 
    set_mozart_and_grid_based_on_held_key(captured_normal_midi_note_in)



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


-- Here we capture monome grid key presses - Grid Key Presses
my_grid.key = function(x,y,z)
-- x is the column
-- y is the row
-- z == 1 means key down, z == 0 means key up
-- We want to capture the key down event and toggle the state of the key in the grid.

print("Hello from ----------- my_grid.key = function -----------------")
print(x .. ","..y .. " z is " .. z.. " value before change " .. grid_state[y][y])

print("preset_grid_button is: " .. preset_grid_button .. ", arm_ratchet is: ".. arm_ratchet .. " captured_normal_midi_note_in is: " ..  captured_normal_midi_note_in .. " preset_mozart_button is: " .. preset_mozart_button .. " midi_note_key_pressed is: " .. midi_note_key_pressed)
--print(x .. ","..y .. " z is " .. z)

-- Reset. It might get set below
-- midi_note_key_pressed = -1


-- To note the keys that are held down 
if z == 1 then
  held_state[x][y] = 1
else
  held_state[x][y] = 0  
end




-- We treat sequence rows and control rows different.

-- If one of the SEQUENCE ROWS (not the bottom control rows)
if y <= TOTAL_SEQUENCE_ROWS then
  -- For sequence rows we only want to capture key down event only  
  if z == 1 then


    -- *Order is important here*. 
    -- We want to save the current state *BEFORE* we push a copy of the grid to the undo lifo
    -- But only do this if we are not touching the control rows (7 & 8)

    --print ("Sequence button pressed")
    --print ("grid_state:" .. get_tally(grid_state))

    -- Every time we change state of sequence rows (non control rows), record the new state in the undo_lifo
    push_undo()

    -- So we save the table to file
    -- (don't bother with control rows)
    --print ("Before set grid_state_dirty = true")
    grid_state_dirty = true

    if preset_grid_button ~= 0 then
      print ("preset_grid_button is: " .. preset_grid_button .. ", x is: " .. x .. ", y is: " .. y) 


      -- Any button pressed on this row (1)
      if y == 1 then

        print ("Setting preset for row: " .. x) 

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

       -- Any button pressed on this row (2)  
      elseif y == 2 then

        print ("Setting preset for row: " .. x) 

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

      elseif y == 3 then

        print ("Setting preset for row: " .. x) 

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

      elseif y == 4 then

        print ("Setting preset for row: " .. x) 

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


      elseif y == 5 then

        print ("Setting preset for row: " .. x) 

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

      elseif y == 6 then

        print ("Setting preset for row: " .. x) 

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


      end  

    elseif preset_mozart_button ~= 0 then

      -- set all rows to MIDI note A4 (treat all the sequence rows the same.)

      mozart_state[1][y] = 33 -- A2
      mozart_state[2][y] = 33
      mozart_state[3][y] = 33
      mozart_state[4][y] = 33

      mozart_state[5][y] = 33
      mozart_state[6][y] = 33
      mozart_state[7][y] = 33
      mozart_state[8][y] = 33

      mozart_state[9][y]  = 33
      mozart_state[10][y] = 33
      mozart_state[11][y] = 33
      mozart_state[12][y] = 33

      mozart_state[13][y] = 33
      mozart_state[14][y] = 33
      mozart_state[15][y] = 33
      mozart_state[16][y] = 33



    else -- preset button is not pressed 

      --print ("Before changing grid_state for sequence rows based on key press and ratchet button.")
      --print ("arm_ratchet is: " .. arm_ratchet)

      -- *Order is important here*. 
      -- Change the state of the grid *AFTER* we have pushed to undo lifo
      
      -- Place RATCHET on step
      if arm_ratchet ~= 0 then -- We place the ratchet on the step
        grid_state[x][y] = arm_ratchet
      else


        -- Conditions under which we want to TOGGLE the grid_state
        -- I.e. we don't want to change the state of the main grid (steps) if we are doing something else like adding a note / slide or changing last step.
        if arm_ratchet == 0 and captured_normal_midi_note_in == -1 and arm_put_slide_on == 0 and arm_take_slide_off == 0  and arm_first_step_button == 0 and arm_last_step_button == 0 and arm_swing_button == 0 then

          -- This TOGGLES the grid states i.e. because z=1 push on/off push off/on etc.
          if grid_state[x][y] ~= 0 then -- "on" might be 1 or something else if its a ratchet etc.
            grid_state[x][y] = 0
          else 
            grid_state[x][y] = 1
          end

        else -- no arm behaviour active.

          -- it makes sense to turn the step on (if its off) if we're putting a midi note / slide / ratchet on it etc. (only exception might be first / last step but currently they operate on row 7)
          if grid_state[x][y] == 0 then 
            grid_state[x][y] = 1
          end

        end

    end -- ratchet test

        --print ("After changing grid_state for sequence rows based on key press and ratchet button.")

       -- in this code path, we captured a midi note and then pressed a step button. 

       if captured_normal_midi_note_in ~= -1 then
        print ("Set mozart_state MIDI note via MIDI input" .. captured_normal_midi_note_in .. " on x: " .. x .. " y: " .. y )

        -- Store the latest captured midi note in the mozart table
        mozart_state[x][y] = captured_normal_midi_note_in
        
       else
        print ("Not doing anything to mozart_state via MIDI input")  
        -- print ("captured_normal_midi_note_in is  " .. captured_normal_midi_note_in) 
       end 



      if midi_note_key_pressed ~= -1 then
        print ("Set mozart_state MIDI note via key press " .. midi_note_key_pressed .. " on x: " .. x .. " y: " .. y )

        -- Store the latest captured midi note in the mozart table
        mozart_state[x][y] = midi_note_key_pressed
      else
        print ("Not doing anything to mozart_state via key press")  
        -- print ("midi_note_key_pressed is  " .. midi_note_key_pressed) 
      end 








       
       
      if arm_put_slide_on == 1 then
        slide_state[x][y] = 1
      end  
        
      if arm_take_slide_off == 1 then
        slide_state[x][y] = 0
      end 
    

    end -- preset_grid_button test

--print ("End of key down test")

  end -- End of key down test

  --print(x .. ","..y .. " value after change " .. grid_state[y][y])
  --print("--------------------------------------------------------------")




else

  -- CONTROL ROWS


-- direct assignement of control rows.
    -- if the button is pressed it should be lit etc.
    -- we need to modifiy the code so grid_state control rows doesn't get set by undo
    -- Gives feedback to user - other function? TODO clarify purpose of setting this.
    grid_state[x][y] = z

    -- RATCHETS
    -- Holding one of the ratchet buttons and a step will put the ratchet "on" the step
    
    print ("Some CONTROL ROW BUTTON PRESSED " .. z)
    

    if grid_button_function_name(x,y) == "Ratchet2" then
      --print ("RATCHET button pressed: " .. z)
      if z == 1 then 
        arm_ratchet = 2
      else
        arm_ratchet = 0 -- Reset arm_ratchet with a key up
      end
    elseif grid_button_function_name(x,y) == "Ratchet3" then
      --print ("RATCHET button pressed: " .. z)
      if z == 1 then 
        arm_ratchet = 3
      else
        arm_ratchet = 0
      end
    elseif grid_button_function_name(x,y) == "Ratchet4" then
      --print ("RATCHET button pressed: " .. z)
      if z == 1 then 
        arm_ratchet = 4
      else
        arm_ratchet = 0
      end
    elseif grid_button_function_name(x,y) == "Ratchet5" then
      --print ("RATCHET button pressed: " .. z)
      if z == 1 then 
        arm_ratchet = 5
      else
        arm_ratchet = 0
      end
    
  

-- Preset buttons
-- These buttons are used to put a preset on one of the sequence rows

  elseif grid_button_function_name(x,y) == "PresetGrid" then
  --print ("Preset button pressed: " .. z)
  if z == 1 then
    print ("Preset button pressed: " .. z) 
    preset_grid_button = 2
  else
    print ("Preset button RESET: " .. z) 
    preset_grid_button = 0 -- Reset preset_grid_button with a key up
  end
elseif grid_button_function_name(x,y) == "PresetMozart" then
  --print ("Preset button pressed: " .. z)
  if z == 1 then 
    preset_mozart_button = 2
  else
    preset_mozart_button = 0
  end
elseif grid_button_function_name(x,y) == "Preset4" then -- currently no button for this
  --print ("Preset button pressed: " .. z)
  if z == 1 then 
    preset_grid_button = 4
  else
    preset_grid_button = 0
  end
elseif grid_button_function_name(x,y) == "Preset5" then -- currently no button for this
  --print ("Preset button pressed: " .. z)
  if z == 1 then 
    preset_grid_button = 5
  else
    preset_grid_button = 0
  end




-- Require alter clock button be pressed so that we don't stop / start the clock acceidentally

elseif grid_button_function_name (x,y) == "ArmMidiCommand" then
  if z == 1 then 
    arm_clock_button = 1
  else
    arm_clock_button = 0
  end


elseif grid_button_function_name (x,y) == "DoMidiStop" then
  if z == 1 and arm_clock_button == 1 then 
    request_midi_stop()
  end


elseif grid_button_function_name (x,y) == "DoMidiStart" then
  if z == 1 and arm_clock_button == 1 then 
    need_to_start_midi = true
  end



--

    -- UNDO
  elseif grid_button_function_name (x,y) == "Undo" then
      ----------
      -- UNDO --
      ----------
      --print ("Pressed 1,8: UNDO")


      -- Only do this if we know we can pop from undo 
      if (lifo_populated(undo_lifo)) then

        --print ("undo_lifo is populated")
      
        -- In order to Undo we: 


        -- local tally = refresh_grid_and_screen()
        -- print ("grid_state BEFORE push_redo is:")
        -- print (grid_state)
        -- print ("tally is:" .. tally)

        push_redo()

        -- local tally = refresh_grid_and_screen()
        -- print ("grid_state BEFORE pop_undo is:")
        -- print (grid_state)
        -- print ("tally is:" .. tally)



        pop_undo()

        -- local tally = refresh_grid_and_screen()
        -- print ("grid_state AFTER pop_undo is:")
        -- print (grid_state)
        --print ("grid_state: " .. get_tally(grid_state))


    
        -- print ("grid_state is:")
        -- print (grid_state)

      else
        print ("undo_lifo is NOT populated")
      end


     -- End of UNDO

    -- CONTROL REDO    
    elseif grid_button_function_name (x,y) == "Redo" then
      -- REDO  
      -- print ("Pressed 2,8: REDO")
      -- local tally = refresh_grid_and_screen()
      -- print ("grid_state BEFORE push_undo is:")
      -- print (grid_state)
      -- print ("tally is:" .. tally)

          -- Only do this if we know we can pop from undo 
      if (lifo_populated(redo_lifo)) then
        -- print ("redo_lifo is populated")

        push_undo()

        -- local tally = refresh_grid_and_screen()
        -- print ("grid_state BEFORE pop_redo is:")
        -- print (grid_state)
        -- print ("tally is:" .. tally)
        pop_redo()

      --  refresh_grid_and_screen()

        -- local tally = refresh_grid_and_screen()
        -- print ("grid_state AFTER pop_redo is:")
        -- print (grid_state)
        -- print ("tally is:" .. tally)

        --print ("grid_state: " .. get_tally(grid_state))
      else
        print ("redo_lifo is NOT populated")
      end  
    --end -- End of REDO



  elseif grid_button_function_name (x,y) == "ArmSlideOn" then

    if z == 1 then 
      arm_put_slide_on = 1
    else
      arm_put_slide_on = 0
    end

elseif grid_button_function_name (x,y) == "ArmSlideOff" then

    if z == 1 then 
      arm_take_slide_off = 1
    else
      arm_take_slide_off = 0
    end


  elseif grid_button_function_name (x,y) == "ArmFirstStep" then
    if z == 1 then 
      arm_first_step_button = 1
    else
      arm_first_step_button = 0
    end

  elseif grid_button_function_name (x,y) == "ArmLastStep" then
    if z == 1 then 
      arm_last_step_button = 1
    else
      arm_last_step_button = 0
    end

  elseif grid_button_function_name (x,y) == "ArmSwing" then
    if z == 1 then 
      arm_swing_button = 1
    else
      arm_swing_button = 0
    end


 -- Set first_step 
 -- (TODO first and last step should be independent of row so we can have phasing) 
 -- (This would require separe step counters)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button1" )  then
    set_first_step(1)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button2" )  then
    set_first_step(2)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button3" )  then
    set_first_step(3)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button4" )  then
    set_first_step(4)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button5" )  then
    set_first_step(5)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button6" )  then
    set_first_step(6)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button7" )  then
    set_first_step(7)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button8" )  then
    set_first_step(8)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button9" )  then
    set_first_step(9)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button10" )  then
    set_first_step(10)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button11" )  then
    set_first_step(11)             
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button12" )  then
    set_first_step(12)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button13" )  then
    set_first_step(13)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button14" )  then
    set_first_step(14)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button15" )  then
    set_first_step(15)
  elseif (arm_first_step_button == 1 and grid_button_function_name (x,y) == "Button16" )  then
    set_first_step(16)
    
  -- Set last_step
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button1" )  then
    set_last_step(1)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button2" )  then
    set_last_step(2)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button3" )  then
    set_last_step(3)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button4" )  then
    set_last_step(4)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button5" )  then
    set_last_step(5)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button6" )  then
    set_last_step(6)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button7" )  then
    set_last_step(7)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button8" )  then
    set_last_step(8)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button9" )  then
    set_last_step(9)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button10" )  then
    set_last_step(10)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button11" )  then
    set_last_step(11)             
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button12" )  then
    set_last_step(12)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button13" )  then
    set_last_step(13)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button14" )  then
    set_last_step(14)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button15" )  then
    set_last_step(15)
  elseif (arm_last_step_button == 1 and grid_button_function_name (x,y) == "Button16" )  then
    set_last_step(16) 

  -- Set Swing
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button1" )  then
    swing_mode = 1 
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button2" )  then
    swing_mode = 2 
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button3" )  then
    swing_mode = 3 
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button4" )  then
    swing_mode = 4 
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button5" )  then
    swing_mode = 5 
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button6" )  then
    swing_mode = 6   
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button7" )  then
    swing_mode = 7 
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button8" )  then
    swing_mode = 8 
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button9" )  then
    swing_mode = 9 
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button10" )  then
    swing_mode = 10 
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button11" )  then
    swing_mode = 11 
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button12" )  then
    swing_mode = 12
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button13" )  then
    swing_mode = 13 
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button14" )  then
    swing_mode = 14 
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button15" )  then
    swing_mode = 15   
  elseif (arm_swing_button == 1 and grid_button_function_name (x,y) == "Button16" )  then
    swing_mode = 16 

  -- Place MIDI note on sequence notes
-- Handles the following key combination: First press and hold a sequence note, then press one of these buttons to put a MIDI note on the button


elseif (grid_button_function_name (x,y) == "Button1") then
  print("button" .. 1)
  conditional_set_mozart(x, y, z, 33)
elseif (grid_button_function_name (x,y) == "Button2") then
  print("button" .. 2)
  conditional_set_mozart(x, y, z, 34)
elseif (grid_button_function_name (x,y) == "Button3") then
  print("button" .. 3)
  conditional_set_mozart(x, y, z, 35)
elseif (grid_button_function_name (x,y) == "Button4") then
  print("button" .. 4)
  conditional_set_mozart(x, y, z, 36)
elseif (grid_button_function_name (x,y) == "Button5") then
  print("button" .. 5)
  conditional_set_mozart(x, y, z, 37)
elseif (grid_button_function_name (x,y) == "Button6") then
  print("button" .. 6)
  conditional_set_mozart(x, y, z, 38)
elseif (grid_button_function_name (x,y) == "Button7") then
  print("button" .. 7)
  conditional_set_mozart(x, y, z, 39)
elseif (grid_button_function_name (x,y) == "Button8") then
  print("button" .. 8)
  conditional_set_mozart(x, y, z, 40)
elseif (grid_button_function_name (x,y) == "Button9") then
  print("button" .. 9)
  conditional_set_mozart(x, y, z, 41)
elseif (grid_button_function_name (x,y) == "Button10") then
  print("button" .. 10)
  conditional_set_mozart(x, y, z, 42)
elseif (grid_button_function_name (x,y) == "Button11") then
  print("button" .. 11)             
  conditional_set_mozart(x, y, z, 43)
elseif (grid_button_function_name (x,y) == "Button12") then
  print("button" .. 12)
  conditional_set_mozart(x, y, z, 44)
elseif (grid_button_function_name (x,y) == "Button13") then
  print("button" .. 13)
  conditional_set_mozart(x, y, z, 45)
elseif (grid_button_function_name (x,y) == "Button14") then
  print("button" .. 14)
  conditional_set_mozart(x, y, z, 46)
elseif (grid_button_function_name (x,y) == "Button15") then
  print("button" .. 15)
  conditional_set_mozart(x, y, z, 47)
elseif (grid_button_function_name (x,y) == "Button16") then
  print("button" .. 16) 
  conditional_set_mozart(x, y, z, 48)
  end -- end of grid_button_function_name tests

end  -- End of Sequence / Control

  -- Always do this else results are not shown to user.
  refresh_grid_and_screen()


end -- End of my_grid.key function definition

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
  local output_grid = create_grid() -- this returns a grid with the dimensions we expect
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



function refresh_grid_and_screen()
  
  --print ("Hello from refresh_grid_and_screen for grid at:")
  --print (grid_state)
  

  local tally = ""

  screen.clear()
  screen.move(1,1)
  
  for col = 1,COLS do 
    for row = 1,ROWS do
      tally = tally .. grid_state[col][row]

      screen.move(col * 7,row * 7)
      --screen.text("table[" .. row .. "]["..col.."] is: " ..grid_state[row][column])
      

      -- Show the scrolling of the steps with the first 6 rows of LEDS. (Others will be used for other controls)
      if (current_step == col and row <= 6) then
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
  status_text = string.format("%.0f",current_tempo) .. " " .. string.format("%.2d", current_step) .. " " .. end_of_line_text .. " " .. conductor_text 
  
  screen.move(1,63)   
  screen.text(status_text)

  screen.font_face(15)
screen.font_size(7)

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




-- function send_gates(sequence_row, mode)
--   -- This function sends a gate or multiple gates for the row 
--   -- The ratchet level / timing is determined by mode.
--   -- Experimental so far.

--   -- Mode == 1 means a normal gate
  

--   local count = mode
--   local timing = 1/32

--   if mode == 2 then  
--     timing = 1/32
--   elseif mode == 3 then
--     timing = 3/32
--   elseif mode == 4 then
--     timing = 3/64
--   end  

--   if mode > 1 then
--     --print("this is supposed to be a ratchet")
--   end 

--   for i=1,count do
--     --print("before do pulse")

--       --print ("A ON LOWEST_MIDI_NOTE_NUMBER_FOR_GATE" .. LOWEST_MIDI_NOTE_NUMBER_FOR_GATE .. " MIDI_NOTE_ON_VELOCITY " .. MIDI_NOTE_ON_VELOCITY .. " sequence_row + MIDI_CHANNEL_GATES " .. sequence_row + MIDI_CHANNEL_GATES)
--       midi_gates_device:note_on (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE, MIDI_NOTE_ON_VELOCITY, sequence_row + MIDI_CHANNEL_GATES)
     
--       --print ("A OFF LOWEST_MIDI_NOTE_NUMBER_FOR_GATE" .. LOWEST_MIDI_NOTE_NUMBER_FOR_GATE .. " MIDI_NOTE_OFF_VELOCITY " .. MIDI_NOTE_OFF_VELOCITY .. " sequence_row + MIDI_CHANNEL_GATES " .. sequence_row + MIDI_CHANNEL_GATES)
--       midi_gates_device:note_off (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE, MIDI_NOTE_OFF_VELOCITY, sequence_row + MIDI_CHANNEL_GATES)

--     if mode ~= 1 then
--       clock.sync( timing )
--     end
--     --print("after sync") 
--   end

-- end  

---



function gate_low(sequence_row)
  --print("gate_low " .. output)
    midi_gates_device:note_off (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE, MIDI_NOTE_OFF_VELOCITY, sequence_row + MIDI_CHANNEL_GATES)
end



