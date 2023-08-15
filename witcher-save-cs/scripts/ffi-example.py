import ctypes
from ctypes import c_char_p
from pathlib import Path

# Load the DLL
dll_path = r".\bin\Release\net7.0\win-x64\publish\witcher-save-cs.dll"
dll = ctypes.CDLL(dll_path)

# Specify the function prototype
function_name = "export_save"  # Replace with the actual function name in your DLL
function = getattr(dll, function_name)
function.argtypes = [c_char_p]
function.restype = None

# Call the function
file_path = b"./QuickSave.sav"  # Replace with the path you want to pass to the function
function(file_path)

