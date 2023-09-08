with open('clear.bin', 'wb') as f:
    # 256 * 4096 (256 sectors of 4096 bits - The entire flash)
    OneMBit = 1024 * 1024
    f.write(b'\FF' * OneMBit)
