# Global GUI variables
DISCONNECTED_DEVS = []
STATUS = ''

def set_status(status):
    global STATUS
    STATUS = status

def get_status():
    global STATUS
    return STATUS

def stop_thread():
    global STATUS
    STATUS = 'STOP_THREAD'


def reset_thread():
    global STATUS
    if STATUS == 'STOP_THREAD':
        STATUS = ''


def is_thread_stopped():
    global STATUS
    return STATUS == 'STOP_THREAD'


def is_dev_disconnected(dev):
    global DISCONNECTED_DEVS
    return dev in DISCONNECTED_DEVS


def disconnect_dev(device):
    global DISCONNECTED_DEVS
    DISCONNECTED_DEVS.append(device)
