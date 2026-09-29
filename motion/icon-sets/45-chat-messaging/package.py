from pathlib import Path
import subprocess, sys
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED
root = Path(__file__).resolve().parent
subprocess.run([sys.executable, str(root.parent / 'library/package-set.py'), str(root)], check=True)
# Include the set-owned review tool and exact native review captures in the source archive.
bundle = root.parents[2] / 'prototype/icons/45/assets/45-chat-messaging.zip'
extras = [root / 'capture-review.mjs', root / 'evidence/native-review-capture.json']
extras += sorted((root / 'evidence').glob('native-small-sizes-*.png'))
extras += sorted((root / 'evidence').glob('native-equal-time-*.png'))
with ZipFile(bundle, 'a') as archive:
    for source in extras:
        if not source.exists():
            continue
        entry = ZipInfo('source/' + str(source.relative_to(root.parents[2])), (2026, 9, 8, 0, 0, 0))
        entry.compress_type = ZIP_DEFLATED
        entry.external_attr = 0o644 << 16
        archive.writestr(entry, source.read_bytes(), compresslevel=9)
