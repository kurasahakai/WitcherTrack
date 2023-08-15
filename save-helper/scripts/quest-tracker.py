import ctypes
from time import sleep
import pandas as pd
from ctypes import c_char_p
from pathlib import Path

def get_guid_set():
    df = pd.read_xml('Quests.xml')
    df = df[df['type'].notnull()]
    df = df[df['GUID'].notnull()]
    return set(df['GUID'])

def load_dll_function():
    # Load the DLL
    dll_path = r".\bin\Release\net7.0\win-x64\publish\witcher-save-cs.dll"
    dll = ctypes.CDLL(dll_path)

    # Specify the function prototype
    function_name = "export_save"  # Replace with the actual function name in your DLL
    function = getattr(dll, function_name)
    function.argtypes = [c_char_p]
    function.restype = None
    return function

if __name__ == '__main__':
    guid_set = get_guid_set()
    read_savefile = load_dll_function()
    file_path = b"./QuickSave.sav"

    while True:
        try:
            read_savefile(file_path)
            df = pd.read_csv('tw3savefile.csv', engine='python', sep=';;', header=None)
            df = df[df[1] == 'Success']
            count = len(set(df[df[0].isin(guid_set)][0]))
            with open('quest-count.txt', 'w') as fp:
                fp.write(count)
            sleep(5)
        except KeyboardInterrupt:
            break
        break
