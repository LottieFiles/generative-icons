"""Stage only supported gallery files. This builds locally; it does not deploy."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
STATIC = (
    'index.html',
    'assets/favicon.svg', 'assets/lottiefiles-logo-dark.svg',
    'assets/dm-sans-400.ttf', 'assets/dm-sans-600.ttf',
    'assets/FONT-LICENSE.txt', 'assets/dmsans-LICENSE.txt',
    'vendor/dotlottie-web.js', 'vendor/dotlottie-player.wasm',
    'vendor/dotlottie-LICENSE.txt',
    'icons/01/index.html', 'icons/01/app.js', 'icons/01/catalog.mjs',
    'icons/01/styles.css', 'icons/03/controller.mjs', 'icons/catalog.json',
    'icons/guide/index.html', 'icons/guide/guide.css', 'icons/guide/guide.js',
    'icons/guide/ai-operating-guide.md', 'icons/guide/SKILL.md',
    'icons/guide/polish-review.md', 'icons/guide/generation-harness.md',
    'icons/guide/production.md',
    'icons/guide/family-contract.json', 'icons/guide/sources.json',
    'icons/guide/using-packages.md', 'icons/guide/package-contract.md', 'icons/guide/development.md',
)


def public_files(root):
    """No recursive copy: unlisted pages, experiments and asset sidecars stay local."""
    prototype = root / 'prototype'
    result = set(STATIC)
    catalog = json.loads((prototype / 'icons/catalog.json').read_text())
    for relative in catalog['sets']:
        if not re.fullmatch(r'\d{2}/assets/set\.json', relative):
            raise ValueError(f'Invalid public manifest path: {relative}')
        manifest = 'icons/' + relative
        result.add(manifest)
        data = json.loads((prototype / manifest).read_text())
        for icon in data['icons']:
            for action in icon.get('animations', icon['motions']):
                name = action['name']
                if not re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', name):
                    raise ValueError(f'Invalid action name: {name}')
                for suffix in ('.json', '.lottie'):
                    relative_asset = str(Path(manifest).parent / (name + suffix))
                    result.add(relative_asset)
                    asset = prototype / relative_asset
                    if suffix == '.lottie' and hashlib.sha256(asset.read_bytes()).hexdigest() != action['sha256']:
                        raise ValueError(f'Stale catalog hash: {relative_asset}')
    for relative in result:
        source = prototype / relative
        if source.resolve() != prototype.resolve() / relative or not source.is_file():
            raise ValueError(f'Missing or symlinked public input: {relative}')
    return sorted(result)


def build(root, output):
    output = output.absolute()
    if output.exists() or output.is_symlink():
        raise ValueError(f'Output already exists; choose a new --output directory: {output}')
    files = public_files(root)
    output.parent.mkdir(parents=True, exist_ok=True)
    # A failed copy or package check cannot replace a working build.
    with tempfile.TemporaryDirectory(prefix='.site-build-', dir=output.parent) as staging:
        stage = Path(staging) / 'site'
        stage.mkdir()
        for relative in files:
            target = stage / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(root / 'prototype' / relative, target)
        for notice in ('LICENSE', 'THIRD_PARTY_NOTICES.md'):
            shutil.copyfile(root / notice, stage / notice)
        subprocess.run([
            sys.executable, str(root / 'motion/icon-sets/library/package.py'),
            '--output', str(stage / 'icons/01/assets/icon-library.zip'),
        ], check=True, capture_output=True, text=True)
        hashes = {str(p.relative_to(stage)): hashlib.sha256(p.read_bytes()).hexdigest()
                  for p in sorted(stage.rglob('*')) if p.is_file()}
        manifest = {'schemaVersion': 1, 'status': 'site-build', 'files': hashes}
        (stage / 'build-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
        stage.rename(output)
    return {'output': str(output), 'files': len(hashes),
            'scope': 'Public file selection and native package provenance. Not visual approval or deployment.'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'dist/site')
    args = parser.parse_args()
    try:
        print(json.dumps(build(ROOT, args.output), indent=2))
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError):
            print(error.stderr, file=sys.stderr)
        sys.exit(1)
