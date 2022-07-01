-- SimonSaysSeeq on Norns
-- Left Button Stop. Right Start

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

ratchet_button = 0
preset_button = 0
arm_clock_button = 0


arm_first_step_button = 0
arm_last_step_button = 0

swing_mode = 0

TOTAL_SEQUENCE_ROWS = 6

GRID_STATE_FILE = "/home/we/SimonSaysSeeq-grid.tbl"

Tab = require "lib/tabutil"

MIDI_CHANNEL_GATES = 1
--MIDI_CHANNEL_B_OFFSET = -4 -- negative offset becuase both betweeners will listen on change 1 - 4

-- This works well with Flame MGTV factory default settings. 
--  http://flame.fortschritt-musik.de/pdf/Manual_Flame_MGTV_module_v100_eng.pdf
LOWEST_MIDI_NOTE_NUMBER_FOR_GATE = 47 -- at least 1 will be added to this.

MIDI_NOTE_ON_VELOCITY = 127
MIDI_NOTE_OFF_VELOCITY = 0


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

table.insert(BUTTONS, {name = "MidiClockArm", x = 5, y = 8})
-- Gap
table.insert(BUTTONS, {name = "MidiClockStop", x = 7, y = 8})
table.insert(BUTTONS, {name = "MidiClockStart", x = 8, y = 8})

table.insert(BUTTONS, {name = "Ratchet2", x = 9, y = 8})
table.insert(BUTTONS, {name = "Ratchet3", x = 10, y = 8})
table.insert(BUTTONS, {name = "Ratchet4", x = 11, y = 8})
table.insert(BUTTONS, {name = "Ratchet5", x = 12, y = 8})

table.insert(BUTTONS, {name = "Preset2", x = 13, y = 8})
table.insert(BUTTONS, {name = "Preset3", x = 14, y = 8})
table.insert(BUTTONS, {name = "PopUndo", x = 15, y = 8})
table.insert(BUTTONS, {name = "PopRedo", x = 16, y = 8})






current_step = first_step


tick_text = "."

tick_count = 1
step_on_one = 1


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
MIDI_NORMAL_PORT = 2


midi_gates_out = midi.connect(MIDI_GATES_PORT)
midi_normal = midi.connect(MIDI_NORMAL_PORT)




-- psudo random for our grid ids
math.randomseed( os.time() )
math.random() -- call a few times so it gets more random (apparently)
math.random() 
math.random()

engine.name = 'PolyPerc'




-- system clock tick
-- this function is started by init() and runs forever
-- if transport_active is on, it steps forward on each clock tick
-- tempo is controlled via the global clock, which can be set in the PARAMETERS menu 
-- instead of tick = function()
function tick()
  while true do
    clock.sync(1/4) 

    tick_count = util.wrap(tick_count + 1, 1, 16)
    step_on_one = util.wrap(step_on_one + 1, 1, 4)




    --if step_on_one == 1 then 
      if transport_active then 
        do_and_advance_step() 
      else
  
        if tick_text == "." then
          tick_text = ".."
        elseif tick_text == ".." then   
          tick_text = "..."
        elseif tick_text == "..." then   
          tick_text = "...."
        else
          tick_text = "."  
        end

        screen.move(80,63)
        screen.text(tick_text)
        screen.update()       
      end

    redraw()

  end
end


-- function midi_start_on_bar()
--   while true do
--     clock.sync(4) -- This makes this run every 4 crotchets / quarter notes e.g. every 4/4 bar.
--     if need_to_start_midi == true then
--       -- we only want to start midi clock at the right time!
--       midi_gates_out:start()
--       midi_gates_out_b:start()
--       midi_normal:start()
--       need_to_start_midi = false
--     end
--   end
-- end

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



