from zipfile import ZipFile
from subprocess import run
from shutil import copy2
from pathlib import Path

CS_SAVETRACKER_PATH = Path('./save-helper/WitcherSaveTracker')
RUST_TARGET_PATH = Path('./witcher-track/target/release')
DLL_PATH = CS_SAVETRACKER_PATH / 'bin/Release/net7.0/win-x64/publish/WitcherSaveTracker.exe'

if __name__ == '__main__':
    run(
        'dotnet publish /p:witcher_save_cs=Static -r win-x64 -c Release'.split(),
        cwd=CS_SAVETRACKER_PATH
    )
    run(
        'cargo build --release',
        cwd='./witcher-track',
    )
    copy2(
        DLL_PATH,
        RUST_TARGET_PATH / "save-helper.exe"
    )

    with ZipFile('witcher-track.zip', mode='w') as zip:
        zip.write('witcher-track/target/release/witcher-track.exe', 'witcher-track.exe')
        zip.write('witcher-track/target/release/save-helper.exe', 'save-helper.exe')
