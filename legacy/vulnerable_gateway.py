# legacy/vulnerable_gateway.py
# ⚠️ INTENTIONALLY VULNERABLE - demonstrates why struct.unpack without
# bounds checking is dangerous. this is the "villain" for comparison.
# run this alongside the wasm gateway to see the difference in crash behavior.
#
# the security thesis:
# - python: malformed packet crashes the process → gateway offline
# - wasm: malformed packet crashes the sandbox → gateway recovers in <10ms

import struct
import sys
import time

def parse_modbus_frame(data: bytes) -> dict:
    """
    parse modbus tcp frame using struct.unpack.
    ⚠️ NO BOUNDS CHECKING - will crash on malformed input!
    this is how most legacy python parsers work.
    """
    # these lines will crash if data is too short:
    transaction_id = struct.unpack('>H', data[0:2])[0]   # 💥 if len < 2
    protocol_id = struct.unpack('>H', data[2:4])[0]      # 💥 if len < 4
    length = struct.unpack('>H', data[4:6])[0]           # 💥 if len < 6
    unit_id = data[6]                                     # 💥 if len < 7
    function_code = data[7]                               # 💥 if len < 8
    
    return {
        'transaction_id': transaction_id,
        'protocol_id': protocol_id,
        'length': length,
        'unit_id': unit_id,
        'function_code': function_code,
    }

def demo_crash_behavior():
    """
    demonstrate how the python gateway crashes on malformed input.
    compare this to the wasm gateway which recovers in <10ms.
    """
    print('')
    print('╔═══════════════════════════════════════════════════════════════╗')
    print('║         LEGACY PYTHON GATEWAY - The "Villain"                 ║')
    print('║   ⚠️  INTENTIONALLY VULNERABLE - For Comparison Only          ║')
    print('╚═══════════════════════════════════════════════════════════════╝')
    print('')
    
    # test frames - valid and malformed
    test_frames = [
        ('Valid frame', bytes([0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03, 0x00, 0x00, 0x00, 0x0A])),
        ('Buffer overflow', bytes([0x00, 0x01, 0x00, 0x00, 0x00, 0xFF, 0x01, 0x03])),
        ('Truncated header', bytes([0x00, 0x01, 0x00])),  # 💥 WILL CRASH HERE
        ('Never reached', bytes([0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03])),
    ]
    
    frames_processed = 0
    
    for name, frame in test_frames:
        print(f'[PYTHON] Processing: {name} ({len(frame)} bytes)')
        try:
            result = parse_modbus_frame(frame)
            frames_processed += 1
            print(f'[PYTHON] ✓ Parsed: function_code=0x{result["function_code"]:02X}')
        except Exception as e:
            print(f'[PYTHON] 💥 CRASHED: {type(e).__name__}: {e}')
            print('')
            print('┌─────────────────────────────────────────────────────────────┐')
            print('│  🔴 GATEWAY OFFLINE - Process terminated                    │')
            print('│                                                             │')
            print('│  In a real deployment:                                      │')
            print('│  • PLC connection lost                                      │')
            print('│  • SCADA system loses visibility                            │')
            print('│  • Manual restart required (30-60 seconds)                  │')
            print('│  • Attackers win                                            │')
            print('└─────────────────────────────────────────────────────────────┘')
            print('')
            print(f'[PYTHON] Frames processed before crash: {frames_processed}')
            sys.exit(1)
    
    print(f'[PYTHON] All frames processed: {frames_processed}')

if __name__ == '__main__':
    demo_crash_behavior()