function greetings()
  
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

  print ("midi.devices are:")

  for key, value in pairs(midi.devices) do

    local midi_text = ""

    print(key, " -- ", value)
    for sub_key, sub_value in pairs(value) do
      print("  " .. sub_key, " -- ", sub_value)
      
      if sub_key == "port" then
        midi_text = midi_text .. "Port " .. sub_value 
      end

      if sub_key == "name" then
        midi_text = midi_text .. ": " .. sub_value
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
end




function do_and_advance_step()
  --print ("do_and_advance_step current_step is:  " .. current_step)

  
  local ratchet_mode = 1 -- default is 1 but it will be set

  engine.hz(400) -- just to give some audible sign for debugging timing

  if need_to_start_midi == true then
  
    if current_step == first_step then

      engine.hz(800) -- just to give some audible sign for debugging timing

      -- we only want to start midi clock at the right time!
      print ("Send MIDI Start current_step is: " .. current_step)
      midi_gates_out:start()
      midi_normal:start()
      need_to_start_midi = false

    else
      print ("Waiting to MIDI Start current_step is: " .. current_step)

    end

   end

  
  -- For each sequence row...
  for output = 1, TOTAL_SEQUENCE_ROWS do
    -- on the current step...
    ratchet_mode = grid_state[current_step][output]

    --process_step(output, ratchet_mode)

    -- process step should then run indep
    clock.run(process_step, output, ratchet_mode)

  end -- end for

  -- advance step
  current_step = util.wrap(current_step + 1, first_step, last_step)

  
end -- end function


function process_step (output, ratchet_mode)

  -- local count = mode
  -- local timing = 1/32

  -- if mode == 2 then  
  --   timing = 1/32
  -- elseif mode == 3 then
  --   timing = 3/32
  -- elseif mode == 4 then
  --   timing = 3/64
  -- end 



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





 
function gate_on(output)
       --print ("A ON LOWEST_MIDI_NOTE_NUMBER_FOR_GATE" .. LOWEST_MIDI_NOTE_NUMBER_FOR_GATE .. " MIDI_NOTE_ON_VELOCITY " .. MIDI_NOTE_ON_VELOCITY .. " sequence_row + MIDI_CHANNEL_GATES " .. sequence_row + MIDI_CHANNEL_GATES)
  midi_gates_out:note_on (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + output, MIDI_NOTE_ON_VELOCITY, MIDI_CHANNEL_GATES)
end 
  
function gate_off(output)
  --print ("A OFF LOWEST_MIDI_NOTE_NUMBER_FOR_GATE" .. LOWEST_MIDI_NOTE_NUMBER_FOR_GATE .. " MIDI_NOTE_OFF_VELOCITY " .. MIDI_NOTE_OFF_VELOCITY .. " sequence_row + MIDI_CHANNEL_GATES " .. sequence_row + MIDI_CHANNEL_GATES)
  midi_gates_out:note_off (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + output, MIDI_NOTE_OFF_VELOCITY, MIDI_CHANNEL_GATES)
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
     midi_gates_out:stop ()
     midi_normal:stop ()
end  

function enc(n,d)
  if n==3 then
     params:delta("clock_tempo", d)
  end
end 


-- Norns (Shield) key presses
function key(n,z)
  print("key pressed.  n:" .. n ..  " z:" .. z )
  
  -- since MIDI and Link offer their own start/stop messages,
  -- we'll only need to manually start if using internal or crow clock sources:
  if params:string("clock_source") == "internal" then

    -- left button pressed 
    if n == 2 and z == 1 then
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

  print ("grid_button_function_name will return: " .. ret)
  return ret

end -- end function definition

function init()
  

--  params:set("clock_source",4)
  
    -- Last In First Out (LIFO) tables for Undo and Redo of grid state functionality
  undo_lifo = {}
  redo_lifo = {}

 
  print ("before init_table")
  init_table()
    
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


   clock.run(tick)       -- start the sequencer


  --  clock.run(function()  -- redraw the screen and grid at 15fps (maybe refresh grid at a different rate?)
  --   while true do
  --     clock.sleep(1/15)
  --     redraw()
  --   end
  -- end)

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

