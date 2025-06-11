-- SimonSaysSeeq Boot Selector
-- This script determines whether to start the normal Norns menu or launch directly into Rust app

local config_file = "/home/we/.config/simonsaysseeq/startup_mode"
local rust_binary = "/home/we/dust/code/SimonSaysSeeqRust/simon_says_seeq"
local lua_script = "/home/we/dust/code/SimonSaysSeeqNorns/SimonSaysSeeqNorns.lua"

-- Utility functions
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
        return "menu" -- default mode
    end
    
    local file = io.open(config_file, "r")
    if not file then
        return "menu"
    end
    
    local mode = file:read("*line")
    file:close()
    
    if mode then
        mode = mode:gsub("^%s*(.-)%s*$", "%1") -- trim whitespace
        if mode == "rust" or mode == "menu" then
            return mode
        end
    end
    
    return "menu" -- fallback to default
end

local function write_config(mode)
    -- Ensure config directory exists
    os.execute("mkdir -p " .. config_file:match("(.*/)[^/]*$"))
    
    local file = io.open(config_file, "w")
    if file then
        file:write(mode .. "\n")
        file:close()
        return true
    end
    return false
end

local function log_message(message)
    print("[SimonSaysSeeq Boot] " .. message)
    
    -- Also write to a log file for debugging
    local log_file = io.open("/tmp/simonsaysseeq_boot.log", "a")
    if log_file then
        log_file:write(os.date("%Y-%m-%d %H:%M:%S") .. " - " .. message .. "\n")
        log_file:close()
    end
end

local function launch_rust_app()
    log_message("Attempting to launch Rust application...")
    
    if not file_exists(rust_binary) then
        log_message("ERROR: Rust binary not found at " .. rust_binary)
        log_message("Falling back to menu mode")
        return false
    end
    
    -- Set environment variables for the Rust app
    local env_vars = {
        "RUST_LOG=info",
        "NORNS_MODE=standalone"
    }
    
    local env_string = table.concat(env_vars, " ")
    local command = env_string .. " " .. rust_binary .. " 2>&1 | tee /tmp/simonsaysseeq_rust.log &"
    
    log_message("Executing: " .. command)
    
    -- Launch the Rust application in background
    local result = os.execute(command)
    
    if result == 0 then
        log_message("Rust application launched successfully")
        return true
    else
        log_message("ERROR: Failed to launch Rust application (exit code: " .. tostring(result) .. ")")
        return false
    end
end

local function launch_lua_script()
    log_message("Loading SimonSaysSeeq Lua script...")
    
    if not file_exists(lua_script) then
        log_message("ERROR: Lua script not found at " .. lua_script)
        return false
    end
    
    -- Load the Lua script
    local success, err = pcall(dofile, lua_script)
    
    if success then
        log_message("Lua script loaded successfully")
        return true
    else
        log_message("ERROR: Failed to load Lua script: " .. tostring(err))
        return false
    end
end

local function show_boot_menu()
    -- Simple text-based boot menu for emergency selection
    print("\n=== SimonSaysSeeq Boot Selector ===")
    print("1. Normal Norns Menu (Lua)")
    print("2. Direct Rust Application")
    print("3. Continue with current config")
    print("\nPress 1, 2, or 3 within 5 seconds...")
    
    -- This is a simplified approach - in a real implementation you might
    -- need to handle input differently depending on Norns hardware capabilities
    io.flush()
    
    -- Simple timeout mechanism (this may need adjustment for Norns)
    local start_time = os.time()
    local choice = nil
    
    while os.time() - start_time < 5 do
        -- In a real Norns environment, you'd read from the appropriate input device
        -- This is a placeholder for the concept
        local input = io.read(0) -- non-blocking read attempt
        if input and (input == "1" or input == "2" or input == "3") then
            choice = input
            break
        end
    end
    
    if choice == "1" then
        write_config("menu")
        return "menu"
    elseif choice == "2" then
        write_config("rust")
        return "rust"
    end
    
    -- Default: use current config
    return read_config()
end

-- Main boot selector logic
local function main()
    log_message("SimonSaysSeeq Boot Selector starting...")
    
    local mode = read_config()
    log_message("Current boot mode: " .. mode)
    
    -- Check for emergency override (e.g., holding a key during boot)
    -- This would need to be implemented based on Norns hardware capabilities
    local emergency_override = false -- placeholder
    
    if emergency_override then
        log_message("Emergency override detected - showing boot menu")
        mode = show_boot_menu()
    end
    
    if mode == "rust" then
        log_message("Boot mode: Direct Rust Application")
        
        local success = launch_rust_app()
        if not success then
            log_message("Rust launch failed, falling back to menu mode")
            mode = "menu"
        else
            -- If Rust app launched successfully, we might want to exit this script
            -- or monitor the Rust process
            log_message("Boot selector work complete - Rust app is running")
            return
        end
    end
    
    if mode == "menu" then
        log_message("Boot mode: Normal Norns Menu")
        -- For menu mode, we typically just let Norns continue its normal startup
        -- The Lua script can be selected manually from the menu
        log_message("Continuing with normal Norns startup...")
        return
    end
    
    log_message("Unexpected mode: " .. tostring(mode) .. " - defaulting to menu")
end

-- Execute main function
local success, error_msg = pcall(main)

if not success then
    log_message("ERROR in boot selector: " .. tostring(error_msg))
    -- Fall back to normal Norns behavior
end

log_message("SimonSaysSeeq Boot Selector finished")