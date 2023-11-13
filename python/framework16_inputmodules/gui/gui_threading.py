# Global GUI variables
STOP_THREAD = False
DISCONNECTED_DEVS = []


def stop_thread():
    global STOP_THREAD
    STOP_THREAD = True


def reset_thread():
    global STOP_THREAD
    STOP_THREAD = False


def is_thread_stopped():
    global STOP_THREAD
    return STOP_THREAD


def is_dev_disconnected(dev):
    global DISCONNECTED_DEVS
    return dev in DISCONNECTED_DEVS


def disconnect_dev(device):
    global DISCONNECTED_DEVS
    DISCONNECTED_DEVS.append(device)
