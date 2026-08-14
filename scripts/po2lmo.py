#!/usr/bin/env python3
import sys
import struct
import re

def sfh_hash(data: bytes, init: int) -> int:
    length = len(data)
    if length == 0:
        return 0
    hash_val = init & 0xFFFFFFFF
    rem = length & 3
    idx = 0
    loop_cnt = length >> 2
    for _ in range(loop_cnt):
        w0 = data[idx] | (data[idx+1] << 8)
        w1 = data[idx+2] | (data[idx+3] << 8)
        hash_val = (hash_val + w0) & 0xFFFFFFFF
        tmp = ((w1 << 11) ^ hash_val) & 0xFFFFFFFF
        hash_val = ((hash_val << 16) ^ tmp) & 0xFFFFFFFF
        idx += 4
        hash_val = (hash_val + (hash_val >> 11)) & 0xFFFFFFFF

    if rem == 3:
        w0 = data[idx] | (data[idx+1] << 8)
        hash_val = (hash_val + w0) & 0xFFFFFFFF
        hash_val = (hash_val ^ (hash_val << 16)) & 0xFFFFFFFF
        c = data[idx+2]
        if c >= 128: c -= 256
        hash_val = (hash_val ^ (c << 18)) & 0xFFFFFFFF
        hash_val = (hash_val + (hash_val >> 11)) & 0xFFFFFFFF
    elif rem == 2:
        w0 = data[idx] | (data[idx+1] << 8)
        hash_val = (hash_val + w0) & 0xFFFFFFFF
        hash_val = (hash_val ^ (hash_val << 11)) & 0xFFFFFFFF
        hash_val = (hash_val + (hash_val >> 17)) & 0xFFFFFFFF
    elif rem == 1:
        c = data[idx]
        if c >= 128: c -= 256
        hash_val = (hash_val + c) & 0xFFFFFFFF
        hash_val = (hash_val ^ (hash_val << 10)) & 0xFFFFFFFF
        hash_val = (hash_val + (hash_val >> 1)) & 0xFFFFFFFF

    hash_val = (hash_val ^ (hash_val << 3)) & 0xFFFFFFFF
    hash_val = (hash_val + (hash_val >> 5)) & 0xFFFFFFFF
    hash_val = (hash_val ^ (hash_val << 4)) & 0xFFFFFFFF
    hash_val = (hash_val + (hash_val >> 17)) & 0xFFFFFFFF
    hash_val = (hash_val ^ (hash_val << 25)) & 0xFFFFFFFF
    hash_val = (hash_val + (hash_val >> 6)) & 0xFFFFFFFF
    return hash_val

def parse_po(po_path):
    entries = []
    with open(po_path, 'r', encoding='utf-8') as f:
        content = f.read()

    blocks = re.split(r'\n\s*\n', content)
    for b in blocks:
        msgid_match = re.search(r'msgid\s+(".*?"(?:\s*\n\s*".*?")*)', b)
        msgstr_match = re.search(r'msgstr\s+(".*?"(?:\s*\n\s*".*?")*)', b)
        if msgid_match and msgstr_match:
            def clean_str(s):
                parts = re.findall(r'"(.*?)"', s)
                res = "".join(parts)
                res = res.replace('\\n', '\n').replace('\\t', '\t').replace('\\"', '"').replace('\\\\', '\\')
                return res
            try:
                k = clean_str(msgid_match.group(1))
                v = clean_str(msgstr_match.group(1))
                if k and v:
                    entries.append((k, v))
            except Exception:
                pass
    return entries

def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <input.po> <output.lmo>")
        sys.exit(1)
    
    in_po = sys.argv[1]
    out_lmo = sys.argv[2]
    entries = parse_po(in_po)
    
    data_bytes = bytearray()
    index_entries = []
    offset = 0

    for k, v in entries:
        k_bytes = k.encode('utf-8')
        v_bytes = v.encode('utf-8')
        key_id = sfh_hash(k_bytes, len(k_bytes))
        val_id = sfh_hash(v_bytes, len(v_bytes))
        if key_id != val_id:
            v_len = len(v_bytes)
            index_entries.append((key_id, 1, offset, v_len))
            data_bytes.extend(v_bytes)
            pad = (4 - (v_len % 4)) % 4
            data_bytes.extend(b'\x00' * pad)
            offset += v_len + pad

    index_entries.sort(key=lambda x: x[0])
    
    with open(out_lmo, 'wb') as f:
        f.write(data_bytes)
        for key_id, val_id, off, length in index_entries:
            f.write(struct.pack('>IIII', key_id, val_id, off, length))
        f.write(struct.pack('>I', offset))
    
    print(f"Wrote {len(index_entries)} translations to {out_lmo}")

if __name__ == '__main__':
    main()
