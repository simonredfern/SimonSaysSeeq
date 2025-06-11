-- SimonSaysSeeq Boot-Time Selector
-- Shows an interactive menu on Norns screen during startup
-- Allows user to choose between Lua menu or direct Rust app launch

local config_file = "/home/we/.config/simonsaysseeq/startup_mode"
local timeout_seconds = 10
local current_selection = 1
local menu_items = {
    {name = "Normal Norns Menu", desc = "Boot to standard menu, select Lua script", mode = "menu"},
    {name = "Direct Rust App", desc = "Launch SimonSaysSeeqRust immediately", mode = "rust"},
    {name = "Use Saved Setting", desc = "Continue with previously saved mode", mode = "saved"}
}

-- State variables
local start_time = 0
local countdown = timeout_seconds
local screen_dirty = true
local key_pressed = false

-- File utilities
local function file_exists(path)
    local file = io.open(path, "r")
    if file then
        file:close()
        return true
    end
    return false
end

local function read_config()
    if not file_exists(config_file) then
        return "menu"
    end
    
    local file = io.open(config_file, "r")
    if not file then
        return "menu"
    end
    
    local mode = file:read("*line")
    file:close()
    
    if mode then
        mode = mode:gsub("^%s*(.-)%s*$", "%1")
        if mode == "rust" or mode == "menu" then
            return mode
        end
    end
    
    return "menu"
end

local function write_config(mode)
    os.execute("mkdir -p " .. config_file:match("(.*/)[^/]*$"))
    
    local file = io.open(config_file, "w")
    if file then
        file:write(mode .. "\n")
        file:close()
        return true
    end
    return false
end

local function launch_rust_app()
    print("Launching SimonSaysSeeqRust...")
    local rust_binary = "/home/we/dust/code/SimonSaysSeeqRust/simon_says_seeq"
    
    if not file_exists(rust_binary) then
        print("ERROR: Rust binary not found!")
        return false
    end
    
    -- Launch Rust app via systemd service for proper management
    local result = os.execute("sudo systemctl start simonsaysseeq-rust")
    return result == 0
end

local function get_saved_mode_display()
    local saved = read_config()
    if saved == "rust" then
        return "Direct Rust App"
    else
        return "Normal Menu"
    end
end

-- Screen drawing functions
local function draw_header()
    screen.level(15)
    screen.move(64, 10)
    screen.text_center("SimonSaysSeeq Boot")
    
    screen.level(10)
    screen.move(64, 20)
    screen.text_center("Choose startup mode:")
end

local function draw_countdown()
    screen.level(8)
    screen.move(64, 55)
    if countdown > 0 then
        screen.text_center("Auto-select in " .. countdown .. "s")
    else
        screen.text_center("Starting...")
    end
end

local function draw_menu_items()
    for i, item in ipairs(menu_items) do
        local y = 25 + (i * 8)
        
        -- Highlight current selection
        if i == current_selection then
            screen.level(15)
            screen.rect(2, y - 3, 124, 7)
            screen.fill()
            screen.level(0) -- Black text on white background
        else
            screen.level(12)
        end
        
        screen.move(5, y)
        local display_name = item.name
        if item.mode == "saved" then
            display_name = item.name .. " (" .. get_saved_mode_display() .. ")"
        end
        screen.text(display_name)
    end
end

local function draw_description()
    screen.level(8)
    screen.move(64, 48)
    screen.text_center(menu_items[current_selection].desc)
end

local function draw_instructions()
    screen.level(6)
    screen.move(2, 62)
    screen.text("E2/E3: Select")
    screen.move(90, 62)
    screen.text("K3: Choose")
end

function redraw()
    if not screen_dirty then return end
    
    screen.clear()
    
    draw_header()
    draw_menu_items()
    draw_description()
    draw_countdown()
    draw_instructions()
    
    screen.update()
    screen_dirty = false
end

-- Input handling
function enc(n, delta)
    if n == 2 or n == 3 then
        current_selection = current_selection + (delta > 0 and 1 or -1)
        if current_selection > #menu_items then
            current_selection = 1
        elseif current_selection < 1 then
            current_selection = #menu_items
        end
        screen_dirty = true
        key_pressed = true
    end
end

function key(n, z)
    if n == 3 and z == 1 then -- K3 pressed
        key_pressed = true
        select_mode()
    elseif n == 1 and z == 1 then -- K1 pressed (cancel/default)
        key_pressed = true
        current_selection = 1 -- Default to normal menu
        select_mode()
    end
end

function select_mode()
    local selected_item = menu_items[current_selection]
    local chosen_mode = selected_item.mode
    
    -- Handle "saved" mode selection
    if chosen_mode == "saved" then
        chosen_mode = read_config()
    end
    
    -- Update display
    screen.clear()
    screen.level(15)
    screen.move(64, 30)
    screen.text_center("Starting " .. selected_item.name .. "...")
    screen.update()
    
    -- Save the choice if it's not "saved"
    if selected_item.mode ~= "saved" then
        write_config(chosen_mode)
    end
    
    -- Execute the chosen mode
    if chosen_mode == "rust" then
        -- Enable systemd service for future boots and start now
        os.execute("sudo systemctl enable simonsaysseeq-rust")
        if launch_rust_app() then
            print("SimonSaysSeeqRust launched successfully")
            -- Exit this selector as Rust app takes over
            cleanup()
            return
        else
            print("Failed to launch Rust app, falling back to menu")
            chosen_mode = "menu"
        end
    end
    
    if chosen_mode == "menu" then
        -- Disable auto-start service
        os.execute("sudo systemctl disable simonsaysseeq-rust")
        print("Continuing to normal Norns menu...")
        -- Let Norns continue normal startup
        cleanup()
        return
    end
end

-- Timer and countdown management
local function timer_callback()
    if key_pressed then
        return -- User interaction detected, stop countdown
    end
    
    countdown = countdown - 1
    screen_dirty = true
    
    if countdown <= 0 then
        -- Timeout reached, use saved setting or default to menu
        local saved_mode = read_config()
        if saved_mode == "rust" then
            current_selection = 2 -- Direct Rust App
        else
            current_selection = 1 -- Normal Menu
        end
        select_mode()
    end
end

function cleanup()
    -- Stop timer if running
    if timer then
        timer:stop()
        timer = nil
    end
    
    -- Clear screen
    screen.clear()
    screen.update()
    
    -- This script should exit to let normal Norns startup continue
    print("Boot selector finished")
end

-- Main initialization
function init()
    print("SimonSaysSeeq Boot Selector starting...")
    
    -- Initialize screen
    screen.clear()
    screen.aa(1)
    screen.line_width(1)
    
    -- Set initial state
    start_time = util.time()
    countdown = timeout_seconds
    current_selection = 1
    screen_dirty = true
    key_pressed = false
    
    -- Check if we should skip the menu (e.g., if a mode was just set)
    local skip_file = "/tmp/simonsaysseeq_skip_menu"
    if file_exists(skip_file) then
        os.execute("rm -f " .. skip_file)
        local saved_mode = read_config()
        if saved_mode == "rust" then
            current_selection = 2
        end
        select_mode()
        return
    end
    
    -- Start countdown timer
    timer = metro.init()
    timer.time = 1.0 -- 1 second intervals
    timer.count = -1 -- Run indefinitely until stopped
    timer.callback = timer_callback
    timer:start()
    
    -- Force initial redraw
    redraw()
    
    print("Boot menu displayed - waiting for user input...")
end

-- Cleanup on script end
function cleanup()
    if timer then
        timer:stop()
    end
end

-- Handle script termination
function script_end()
    cleanup()
end