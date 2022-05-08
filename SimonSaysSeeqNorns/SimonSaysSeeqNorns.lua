-- SimonSaysSeeq on Norns
-- Left Button Stop. Right Start

version = 0.1

local volts = 0
local slew = 0

-- on/off for stepped sequence 
transport_active = true


-- Dimensions of Grid
COLS = 16
ROWS = 8


-- Crow Outputs
CROW_OUTPUTS = 4

GRID_TABLE_FILE = "/home/we/SimonSaysSeeq-grid.tbl"

Tab = require "lib/tabutil"

current_step = 1

-- highlight_grid = {}

greetings_done = false



my_grid = grid.connect()

grid_table_dirty = false

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
      -- we only want to start midi clock on the beat
      out_midi:start()
      need_to_start_midi = false
    end
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
    if grid_table[current_step][output] == 1 then
      gate_high(output)
    elseif grid_table[current_step][output] == 0 then
      gate_low(output)
    end
  end
  
end
  
  

function clock.transport.start()

  print("transport.start")
  --tick_id = clock.run(tick)
  transport_active = true
  
  --current_step = 1
  
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

  --clock.cancel(tick_id)
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
  params:set("clock_tempo",136)
  
  --midi_watcher_id = clock.run(midi_watcher)
  
end


 clock.run(tick)       -- start the sequencer
 
 clock.run(midi_watcher) 

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
        if (grid_table_dirty == true) then
          Tab.save(grid_table, GRID_TABLE_FILE)
          grid_table_dirty = false
        end
    end
  end)


 


function init_table()
  
  print ("Hello from init_table")
  
  -- Try to load the table
  grid_table = Tab.load (GRID_TABLE_FILE)
  print("Result of table load is:")
  print (grid_table)
  
  -- if it doesn't exist
  if grid_table == nil then
    print ("No table, I will generate a structure and save that")
    grid_table = {}
    for col = 1, COLS do 
      grid_table[col] = {} -- create a table for each col
      for row = 1, ROWS do
        if col == row then -- eg. if coordinate is (3,3)
          grid_table[col][row] = 1
        else -- eg. if coordinate is (3,2)
          grid_table[col][row] = 0
        end
      end
    end
    
    Tab.save(grid_table, GRID_TABLE_FILE)
    grid_table = Tab.load (GRID_TABLE_FILE)
  else
    print ("I already have a table")
  end
  
  print ("Bye from init_table")
  

end




-- Handle key presses on the grid
my_grid.key = function(x,y,z)
-- x is the row
-- y is the column
-- z == 1 means key down, z == 0 means key up
-- We want to capture the key down event and toggle the state of the key in the grid.

--print("--------------------------------------------------------------")
--print(x .. ","..y .. " z is " .. z.. " value before change " .. grid_table[x][y])
  
-- Capture key down event only  
if z == 1 then
  if grid_table[x][y] == 1 then
    grid_table[x][y] = 0
  else 
    grid_table[x][y] = 1
  end
  -- So we save the table to file
  grid_table_dirty = true
end

--print(x .. ","..y .. " value after change " .. grid_table[x][y])
--print("--------------------------------------------------------------")

end


function refresh_grid()
  
  -- print ("Hello from refresh_grid")
  
  screen.clear()
  screen.move(1,1)
  
    for col = 1,COLS do 
      for row = 1,ROWS do
        screen.move(col * 7,row * 7)
        --screen.text("table[" .. row .. "]["..col.."] is: " ..grid_table[row][column])
        

        -- Show the scrolling of the steps with the first 6 rows of LEDS. (Others will be used for other controls)
        if (current_step == col and row <= 6) then
           -- This is the scrolling cursor
           screen.text("*")
          
          
          if (grid_table[col][row] == 1) then 
            -- If current step and key is on, highlight it.
            my_grid:led(col,row,15) 
          else
            -- Else use scrolling brightness
            my_grid:led(col,row,6)
          end
        else
          if (grid_table[col][row] == 1) then
             -- Not current step but Grid square is On
            my_grid:led(col,row,12)
          else 
             -- Not current step and key is off
            my_grid:led(col,row,0)
          end
          -- Show the stored value on screen
          screen.text(grid_table[col][row])
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
  
  -- print ("Bye from refresh_grid")
  
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


