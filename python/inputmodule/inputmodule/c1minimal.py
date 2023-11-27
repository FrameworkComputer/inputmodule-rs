from inputmodule.inputmodule import send_command, CommandVals

RGB_COLORS = ["white", "black", "red", "green",
              "blue", "cyan", "yellow", "purple"]


def get_color(dev):
    res = send_command(dev, CommandVals.SetColor, with_response=True)
    return (int(res[0]), int(res[1]), int(res[2]))


def set_color(dev, color):
    rgb = None
    if color == "white":
        rgb = [0xFF, 0xFF, 0xFF]
    elif color == "black":
        rgb = [0x00, 0x00, 0x00]
    elif color == "red":
        rgb = [0xFF, 0x00, 0x00]
    elif color == "green":
        rgb = [0x00, 0xFF, 0x00]
    elif color == "blue":
        rgb = [0x00, 0x00, 0xFF]
    elif color == "yellow":
        rgb = [0xFF, 0xFF, 0x00]
    elif color == "cyan":
        rgb = [0x00, 0xFF, 0xFF]
    elif color == "purple":
        rgb = [0xFF, 0x00, 0xFF]
    else:
        print(f"Unknown color: {color}")
        return

    if rgb:
        send_command(dev, CommandVals.SetColor, rgb)