function init_table()
  
  print ("Hello from init_table")
  
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

  
  print ("Bye from init_table")
  

end -- end init_table

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







-- Grid Key Presses
-- Handle key presses on the grid
my_grid.key = function(x,y,z)
-- x is the row
-- y is the column
-- z == 1 means key down, z == 0 means key up
-- We want to capture the key down event and toggle the state of the key in the grid.

--print("--------------------------------------------------------------")
--print(x .. ","..y .. " z is " .. z.. " value before change " .. grid_state[y][y])
--print(x .. ","..y .. " z is " .. z)







-- We treat sequence rows and control rows different.

-- SEQUENCE ROWS
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

    if preset_button ~= 0 then
      print ("preset_button is: " .. preset_button .. ", x is: " .. x .. ", y is: " .. y) 


      -- Any button pressed on row 1
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


      end  

    else -- preset button is not pressed 

      --print ("Before changing grid_state for sequence rows based on key press and ratchet button.")
      --print ("ratchet_button is: " .. ratchet_button)

      -- *Order is important here*. 
      -- Change the state of the grid *AFTER* we have pushed to undo lifo
      
      -- Place RATCHET on step
      if ratchet_button ~= 0 then -- We place the ratchet on the step
        grid_state[x][y] = ratchet_button
      else  -- No ratchet, do normal toggle 
        -- This TOGGLES the grid states i.e. because z=1 push on/off push off/on etc.
        if grid_state[x][y] ~= 0 then -- "on" might be 1 or something else if its a ratchet etc.
          grid_state[x][y] = 0
        else 
          grid_state[x][y] = 1
        end
      end

        --print ("After changing grid_state for sequence rows based on key press and ratchet button.")

    

    end -- preset_button test

--print ("End of key down test")

  end -- End of key down test

  --print(x .. ","..y .. " value after change " .. grid_state[y][y])
  --print("--------------------------------------------------------------")




else

  -- CONTROL ROWS


-- direct assignement of control rows.
    -- if the button is pressed it should be lit etc.
    -- we need to modifiy the code so grid_state control rows doesn't get set by undo
    grid_state[x][y] = z

    -- RATCHETS
    -- Holding one of the ratchet buttons and a step will put the ratchet "on" the step
    
    print ("Some CONTROL ROW BUTTON PRESSED " .. z)
    

    if grid_button_function_name(x,y) == "Ratchet2" then
      --print ("RATCHET button pressed: " .. z)
      if z == 1 then 
        ratchet_button = 2
      else
        ratchet_button = 0 -- Reset ratchet_button with a key up
      end
    elseif grid_button_function_name(x,y) == "Ratchet3" then
      --print ("RATCHET button pressed: " .. z)
      if z == 1 then 
        ratchet_button = 3
      else
        ratchet_button = 0
      end
    elseif grid_button_function_name(x,y) == "Ratchet4" then
      --print ("RATCHET button pressed: " .. z)
      if z == 1 then 
        ratchet_button = 4
      else
        ratchet_button = 0
      end
    elseif grid_button_function_name(x,y) == "Ratchet5" then
      --print ("RATCHET button pressed: " .. z)
      if z == 1 then 
        ratchet_button = 5
      else
        ratchet_button = 0
      end
    
  

-- Preset buttons
-- These buttons are used to put a preset on one of the sequence rows

  elseif grid_button_function_name(x,y) == "Preset2" then
  --print ("Preset button pressed: " .. z)
  if z == 1 then
    print ("Preset button pressed: " .. z) 
    preset_button = 2
  else
    print ("Preset button RESET: " .. z) 
    preset_button = 0 -- Reset preset_button with a key up
  end
elseif grid_button_function_name(x,y) == "Preset3" then
  --print ("Preset button pressed: " .. z)
  if z == 1 then 
    preset_button = 3
  else
    preset_button = 0
  end
