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

GRID_STATE_FILE = "/home/we/SimonSaysSeeq-grid.tbl"

Tab = require "lib/tabutil"

current_step = 1


greetings_done = false



my_grid = grid.connect()

grid_state_dirty = false

print (my_grid)

out_midi = midi.connect(1)






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
  while true do
    clock.sync(1) -- do this every beat
    grid_state["grid_state_popularity_counter"] = grid_state["grid_state_popularity_counter"] + 1
    --print ("grid_state.grid_state_popularity_counter is: " .. grid_state["grid_state_popularity_counter"])
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
  
  current_step = util.wrap(current_step + 1, 1, COLS)
  
  
  for output = 1, CROW_OUTPUTS do
    if grid_state[current_step][output] == 1 then
      gate_high(output)
    elseif grid_state[current_step][output] == 0 then
      gate_low(output)
    end
  end
  
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








function enc(n,d)
  if n==3 then
     params:delta("clock_tempo", d)
  end
end 

function key(n,z)
  print("key pressed.  n:" .. n ..  " z:" .. z )
  
  -- since MIDI and Link offer their own start/stop messages,
  -- we'll only need to manually start if using internal or crow clock sources:
  if params:string("clock_source") == "internal" or params:string("clock_source") == "crow" then

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
  
--  crow.clear()
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
  
  -- What does this do?
  crow.reset()
  
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
  
  grid_state["grid_state_popularity_counter"]=0 -- Reset this (apart from anything this assures the key is there)

  print("Result of table load is:")
  print (grid_state)
  return grid_state
end
  
 
function create_grid()
  local fresh_grid = {}
  fresh_grid["grid_state_popularity_counter"]=0 -- we can increment this to see how popular this grid is
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
  -- TODO check memory / count of states?

  -- When we push to the undo_lifo, we want to *copy* the grid_state (not reference) so that any subsequent changes to grid_state are not saved on the undo_lifo 
  -- Inserts in the last position of the table (push)
  table.insert (undo_lifo, get_copy_of_grid(grid_state))

  print ("undo_lifo size is: ".. lifo_size(undo_lifo))

end  

function pop_undo()

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

end 




function push_redo()
    -- 1) Push the current state to the redo_lifo so we can get back to it.
    -- Similarly we want to *copy* the grid_state (not reference) 
    -- so any subsequent changes to the grid_state are not reflected in the redo_lifo
    table.insert (redo_lifo, get_copy_of_grid(grid_state))
    
    print ("redo_lifo size is: ".. lifo_size(redo_lifo))
end  


function pop_redo()
      -- 2) Pop the redo_lifo into the current state.

      -- TODO need to copy this?
      local redo_state = table.remove (redo_lifo) 
      grid_state = get_copy_of_grid(redo_state)

      print ("redo_lifo size is: ".. lifo_size(redo_lifo))

end  








-- Handle key presses on the grid
my_grid.key = function(x,y,z)
-- x is the row
-- y is the column
-- z == 1 means key down, z == 0 means key up
-- We want to capture the key down event and toggle the state of the key in the grid.

--print("--------------------------------------------------------------")
--print(x .. ","..y .. " z is " .. z.. " value before change " .. grid_state[x][y])
  
  -- Capture key down event only  
  if z == 1 then
    if grid_state[x][y] == 1 then
      grid_state[x][y] = 0
    else 
      grid_state[x][y] = 1
    end


    -- As long as not touching "control rows" save the state so we can undo.
    if y < 6 then
      print ("grid_state less than 6 row press")
      print (grid_state)
      local tally = get_tally(grid_state)
      print ("tally is:" .. tally)

      -- Every time we change state of rows less than 6 (non control rows), record the new state in the undo_lifo
      push_undo()

      -- So we save the table to file
      -- (don't bother with control rows)
      grid_state_dirty = true


    end   

    
    if (x == 1 and y == 8) then
      ----------
      -- UNDO --
      ----------
      print ("Pressed 1,8: UNDO")


      -- Only do this if we know we can pop from undo 
      if (lifo_populated(undo_lifo)) then

        print ("undo_lifo is populated")
      
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
      -- print ("tally is:" .. tally)


     -- refresh_grid()
      -- print ("grid_state is:")
      -- print (grid_state)

      else
        print ("undo_lifo is NOT populated")

      end


    end 
    
    if (x == 2 and y == 8) then
      -- REDO  
      -- print ("Pressed 2,8: REDO")
      -- local tally = refresh_grid()
      -- print ("grid_state BEFORE push_undo is:")
      -- print (grid_state)
      -- print ("tally is:" .. tally)

          -- Only do this if we know we can pop from undo 
      if (lifo_populated(redo_lifo)) then
        print ("redo_lifo is populated")

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
      else
        print ("redo_lifo is NOT populated")


    end

  end
    
  -- For debugging purposes / so we can remove entries
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
    end  


    -- For debugging purposes / so we can remove entries
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
    end 



    -- Always do this else results are not obvious to user.
    refresh_grid()
  end -- End of key down test

  --print(x .. ","..y .. " value after change " .. grid_state[x][y])
  --print("--------------------------------------------------------------")


end -- End of function definition


function get_tally(input_grid)
  -- A helper debug function to show the state of a grid
  -- A grid is a table with known dimensions
  -- Used for debugging
  local tally = "grid_state_popularity_counter:" .. input_grid["grid_state_popularity_counter"] .. "-"
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
  -- copy all the key values 
  output_grid["grid_state_popularity_counter"] = input_grid["grid_state_popularity_counter"]
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
          
          
          if (grid_state[col][row] == 1) then 
            -- If current step and key is on, highlight it.
            my_grid:led(col,row,15) 
          else
            -- Else use scrolling brightness
            my_grid:led(col,row,6)
          end
        else
          if (grid_state[col][row] == 1) then
             -- Not current step but Grid square is On
            my_grid:led(col,row,12)
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



function gate_high(output_port)
  --print("gate_high: " .. output_port)
  --crow.output[output_port].action = "pulse(time,level,polarity)"
  crow.output[output_port].action = "pulse(0.003,10,1)"
  crow.output[output_port].execute()
end

function gate_low(output)
  --print("gate_low " .. output)
  -- causes error message
  --crow.output[output_port].volts = 0
  --crow.output[output_port].execute()
  
end











--  FIFO (queue)

--  You can simply implement FIFO by using table.insert to add new items, and table.remove to remove the first one, like this:
 
 
--  t = {}
 
--  table.insert (t, "a")  --> insert an item
--  table.insert (t, "b")  --> insert another
 
--  x = table.remove (t, 1)  --> remove first item
--  print (x)  --> a
 
--  x = table.remove (t, 1)  --> remove first item
--  print (x)  --> b
 
 
--  LIFO (stack)
 
--  A small change implements last in, first out:
 
 
--  t = {}
 
--  table.insert (t, "a")  --> insert an item
--  table.insert (t, "b")  --> insert another
 
--  x = table.remove (t)  --> remove last item
--  print (x)  --> b
 
--  x = table.remove (t)  --> remove last item
--  print (x)  --> a
 
