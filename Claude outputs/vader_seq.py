"""Adds Vader's 'choke_seq' tag: the whole Force choke as one 96-tick action (the skill's own animation, so nothing can
cut it): arm out and fist clenched for 1 s, then two saber swings (the spinning cut from his skill2 frames), timed
to the native slashes at ticks 68 and 86. Reuses existing frames (no new pixels). Usage: python vader_seq.py <fanim>"""
import json, sys
p = sys.argv[1]
a = json.load(open(p, encoding='utf-8-sig'))
an = a['anims']
ch, sw = an['choke']['frames'], an['skill2']['frames']
seq = [(ch[0], 8)] + [(ch[i], 6) for i in (1, 2, 3, 2, 1, 2, 3, 2, 1)] \
    + [(sw[0], 3), (sw[1], 3), (sw[2], 6), (sw[3], 6)] + [(sw[0], 3), (sw[1], 3), (sw[2], 6), (sw[3], 4)]
assert sum(t for _, t in seq) == 96
an['choke_seq'] = {'frames': [{'duration': t / 60, 'data': dict(f['data'])} for f, t in seq]}
json.dump(a, open(p, 'w'))
print('choke_seq', len(seq), 'frames,', sum(t for _, t in seq), 'ticks')
