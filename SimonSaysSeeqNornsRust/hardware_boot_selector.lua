-- SimonSaysSeeq Hardware Boot Selector
-- Simple boot-time selection using Norns hardware buttons
-- Hold K2 during boot for Rust app, K3 for menu, or wait for default

local config_file = "/home/we/.config/simonsaysseeq/startup_mode"
local timeout_seconds = 5
local check_interval = 0.1
local elapsed_time = 0

-- State tracking
local k2_held = false
local k3_held = false
local selection_made = false
local timer_active = false

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
    print("Launching SimonSaysSeeqRust via systemd...")
    
    -- Enable and start the systemd service
    os.execute("sudo systemctl enable simonsaysseeq-rust")
    local result = os.execute("sudo systemctl start simonsaysseeq-rust")
    
    if result == 0 then
        print("SimonSaysSeeqRust service started successfully")
        return true
    else
        print("Failed to start SimonSaysSeeqRust service")
        return false
    end
end

local function setup_menu_mode()
    print("Setting up normal Norns menu mode...")
    
    -- Disable auto-start service
    os.execute("sudo systemctl disable simonsaysseeq-rust")
    os.execute("sudo systemctl stop simonsaysseeq-rust")
    
    -- Save config
    write_config("menu")
    
    print("Normal menu mode configured")
    return true
end

local function setup_rust_mode()
    print("Setting up direct Rust app mode...")
    
    -- Save config and launch
    write_config("rust")
    
    if launch_rust_app() then
        print("Rust mode configured and launched")
        return true
    else
        print("Rust launch failed, falling back to menu mode")
        setup_menu_mode()
        return false
    end
end

-- Screen display functions
local function show_selection_screen()
    screen.clear()
    screen.level(15)
    
    -- Title
    screen.move(64, 10)
    screen.text_center("SimonSaysSeeq Boot")
    
    -- Instructions
    screen.level(12)
    screen.move(64, 25)
    screen.text_center("Hold button to select:")
    
    -- Button options
    screen.level(k2_held and 15 or 10)
    screen.move(64, 35)
    screen.text_center("K2: Direct Rust App")
    
    screen.level(k3_held and 15 or 10)
    screen.move(64, 43)
    screen.text_center("K3: Normal Menu")
    
    -- Countdown
    screen.level(8)
    screen.move(64, 55)
    local remaining = math.max(0, timeout_seconds - elapsed_time)
    screen.text_center(string.format("Auto-select in %.1fs", remaining))
    
    -- Current saved setting
    screen.level(6)
    screen.move(64, 62)
    local saved = read_config()
    local saved_text = saved == "rust" and "Default: Rust App" or "Default: Menu"
    screen.text_center(saved_text)
    
    screen.update()
end

local function show_starting_screen(mode_text)
    screen.clear()
    screen.level(15)
    screen.move(64, 25)
    screen.text_center("Starting...")
    screen.move(64, 35)
    screen.text_center(mode_text)
    screen.update()
end

-- Main selection logic
local function make_selection()
    if selection_made then return end
    selection_made = true
    
    local chosen_mode = "menu" -- default
    local mode_text = "Normal Menu"
    
    if k2_held then
        chosen_mode = "rust"
        mode_text = "Rust Application"
    elseif k3_held then
        chosen_mode = "menu"
        mode_text = "Normal Menu"
    else
        -- Timeout - use saved setting
        chosen_mode = read_config()
        mode_text = chosen_mode == "rust" and "Rust Application" or "Normal Menu"
    end
    
    show_starting_screen(mode_text)
    
    -- Execute the selection
    if chosen_mode == "rust" then
        setup_rust_mode()
    else
        setup_menu_mode()
    end
    
    -- Clean up and exit
    cleanup()
end

-- Input handlers
function key(n, z)
    if selection_made then return end
    
    if n == 2 then
        k2_held = (z == 1)
        if z == 0 and k2_held then -- K2 released after being held
            make_selection()
        end
    elseif n == 3 then
        k3_held = (z == 1)
        if z == 0 and k3_held then -- K3 released after being held
            make_selection()
        end
    end
end

-- Timer callback for updates and timeout
local function timer_tick()
    if selection_made then return end
    
    elapsed_time = elapsed_time + check_interval
    
    -- Check for timeout
    if elapsed_time >= timeout_seconds then
        make_selection()
        return
    end
    
    -- Update display
    show_selection_screen()
end

function cleanup()
    if timer_active and clock then
        clock.cancel(timer_id)
        timer_active = false
    end
    
    screen.clear()
    screen.update()
    
    print("Hardware boot selector finished")
end

-- Initialization
function init()
    print("SimonSaysSeeq Hardware Boot Selector starting...")
    
    -- Check for skip flag (if selection was just made via SSH)
    local skip_file = "/tmp/simonsaysseeq_skip_boot_menu"
    if file_exists(skip_file) then
        os.execute("rm -f " .. skip_file)
        print("Skipping boot menu - using saved configuration")
        cleanup()
        return
    end
    
    -- Initialize screen
    screen.clear()
    screen.aa(1)
    screen.line_width(1)
    
    -- Reset state
    selection_made = false
    k2_held = false
    k3_held = false
    elapsed_time = 0
    
    -- Show initial screen
    show_selection_screen()
    
    -- Start timer for updates and timeout
    timer_active = true
    timer_id = clock.run(function()
        while timer_active and not selection_made do
            clock.sleep(check_interval)
            timer_tick()
        end
    end)
    
    print("Hardware boot selector ready - hold K2 for Rust, K3 for Menu")
end

-- Cleanup on script termination
function cleanup()
    if timer_active and timer_id then
        clock.cancel(timer_id)
        timer_active = false
    end
end