elseif grid_button_function_name(x,y) == "Preset4" then -- currently no button for this
  --print ("Preset button pressed: " .. z)
  if z == 1 then 
    preset_button = 4
  else
    preset_button = 0
  end
elseif grid_button_function_name(x,y) == "Preset5" then -- currently no button for this
  --print ("Preset button pressed: " .. z)
  if z == 1 then 
    preset_button = 5
  else
    preset_button = 0
  end




-- Require alter clock button be pressed so that we don't stop / start the clock acceidentally

elseif grid_button_function_name (x,y) == "MidiClockArm" then
  if z == 1 then 
    arm_clock_button = 1
  else
    arm_clock_button = 0
  end


elseif grid_button_function_name (x,y) == "MidiClockStop" then
  if z == 1 and arm_clock_button == 1 then 
    request_midi_stop()
  end


elseif grid_button_function_name (x,y) == "MidiClockStart" then
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



    -- Pop UNDO For debugging purposes / so we can remove entries
    elseif grid_button_function_name (x,y) == "PopUndo" then
      -- List the UNDO LIFO
      --print("here comes undo_lifo")
      for key, value in pairs(undo_lifo) do
        print(key, " -- ", value)
        -- the values in undo_lifo are tables.
        print (get_tally(value))
      end

      --print("Now, remove an entry from the redo_lifo")
      pop_undo()
    --end  -- End of Pop UNDO 


    -- Pop REDO For debugging purposes / so we can remove entries
  elseif grid_button_function_name (x,y) == "PopRedo" then
      -- List the REDO LIFO
      --print("here comes redo_lifo")
      for key, value in pairs(redo_lifo) do
        print(key, " -- ", value)
        -- the values in undo_lifo are tables.
        print (get_tally(value))
      end

      --print("Now, remove an entry from the redo_lifo")
      pop_redo()

  elseif grid_button_function_name (x,y) == "ArmFirstStep" then
    if z == 1 then 
      arm_first_step_button = true
    else
      arm_first_step_button = false
    end

  elseif grid_button_function_name (x,y) == "ArmLastStep" then
    if z == 1 then 
      arm_last_step_button = true
    else
      arm_last_step_button = false
    end

 -- Set first_step
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button1" )  then
    set_first_step(1)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button2" )  then
    set_first_step(2)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button3" )  then
    set_first_step(3)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button4" )  then
    set_first_step(4)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button5" )  then
    set_first_step(5)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button6" )  then
    set_first_step(6)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button7" )  then
    set_first_step(7)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button8" )  then
    set_first_step(8)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button9" )  then
    set_first_step(9)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button10" )  then
    set_first_step(10)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button11" )  then
    set_first_step(11)             
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button12" )  then
    set_first_step(12)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button13" )  then
    set_first_step(13)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button14" )  then
    set_first_step(14)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button15" )  then
    set_first_step(15)
  elseif (arm_first_step_button == true and grid_button_function_name (x,y) == "Button16" )  then
    set_first_step(16)
    
  -- Set last_step
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button1" )  then
    set_last_step(1)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button2" )  then
    set_last_step(2)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button3" )  then
    set_last_step(3)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button4" )  then
    set_last_step(4)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button5" )  then
    set_last_step(5)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button6" )  then
    set_last_step(6)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button7" )  then
    set_last_step(7)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button8" )  then
    set_last_step(8)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button9" )  then
    set_last_step(9)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button10" )  then
    set_last_step(10)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button11" )  then
    set_last_step(11)             
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button12" )  then
    set_last_step(12)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button13" )  then
    set_last_step(13)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button14" )  then
    set_last_step(14)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button15" )  then
    set_last_step(15)
  elseif (arm_last_step_button == true and grid_button_function_name (x,y) == "Button16" )  then
    set_last_step(16) 


  end -- end of grid_button_function_name tests

end  -- End of Sequence / Control

  -- Always do this else results are not shown to user.
  refresh_grid_and_screen()


