"""Replace the mod champions' gameplay JSON inside a TFM2 container (career save / custom DB) with the current mod data.
Usage: patch.py <in> <out> [--check]"""
import sys, zlib, struct, json, glob, os, gzip, binascii
MODS = os.path.expanduser('~/mnt/Teamfight Manager2/mods')
cur = {}
for f in glob.glob(MODS + '/*/champion/*.data_champion'):
    j = json.load(open(f, encoding='utf-8'))
    for k in [k for k in j if k.startswith('view_')]: del j[k]
    cur[j['id']] = j
src, dst = sys.argv[1], sys.argv[2]
b = open(src, 'rb').read()
assert b[:4] == b'TFM2'
gzlen = struct.unpack('<Q', b[13:21])[0]
start = len(b) - gzlen
header, gz = b[:start], b[start:]
assert struct.unpack('<I', b[21:25])[0] == binascii.crc32(gz) & 0xffffffff, 'crc mismatch on input'
p = zlib.decompress(gz, 16 + zlib.MAX_WBITS)
out = bytearray(); k = 0; n_rep = {}; missing = {}
def keys(o, pre=''):
    s = set()
    if isinstance(o, dict):
        for kk, v in o.items(): s.add(pre + '.' + kk); s |= keys(v, pre + '.' + kk)
    elif isinstance(o, list):
        for v in o: s |= keys(v, pre + '[]')
    return s
while True:
    i = p.find(b'1HCM', k)
    if i < 0: out += p[k:]; break
    n = struct.unpack('<Q', p[i + 8:i + 16])[0]
    try:
        old = json.loads(p[i + 16:i + 16 + n]); cid = old.get('id')
    except Exception:
        out += p[k:i + 4]; k = i + 4; continue
    out += p[k:i]
    if cid in cur:
        new = json.dumps(cur[cid], separators=(',', ':'), ensure_ascii=False).encode('utf-8')
        out += p[i:i + 8] + struct.pack('<Q', len(new)) + new
        n_rep[cid] = n_rep.get(cid, 0) + 1
        extra = keys(cur[cid]) - keys(old)
        if extra: missing[cid] = sorted(extra)[:8]
    else:
        out += p[i:i + 16 + n]
    k = i + 16 + n
newgz = gzip.compress(bytes(out), compresslevel=6)
hdr = bytearray(header)
struct.pack_into('<Q', hdr, 13, len(newgz))
struct.pack_into('<I', hdr, 21, binascii.crc32(newgz) & 0xffffffff)
open(dst, 'wb').write(bytes(hdr) + newgz)
print('replaced', n_rep)
print('keys the old copy lacked (new fields):', missing)
# verify: re-read and re-parse every JSON entry
b2 = open(dst, 'rb').read(); g2 = b2[len(b2) - struct.unpack('<Q', b2[13:21])[0]:]
p2 = zlib.decompress(g2, 16 + zlib.MAX_WBITS)
assert struct.unpack('<I', b2[21:25])[0] == binascii.crc32(g2) & 0xffffffff
print('verify sizes', len(p), '->', len(p2), 'ok')
