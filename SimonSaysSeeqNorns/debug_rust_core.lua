-- Debug script to test Rust core loading on Norns
-- This script provides detailed diagnostics for troubleshooting library loading issues

print("=== SimonSaysSeeq Rust Core Debug Script ===")
print("Platform: " .. (jit and jit.os or "Unknown"))
print("Lua version: " .. _VERSION)

-- Check current working directory
local current_dir = debug.getinfo(1, 'S').source:match [[^@?(.*[\/])[^\/]-$]]
print("Script directory: " .. (current_dir or "Unknown"))

-- Function to check file existence and properties
local function check_file(path)
    local file = io.open(path, "r")
    if file then
        file:close()
        
        -- Try to get file size using shell command if available
        local handle = io.popen("ls -la '" .. path .. "' 2>/dev/null")
        if handle then
            local result = handle:read("*a")
            handle:close()
            if result and result ~= "" then
                print("  File details: " .. result:gsub("\n", ""))
            else
                print("  File exists but cannot get details")
            end
        else
            print("  File exists")
        end
        return true
    else
        print("  File does not exist")
        return false
    end
end

-- Check for library files
print("\n--- Library File Check ---")
local lib_paths = {
    "simon_says_seeq_core.so",
    "./simon_says_seeq_core.so",
    "~/dust/code/SimonSaysSeeqNorns/simon_says_seeq_core.so"
}

for _, path in ipairs(lib_paths) do
    print("Checking: " .. path)
    check_file(path)
end

-- Check Lua's package path
print("\n--- Lua Package Configuration ---")
print("package.path:")
for path in package.path:gmatch("[^;]+") do
    print("  " .. path)
end

print("package.cpath:")
for path in package.cpath:gmatch("[^;]+") do
    print("  " .. path)
end

-- Test different loading methods
print("\n--- Library Loading Tests ---")

-- Test 1: Direct require
print("Test 1: require('simon_says_seeq_core')")
local success1, result1 = pcall(require, 'simon_says_seeq_core')
if success1 then
    print("  ✓ SUCCESS: Library loaded via require")
    print("  Type: " .. type(result1))
    if type(result1) == "table" then
        print("  Functions available:")
        for k, v in pairs(result1) do
            print("    " .. k .. " (" .. type(v) .. ")")
        end
    end
else
    print("  ✗ FAILED: " .. tostring(result1))
end

-- Test 2: Load with full path
print("\nTest 2: package.loadlib with full path")
local full_path = (current_dir or "") .. "simon_says_seeq_core.so"
print("  Trying path: " .. full_path)
local success2, result2 = pcall(package.loadlib, full_path, "luaopen_simon_says_seeq_core")
if success2 and result2 then
    print("  ✓ SUCCESS: Library found with loadlib")
    local success3, module = pcall(result2)
    if success3 then
        print("  ✓ SUCCESS: Module initialized")
        print("  Type: " .. type(module))
    else
        print("  ✗ FAILED to initialize: " .. tostring(module))
    end
else
    print("  ✗ FAILED: " .. tostring(result2))
end

-- Test 3: Check for missing dependencies
print("\nTest 3: Dependency check")
local deps_to_check = {
    "libm.so.6",
    "libc.so.6", 
    "libdl.so.2",
    "libpthread.so.0"
}

for _, dep in ipairs(deps_to_check) do
    local check_cmd = "ldconfig -p | grep " .. dep .. " > /dev/null 2>&1 && echo 'found' || echo 'missing'"
    local handle = io.popen(check_cmd)
    if handle then
        local result = handle:read("*a"):gsub("\n", "")
        handle:close()
        print("  " .. dep .. ": " .. result)
    else
        print("  " .. dep .. ": cannot check")
    end
end

-- Test 4: Check library dependencies (if ldd is available)
print("\nTest 4: Library dependencies")
if check_file(full_path) then
    local ldd_cmd = "ldd '" .. full_path .. "' 2>/dev/null"
    local handle = io.popen(ldd_cmd)
    if handle then
        local result = handle:read("*a")
        handle:close()
        if result and result ~= "" then
            print("  Library dependencies:")
            for line in result:gmatch("[^\n]+") do
                print("    " .. line)
            end
        else
            print("  Cannot check dependencies (ldd not available or file not found)")
        end
    end
end

-- Test 5: Check Lua C module loading capability
print("\nTest 5: Basic C module loading test")
local basic_success, basic_result = pcall(require, 'io')
if basic_success then
    print("  ✓ Basic Lua C modules work")
else
    print("  ✗ Basic Lua C modules failed: " .. tostring(basic_result))
end

-- Test 6: Environment check
print("\nTest 6: Environment")
local env_vars = {"HOME", "USER", "PATH", "LD_LIBRARY_PATH"}
for _, var in ipairs(env_vars) do
    local val = os.getenv(var)
    if val then
        print("  " .. var .. "=" .. val)
    else
        print("  " .. var .. " not set")
    end
end

print("\n=== Debug Complete ===")
print("If the library still doesn't load, check:")
print("1. File was copied correctly (ARM version)")
print("2. File permissions are correct (chmod +x)")
print("3. All dependencies are available")
print("4. Norns has sufficient memory")
print("5. Consider compiling directly on Norns")