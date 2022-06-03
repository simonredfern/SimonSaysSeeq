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



rough_undo_lifo_count = 0
rough_redo_lifo_count = 0

-- Crow Outputs
CROW_OUTPUTS = 4

RATCHET_LANE_A = 5

GRID_STATE_FILE = "/home/we/SimonSaysSeeq-grid.tbl"

Tab = require "lib/tabutil"

MIDI_CHANNEL_OFFSET = 0

current_step = 1


greetings_done = false



my_grid = grid.connect()

grid_state_dirty = false

print (my_grid)

out_midi = midi.connect(1)

-- psudo random for our grid ids
math.randomseed( os.time() )
math.random() -- call a few times so it gets more random (apparently)
math.random() 
math.random()




-- system clock tick
-- this function is started by init() and runs forever
-- if transport_active is on, it steps forward on each clock tick
-- tempo is controlled via the global clock, which can be set in the PARAMETERS menu 
-- instead of tick = function()
function tick()
  while true do
    clock.sync(1/4)
    if transport_active then advance_step() end
  end
end


function midi_watcher()
  while true do
    clock.sync(1/4)
    if need_to_start_midi == true then
      -- we only want to start midi clock at the right time!
      out_midi:start()
      need_to_start_midi = false
    end
  end
end

function grid_state_popularity_watcher()
  -- get some idea of how much a particular grid is used
  -- the idea is to use this on a fast forward backward undo / redo that just scrolls through the popular states
  -- (i.e. states that have been used for a long time)
  while true do
    clock.sync(1) -- do this every beat
    grid_state["gspc"] = grid_state["gspc"] + 1
    --print ("grid_state.gspc is: " .. grid_state["gspc"])
  end
end



function greetings()
  
  screen.move(10,10)
  screen.text("Hello, I am SimonSaysSeeq")
  screen.move(20,20)
  screen.text("on Norns v" .. version)
  
  screen.move(10,30)
  
  
  grid_text = "Unknown"
  
  --screen.move(20,40)
  
  if (not my_grid) then
    grid_text = "Grid NOT CONNECTED"
  else
    grid_text = "Grid: " .. tostring(my_grid.name)
  end 
  
  screen.text(grid_text)
  
  screen.move(20,60)   
  screen.text(my_grid.cols .. " X " .. my_grid.rows)


  screen.update()
  
  
  clock.sleep(8)
  --print("now awake")
  greetings_done = true
end




function advance_step()
  --print ("advance_step")
  

  --current_step = current_step + 1
  --if (current_step > COLS) then
  --  current_step = 1
  --end
  
  local gate_type = "NORMAL" -- default

  current_step = util.wrap(current_step + 1, 1, COLS)
  
  
  for output = 1, CROW_OUTPUTS do
    if grid_state[current_step][output] ~= 0 then -- could be 1 or 2 (ratchet) or...

      -- Get the type of output (ratchet)
      -- For now, only for output 2

      --if output == 2 then -- open high hat
       -- check row 5 of the current step, if its 1 let's ratched open hi hat.
       --if grid_state[current_step][RATCHET_LANE_A] == 1 then
      --  gate_type = "RATCHET"
      -- end
      --end

      -- direct relation between value on grid at count of pulses we will get
      pulses(output, grid_state[current_step][output])


      
      --gate_high(output, grid_state[current_step][output] )



      --gate_low(output)

    elseif grid_state[current_step][output] == 0 then

      --gate_low(output)
    end -- end non zero
  end -- end for
  
end
  
  

function clock.transport.start()

  print("transport.start")

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
  current_step = 1


  transport_active = false
  screen.move(80,80)
  screen.text("STOP")
  screen.update()
end



function request_midi_stop()
  print("request_midi_stop")
  need_to_start_midi = false
  -- can stop the midi clock at any time.
     out_midi:stop ()
end  

function send_midi_note_on(note, vel, ch)
  print("send_midi_note_on")
  out_midi:note_on (note, vel, ch)
end  