end -- End of function definition

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


    
      end
    end
  
  
    current_tempo = clock.get_tempo()
  

if need_to_start_midi == true then
  end_of_line_text = "Pending.."
else
  end_of_line_text = ""
end

if current_step <= 4 then
  start_of_line_text = "__"
elseif current_step <= 8 then
  start_of_line_text = "["
elseif current_step <= 12 then
  start_of_line_text = " ]"
elseif  current_step <= 16 then 
  start_of_line_text = "^^"
end    



  status_text = start_of_line_text .. " " .. current_tempo .. " BPM. Step " .. current_step .. " " .. end_of_line_text
  
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




function send_gates(sequence_row, mode)
  -- This function sends a gate or multiple gates for the row 
  -- The ratchet level / timing is determined by mode.
  -- Experimental so far.

  -- Mode == 1 means a normal gate
  

  local count = mode
  local timing = 1/32

  if mode == 2 then  
    timing = 1/32
  elseif mode == 3 then
    timing = 3/32
  elseif mode == 4 then
    timing = 3/64
  end  

  if mode > 1 then
    --print("this is supposed to be a ratchet")
  end 

  for i=1,count do
    --print("before do pulse")

    -- Since each betweener can only output 4 gates we need to split across the devices
    if sequence_row >= 1 and sequence_row <= 4 then

      --print ("A ON LOWEST_MIDI_NOTE_NUMBER_FOR_GATE" .. LOWEST_MIDI_NOTE_NUMBER_FOR_GATE .. " MIDI_NOTE_ON_VELOCITY " .. MIDI_NOTE_ON_VELOCITY .. " sequence_row + MIDI_CHANNEL_GATES " .. sequence_row + MIDI_CHANNEL_GATES)
      midi_gates_out:note_on (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE, MIDI_NOTE_ON_VELOCITY, sequence_row + MIDI_CHANNEL_GATES)
     
      --print ("A OFF LOWEST_MIDI_NOTE_NUMBER_FOR_GATE" .. LOWEST_MIDI_NOTE_NUMBER_FOR_GATE .. " MIDI_NOTE_OFF_VELOCITY " .. MIDI_NOTE_OFF_VELOCITY .. " sequence_row + MIDI_CHANNEL_GATES " .. sequence_row + MIDI_CHANNEL_GATES)
      midi_gates_out:note_off (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE, MIDI_NOTE_OFF_VELOCITY, sequence_row + MIDI_CHANNEL_GATES)
    elseif sequence_row >= 5 and sequence_row <= 8 then 
      midi_gates_out_b:note_on (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE, MIDI_NOTE_ON_VELOCITY, sequence_row + MIDI_CHANNEL_B_OFFSET)
      midi_gates_out_b:note_off (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE, MIDI_NOTE_OFF_VELOCITY, sequence_row + MIDI_CHANNEL_B_OFFSET)
    end

    --send_midi_note_on(50, 127, MIDI_CHANNEL_OFFSET + sequence_row)
    --send_midi_note_off(45, 0, MIDI_CHANNEL_OFFSET + sequence_row)
    --print("after do pulse")

    if mode ~= 1 then
      clock.sync( timing )
    end
    --print("after sync") 
  end

end  

---



function gate_low(sequence_row)
  --print("gate_low " .. output)
  --send_midi_note_off(45, 0, MIDI_CHANNEL_OFFSET + sequence_row)

  if sequence_row >= 1 and sequence_row <= 4 then
    midi_gates_out:note_off (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE, MIDI_NOTE_OFF_VELOCITY, sequence_row + MIDI_CHANNEL_GATES)
  elseif sequence_row >= 5 and sequence_row <= TOTAL_SEQUENCE_ROWS then 
    midi_gates_out_b:note_off (LOWEST_MIDI_NOTE_NUMBER_FOR_GATE, MIDI_NOTE_OFF_VELOCITY, sequence_row + MIDI_CHANNEL_B_OFFSET)
  end

end



