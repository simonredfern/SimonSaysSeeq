-- SimonSaysSeeq on Norns
-- Work in Progress!

local volts = 0
local slew = 0

-- on/off for stepped sequence 
sequence = true

COLS = 16
ROWS = 8

GRID_TABLE_FILE = "/home/we/simon-says-seeq-grid.tbl"

Tab = require "lib/tabutil"

current_step = 1

-- highlight_grid = {}

greetings_done = false



my_grid = grid.connect()

print (my_grid)



local BAR_VALS = {
  { str = "1/256", value = 1 / 256, ppq = 64 },  -- [1]
  { str = "1/128", value = 1 / 128, ppq = 32 },  -- [2]
  { str = "1/96", value = 1 / 96, ppq = 24 },    -- [3]
  { str = "1/64", value = 1 / 64, ppq = 16 },    -- [4]
  { str = "1/48", value = 1 / 48, ppq = 12 },    -- [5]
  { str = "1/32", value = 1 / 32, ppq = 8 },     -- [6] *
  { str = "1/16", value = 1 / 16, ppq = 4 },     -- [7]
  { str = "1/8", value = 1 / 8, ppq = 2 },       -- [8]
  { str = "1/4", value = 1 / 4, ppq = 1 },       -- [9]
  { str = "1/2", value = 1 / 2, ppq = 0.5 },     -- [10]
  { str = "1", value = 1, ppq = 0.25 },          -- [11]
  { str = "2", value = 2, ppq = 0.125 },         -- [12]
  { str = "3", value = 3, ppq = 0.083 },         -- [13]
  { str = "4", value = 4, ppq = 0.0625 },        -- [14]
}

local beat_div = BAR_VALS[1].value --grid.index].value 


--update_id = clock.run(update)

-- system clock tick
-- this function is started by init() and runs forever
-- if the sequence is on, it steps forward on each clock tick
-- tempo is controlled via the global clock, which can be set in the PARAMETERS menu 
tick = function()
  while true do
    clock.sync(1)
    if sequence then advance_step() end
  end
end

function greetings()
  
  screen.move(10,10)
  screen.text("Hello, I am SimonSaysSeeq")
  screen.move(20,20)
  screen.text("on Norns v0.1")
  
  screen.move(10,30)
  
  screen.text("My Grid is:")
  screen.move(20,40)
  
  if (not my_grid) then
    screen.text("not connected")
  else
    screen.text(tostring(my_grid))
  end 
 
  screen.move(20,50) 
  screen.text(my_grid.name)
  screen.move(20,60)   
  screen.text(my_grid.cols .. " X " .. my_grid.rows)


  screen.update()
  
  
  clock.sleep(3)
  --print("now awake")
  greetings_done = true
end




function advance_step()
  --print ("advance_step")
  
  current_step = current_step + 1
  
  if (current_step > COLS) then
    current_step = 1
  end
  
end
  




function init()
  
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
  

end


  clock.run(tick)       -- start the sequencer


  clock.run(function()  -- redraw the screen and grid at 15fps
    while true do
      clock.sleep(1/15)
      redraw()
      --gridredraw()
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
end

--print(x .. ","..y .. " value after change " .. grid_table[x][y])
--print("--------------------------------------------------------------")

end


function refresh_grid()
  
  -- print ("Hello from refresh_grid")
  
  screen.clear()
  screen.move(1,1)
  
    for col = 1,COLS do -- rows 1 to 4
      for row = 1,ROWS do -- columns 1 to 4
        screen.move(col * 7,row * 7)
        --screen.text("table[" .. row .. "]["..col.."] is: " ..grid_table[row][column])
        

        if (current_step == col and row <= 6) then
           -- This is the scrolling cursor
           screen.text("*")
           my_grid:led(col,row,6)
        else
          if (grid_table[col][row] == 1) then
             -- Grid square is On
            my_grid:led(col,row,12)
          else 
             -- Grid square is Off
            my_grid:led(col,row,2)
          end
          -- Show the stored value on screen
          screen.text(grid_table[col][row])
        end

        --screen.text(highlight_grid[row][colum])
        
        screen.update()
      end
    end
  
  
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


--function redraw_grid()
--  my_grid:all(0) -- turn all the LEDs off...
--  for i=1,16 do
--    my_grid:led(i,2,4) -- set step positions to brightness 4
--  end
--  my_grid:refresh() -- refresh the grid
--end



--function grid.add(new_grid) -- must be grid.add, not g.add (this is a function of the grid class)
--  print(new_grid.name.." says 'hello!'")
--   -- each grid added can be queried for device information:
--  print("new grid found at port: "..new_grid.port)
--  g = grid.connect(new_grid.port) -- connect script to the new grid
--  --grid_connected = true -- a grid has been connected!
--  --grid_dirty = true -- enable flag to redraw grid, because data has changed
--end

--function grid.remove(g) -- must be grid.remove, not g.remove (this is a function of the grid class)
--  print(g.name.." says 'goodbye!'")
--end


function gate_high(gate_number)
  print("gate_high: " .. gate_number)
end