function send_midi_note_off(note, vel, ch)
  print("send_midi_note_off")
  out_midi:note_off (note, vel, ch)
end  


function enc(n,d)
  if n==3 then
     params:delta("clock_tempo", d)
  end
end 

function key(n,z)
  print("key pressed.  n:" .. n ..  " z:" .. z )
  
  -- since MIDI and Link offer their own start/stop messages,
  -- we'll only need to manually start if using internal or crow clock sources:
  if params:string("clock_source") == "internal" then

    if n == 2 and z == 1 then
      if transport_active then
        
        clock.transport.stop()
        request_midi_stop()
      end
      
      screen_dirty = true
    end
    
    if n == 3 and z == 1 then
      if not transport_active then
        clock.transport.start()
        request_midi_start()
      end
      
      screen_dirty = true
    end
    
  end
end




function init()
  

--  params:set("clock_source",4)
  
  

    -- last in first out
  undo_lifo = {}
  redo_lifo = {}

 
  
  
  print ("before init_table")
  init_table()
    
  refresh_grid()

  print("hello")
  my_grid:all(2)
  my_grid:refresh() -- refresh the LEDs
    
    
  print("my_grid follows: ")
  print(my_grid)
  print("my_grid.name is: " .. my_grid.name)
  print("my_grid.cols is: " .. my_grid.cols)
  print("my_grid.rows is: " .. my_grid.rows)
  

  -- Set the starting tempo. Can be changed with right kno
  -- TODO store and retreive this
  params:set("clock_tempo",136)
  
  --midi_watcher_id = clock.run(midi_watcher)
  
end


 clock.run(tick)       -- start the sequencer
 
 clock.run(midi_watcher) 

 clock.run(grid_state_popularity_watcher) 

 

  clock.run(function()  -- redraw the screen and grid at 15fps (maybe refresh grid at a different rate?)
    while true do
      clock.sleep(1/15)
      redraw()
      --gridredraw()
    end
  end)


  -- Periodically check if we need to save the grid state to file.
  clock.run(function()
    while true do
      clock.sleep(5)
        if (grid_state_dirty == true) then
          Tab.save(grid_state, GRID_STATE_FILE)
          grid_state_dirty = false
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

 -- math.randomseed( os.time() )
  

  fresh_grid["id"]=math.random(1,99999999999999)
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
  

  -- Push Undo so we can get back to initial state
  push_undo()

  
  print ("Bye from init_table")
  

end

function push_undo()

  print("push_undo says hello. Store Undo LIFO")
  -- TODO check memory / count of states? - if this gets very large, truncate from the other side

  -- When we push to the undo_lifo, we want to *copy* the grid_state (not reference) so that any subsequent changes to grid_state are not saved on the undo_lifo 
  -- Inserts in the last position of the table (push)
  table.insert (undo_lifo, get_copy_of_grid(grid_state))

  print ("undo_lifo size is: ".. lifo_size(undo_lifo))

end  

function pop_undo()

  if lifo_size(undo_lifo) > 1 then

    -- 2) Pop from the undo_lifo to the current state.
    -- Removes from the last element of the table (pop)
    local undo_state = table.remove (undo_lifo)

    grid_state = get_copy_of_grid(undo_state)

    print ("undo_lifo size is: ".. lifo_size(undo_lifo))
    
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
    
    print ("redo_lifo size is: ".. lifo_size(redo_lifo))
end  


function pop_redo()

  if lifo_size(redo_lifo) > 1 then

      -- 2) Pop the redo_lifo into the current state.

      -- TODO need to copy this?
      local redo_state = table.remove (redo_lifo) 
      grid_state = get_copy_of_grid(redo_state)

      print ("redo_lifo size is: ".. lifo_size(redo_lifo))
    else 
      print ("Not poping last redo_lifo because its size is 1 or less ")  
    end

end  








-- Handle key presses on the grid
my_grid.key = function(x,y,z)
-- x is the row
-- y is the column
-- z == 1 means key down, z == 0 means key up
-- We want to capture the key down event and toggle the state of the key in the grid.

