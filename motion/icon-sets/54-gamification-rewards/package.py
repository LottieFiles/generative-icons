from pathlib import Path
import subprocess,sys
import json
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED
root=Path(__file__).resolve().parent
subprocess.run([sys.executable,str(root.parent/"library/package-set.py"),str(root)],check=True)
# Include the direct imports used by this set's helpers and native paint builder.
# Keep this set-local until the shared source-bundle inventory is updated.
repo=root.parents[2]
config=json.loads((root/'set-config.json').read_text())
bundle=repo/'prototype/icons'/config['id'][:2]/'assets'/(config['id']+'.zip')
extras=[root.parent/'library'/name for name in ['ai-authoring.mjs','paint-tokens.mjs','family-contract.json']]
extras += [repo/'prototype/icons/54/review.html']
extras += [repo/'docs'/name for name in ['development.md','verification.md']]
extras += [root/'evidence'/name for name in ['native-small-dark.png','native-small-light.png','native-initial-dark.png','native-equal-time.png'] if (root/'evidence'/name).exists()]
with ZipFile(bundle,'a') as archive:
    present=set(archive.namelist())
    for source in extras:
        name='source/'+str(source.relative_to(repo))
        if name in present:
            continue
        entry=ZipInfo(name,(2026,9,8,0,0,0))
        entry.compress_type=ZIP_DEFLATED
        entry.external_attr=0o644 << 16
        archive.writestr(entry,source.read_bytes(),compresslevel=9)
