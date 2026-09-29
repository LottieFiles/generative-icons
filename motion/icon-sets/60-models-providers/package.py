"""Package this complete set with its current native proof and rebuild inputs."""
from pathlib import Path
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED
import subprocess
import sys

root = Path(__file__).resolve().parent
repo = root.parents[2]
subprocess.run([sys.executable, str(root.parent / "library/package-set.py"), str(root)], check=True)

# Keep the shared packager unchanged. Include the paint binder imported by this
# builder and the source-check entry points described in this set's README.
extras = [
    "motion/icon-sets/library/paint-tokens.mjs",
    "motion/icon-sets/library/preflight.mjs",
    "motion/icon-sets/library/build-checked.mjs",
    "motion/icon-sets/library/generation-context.mjs",
    "motion/icon-sets/library/family-contract.json",
    "docs/development.md",
    "docs/verification.md",
]
target = repo / "prototype/icons/60/assets/60-models-providers.zip"
with ZipFile(target, "a") as bundle:
    existing = set(bundle.namelist())
    for relative in extras:
        name = "source/" + relative
        if name in existing:
            continue
        entry = ZipInfo(name, (2026, 9, 8, 0, 0, 0))
        entry.compress_type = ZIP_DEFLATED
        entry.external_attr = 0o644 << 16
        bundle.writestr(entry, (repo / relative).read_bytes(), compresslevel=9)
