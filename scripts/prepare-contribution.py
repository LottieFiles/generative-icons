"""Prepare an explicit source/evidence ZIP for review; never upload or submit."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import tempfile
import os
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def prepare(set_id, output, evidence):
    if not re.fullmatch(r'\d{2}-[a-z][a-z0-9-]*', set_id):
        raise ValueError('Use a source set ID')
    source = ROOT / 'motion/icon-sets' / set_id
    if not source.is_dir() or source.is_symlink():
        raise ValueError('Source set is missing or is a symlink')
    files = [source / n for n in ['definitions.mjs', 'set-config.json', 'build.mjs', 'verify.mjs']]
    if (source / 'batches/.gitkeep').is_file():
        files.append(source / 'batches/.gitkeep')
    for p in source.rglob('*.mjs'):
        if not any(part in ['evidence', 'native', 'scenes', '.local', 'wip', 'unreleased'] for part in p.relative_to(source).parts) and not p.name.endswith('.test.mjs'):
            files.append(p)
    for relative in evidence:
        p = source / relative
        if '..' in Path(relative).parts or Path(relative).is_absolute() or not relative.startswith('evidence/') or p.suffix not in ['.json', '.md', '.png', '.webp']:
            raise ValueError('Select evidence files explicitly under the source set evidence/ directory')
        files.append(p)
    files += [ROOT / 'LICENSE', ROOT / 'THIRD_PARTY_NOTICES.md']
    payload = {}
    for file in sorted(set(files)):
        if file.is_symlink() or any(p.is_symlink() for p in file.parents if p != ROOT) or not file.is_file():
            raise ValueError(f'Expected regular source file: {file}')
        relative = file.relative_to(ROOT).as_posix()
        if any(part.lower() in ['.local', 'wip', 'unreleased'] for part in file.parts):
            raise ValueError('WIP content is excluded')
        payload[relative] = file.read_bytes()
    manifest = dict(schemaVersion=1, set=set_id, status='prepared-for-review',
                    files={name: hashlib.sha256(data).hexdigest() for name, data in payload.items()},
                    dependencies='Apply to a compatible authoring checkout; shared toolchain is not included',
                    review='Source, native output, visual review, continuous playback and owner acceptance remain separate')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.contribution-', dir=output.parent) as staging:
        pending = Path(staging) / 'bundle.zip'
        with zipfile.ZipFile(pending, 'w', zipfile.ZIP_DEFLATED) as archive:
            for name, data in payload.items():
                archive.writestr(name, data)
            archive.writestr('contribution.json', json.dumps(manifest, indent=2) + '\n')
        # Atomic creation, refusing an existing output even during a race.
        os.link(pending, output)
    print(json.dumps(dict(output=str(output), **manifest), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('set_id')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--evidence', action='append', default=[])
    args = parser.parse_args()
    try:
        prepare(args.set_id, args.output, args.evidence)
    except (ValueError, OSError) as error:
        parser.exit(1, str(error) + '\n')
