import os
import time

from inputmodule.inputmodule import bootloader_jump
from inputmodule import uf2conv

def dev_to_str(dev):
    return dev.name

def flash_firmware(dev, fw_path):
    print(f"Flashing {fw_path} onto {dev_to_str(dev)}")

    # First jump to bootloader
    drives = uf2conv.list_drives()
    if not drives:
        print("Jump to bootloader")
        bootloader_jump(dev)

    timeout = 10  # 5s
    while not drives:
        if timeout == 0:
            print("Failed to find device in bootloader")
            # TODO: Handle return value
            return False
        # Wait for it to appear
        time.sleep(0.5)
        timeout -= 1
        drives = uf2conv.get_drives()


    if len(drives) == 0:
        print("No drive to deploy.")
        return False

    # Firmware is pretty small, can just fit it all into memory
    with open(fw_path, 'rb') as f:
        fw_buf = f.read()

    for d in drives:
        print("Flashing {} ({})".format(d, uf2conv.board_id(d)))
        uf2conv.write_file(d + "/NEW.UF2", fw_buf)

    print("Flashing finished")

# Example return value
# {
#   '0.1.7': {
#     'ansi': 'framework_ansi_default_v0.1.7.uf2',
#     'gridpad': 'framework_gridpad_default_v0.1.7.uf2'
#   },
#   '0.1.8': {
#     'ansi': 'framework_ansi_default.uf2',
#     'gridpad': 'framework_gridpad_default.uf2',
#   }
# }
def find_releases(res_path, filename_format):
    from os import listdir
    from os.path import isfile, join
    import re

    releases = {}
    try:
        versions = listdir(os.path.join(res_path, "releases"))
    except FileNotFoundError:
        return releases

    for version in versions:
        path = join(res_path, "releases", version)
        releases[version] = {}
        for filename in listdir(path):
            if not isfile(join(path, filename)):
                continue
            type_search = re.search(filename_format, filename)
            if not type_search:
                print(f"Filename '{filename}' not matching patten!")
                sys.exit(1)
                continue
            fw_type = type_search.group(1)
            releases[version][fw_type] = os.path.join(res_path, "releases", version, filename)
    return releases
