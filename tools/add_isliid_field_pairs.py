"""Round 93: add the aura field 2-frame pairs (aura_field_<k>_rank<r>_pair0..3) to the existing aura_fields8 sheet and
Isliid's data, without regenerating any art. The native code emits one pair every 12 ticks (it was a single frame every
6). generate_isliid_eight_frame_art.py makes the same entries on a full rebuild. Safe to run twice."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MOD = ROOT / "mods" / "tfm2_custom"
P = "tfm2_isliid_emperor_"


def main() -> None:
    fanim = MOD / "vfx" / "aura_fields8#anim.fanim"
    meta = json.loads(fanim.read_text(encoding="utf-8"))
    anims = meta["anims"]
    added = []
    for r in range(8):
        for k in range(7):
            frames = anims[f"aura_field_{k}_rank{r}"]["frames"]
            for n in range(len(frames) // 2):
                tag = f"aura_field_{k}_rank{r}_pair{n}"
                anims[tag] = {"frames": [frames[2 * n], frames[2 * n + 1]]}
                added.append(tag)
    fanim.write_text(json.dumps(meta, separators=(",", ":")), encoding="utf-8")
    # the editor's manifest lists the same animations (tools/verify_isliid.py checks they match)
    manifest_path = ROOT / "editor" / "isliid-art-manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["fields"] = anims
    manifest_path.write_text(json.dumps(manifest, separators=(",", ":")), encoding="utf-8")
    path = MOD / "champion" / "tfm2_isliid_emperor.data_champion"
    data = json.loads(path.read_text(encoding="utf-8"))
    effects = data["view_effects"]
    have = {e["name"] for e in effects}
    # keep them next to the single-frame field aliases
    last = max(i for i, e in enumerate(effects) if e["name"].startswith(P + "aura_field_"))
    new = [{"type": "Animation", "name": P + tag, "anim": "asset/tfm2_custom/vfx/aura_fields8", "tag": tag, "z": -2,
            "is_follow": False} for tag in added if P + tag not in have]
    effects[last + 1:last + 1] = new
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"{len(added)} field pairs in the sheet, {len(new)} new data aliases")


if __name__ == "__main__":
    main()
