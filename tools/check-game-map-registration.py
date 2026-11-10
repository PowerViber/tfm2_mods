"""Check all 900 collision cells against the installed original wall pixels."""
import json,sys
from PIL import Image
doc=json.load(open(sys.argv[1],encoding='utf-8'));wall=Image.open(sys.argv[2]).convert('RGBA')
assert wall.size==(1280,1280)
mismatches=[]
for y in range(30):
    for x in range(30):
        if bool(wall.getpixel((160+x*32+16,160+y*32+16))[3]>128)!=bool(doc['walls'][y][x]):mismatches.append([x,y])
assert not mismatches,('Map registration/collision mismatch',mismatches)
print(json.dumps({'cells_checked':900,'matching':900,'blocked':sum(map(sum,doc['walls'])),'source_padding_pixels':160,'source_pixels_per_cell':32}))