--print("--------------------------------------------------------------")
--print(x .. ","..y .. " z is " .. z.. " value before change " .. grid_state[x][y])
  
-- TODO will want to treat sequence rows and control rows differently in terms of button presses.
-- i.e. sequence rows toggle
-- control rows are momentary press 



print(x .. ","..y .. " z is " .. z)

if x == 16 and y == 7 then
  print ("RATCHET button pressed: " .. z)
  ratchet_button = 2
elseif x == 15 and y == 7 then
  print ("RATCHET button pressed: " .. z)
  ratchet_button = 3
elseif x == 14 and y == 7 then
  print ("RATCHET button pressed: " .. z)
  ratchet_button = 4
elseif x == 13 and y == 7 then
  print ("RATCHET button pressed: " .. z)
  ratchet_button = 5
elseif x == 12 and y == 7 then
  print ("RATCHET button pressed: " .. z)
  ratchet_button = 6
else
  print ("RATCHET button NOT pressed: " .. z)
  ratchet_button = 0
end 


-- We treat sequence rows and control rows different.

-- SEQUENCE ROWS
if y < 5 then
  -- For sequence rows we only want to capture key down event only  
  if z == 1 then


    -- *Order is important here*. 
    -- We want to save the current state *before* we push a copy of the grid to the undo lifo
    -- But only do this if we are not touching the control rows (7 & 8)

    print ("Sequence button pressed")
    print ("grid_state:" .. get_tally(grid_state))

    -- Every time we change state of rows less than 6 (non control rows), record the new state in the undo_lifo
    push_undo()

    -- So we save the table to file
    -- (don't bother with control rows)
    grid_state_dirty = true
  



    -- *Order is important here*. 
    -- Change the state of the grid *AFTER  * we have pushed to undo lifo
    -- This TOGGLES the grid states i.e. because z=1 push on/off push off/on etc.
    if grid_state[x][y] ~= 0 then -- "on" might be 1 or something else if its a ratchet etc.
      grid_state[x][y] = 0
    else 
      if ratchet_button ~= 0 then
        grid_state[x][y] = ratchet_button -- set the state to some kind of ratchet
      else  
        grid_state[x][y] = 1
      end
    end
    

  end -- End of key down test

  --print(x .. ","..y .. " value after change " .. grid_state[x][y])
  --print("--------------------------------------------------------------")




else

  -- CONTROL ROWS


-- direct assignement of control rows.
    -- if the button is pressed it should be lit etc.
    -- we need to modifiy the code so grid_state control rows doesn't get set by undo
    grid_state[x][y] = z




    -- CONTROL UNDO

    -- Buttons in a Control row 
    if (x == 1 and y == 8) then
      ----------
      -- UNDO --
      ----------
      print ("Pressed 1,8: UNDO")


      -- Only do this if we know we can pop from undo 
      if (lifo_populated(undo_lifo)) then

        --print ("undo_lifo is populated")
      
        -- In order to Undo we: 


        -- local tally = refresh_grid()
        -- print ("grid_state BEFORE push_redo is:")
        -- print (grid_state)
        -- print ("tally is:" .. tally)

        push_redo()

        -- local tally = refresh_grid()
        -- print ("grid_state BEFORE pop_undo is:")
        -- print (grid_state)
        -- print ("tally is:" .. tally)



        pop_undo()

        -- local tally = refresh_grid()
        -- print ("grid_state AFTER pop_undo is:")
        -- print (grid_state)
        print ("grid_state: " .. get_tally(grid_state))


    
        -- print ("grid_state is:")
        -- print (grid_state)

      else
        print ("undo_lifo is NOT populated")
      end


    end -- End of UNDO

    -- CONTROL REDO    
    if (x == 2 and y == 8) then
      -- REDO  
      -- print ("Pressed 2,8: REDO")
      -- local tally = refresh_grid()
      -- print ("grid_state BEFORE push_undo is:")
      -- print (grid_state)
      -- print ("tally is:" .. tally)

          -- Only do this if we know we can pop from undo 
      if (lifo_populated(redo_lifo)) then
        -- print ("redo_lifo is populated")

        push_undo()

        -- local tally = refresh_grid()
        -- print ("grid_state BEFORE pop_redo is:")
        -- print (grid_state)
        -- print ("tally is:" .. tally)
        pop_redo()

      --  refresh_grid()

        -- local tally = refresh_grid()
        -- print ("grid_state AFTER pop_redo is:")
        -- print (grid_state)
        -- print ("tally is:" .. tally)

        print ("grid_state: " .. get_tally(grid_state))
      else
        print ("redo_lifo is NOT populated")
      end  
    end -- End of REDO



    -- Pop UNDO For debugging purposes / so we can remove entries
    if (x == 15 and y == 8) then
      -- List the UNDO LIFO
      print("here comes undo_lifo")
      for key, value in pairs(undo_lifo) do
        print(key, " -- ", value)
        -- the values in undo_lifo are tables.
        print (get_tally(value))
      end

      print("Now, remove an entry from the redo_lifo")
      pop_undo()
    end  -- End of Pop UNDO 


    -- Pop REDO For debugging purposes / so we can remove entries
    if (x == 16 and y == 8) then
      -- List the REDO LIFO
      print("here comes redo_lifo")
      for key, value in pairs(redo_lifo) do
        print(key, " -- ", value)
        -- the values in undo_lifo are tables.
        print (get_tally(value))
      end

      print("Now, remove an entry from the redo_lifo")
      pop_redo()
    end -- End of Pop REDO

  end  -- End of Sequence / Control







  -- Always do this else results are not shown to user.
  refresh_grid()


end -- End of function definition


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



function refresh_grid()
  
  --print ("Hello from refresh_grid for grid at:")
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

        --screen.text(highlight_grid[row][colum])
        
        screen.update()
      end
    end
  
  
    current_tempo = clock.get_tempo()
  
  
  --  ..  params:get("step_div")
  
  status_text = current_tempo .. " BPM. Step " .. current_step
  
  screen.move(1,63)   
  screen.text(status_text)

  my_grid:refresh()
  
  -- print ("Bye from refresh_grid tally is:" .. tally)
  
  return tally

end  


-- redraw gets called by the norns implicitly sometimes and explicitly by us too.
function redraw()


  if (greetings_done == false) then
    --print("i will do greetings becuase not done yet")
    clock.run(greetings)
  else 
    --print ("i will print table because greetings done")
    refresh_grid()
  end
  
  screen.update()
end




function pulses(output_port, count)
  if count > 1 then
    print("this is supposed to be a ratchet")
  end 

  for i=1,count do
    print("before do pulse")
    send_midi_note_on(50, 127, MIDI_CHANNEL_OFFSET + output_port)
    send_midi_note_off(45, 0, MIDI_CHANNEL_OFFSET + output_port)
    print("after do pulse")
    clock.sync( 1/32 )
    print("after sync") 
  end

end  


function gate_high(output_port, gate_type)
  --print("gate_high: " .. output_port)

-- See if we want to ratchet based on a certain row / config.

if gate_type == 1 then -- normal

  send_midi_note_on(50, 127, MIDI_CHANNEL_OFFSET + output_port)
  gate_low(output_port)

elseif gate_type == 2 then -- ratchet
    print("gate_high: " .. output_port)
    print("RATCHET")


    clock.run(ratchet)


    --send_midi_note_on(45, 127, MIDI_CHANNEL_OFFSET + output_port)
    --gate_low(output_port)
    

  end


  
end

function gate_low(output_port)
  --print("gate_low " .. output)
  send_midi_note_off(45, 0, MIDI_CHANNEL_OFFSET + output_port)
end

function ratchet()
  for i=1,4 do
    print("before ratchet clock sync")
    clock.sleep(1)
    print("after ratchet clock sync")

    
    
  end
end


