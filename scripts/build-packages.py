"""Build the npm packages from selected, hash-checked inputs. Does not publish."""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ['runtime', 'react', 'icons', 'cli']


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write_json(file, value):
    file.parent.mkdir(parents=True, exist_ok=True)
    file.write_text(json.dumps(value, indent=2) + '\n')


def copy(source, destination):
    if source.is_symlink() or not source.is_file():
        raise ValueError(f'Expected a regular build input: {source}')
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)


def action(value):
    result = dict(direction='forward', startState='rest', endState='rest')
    for key in ['name', 'motionId', 'fps', 'frames', 'active', 'rest', 'startState', 'endState', 'continuous', 'direction']:
        if key in value:
            result[key] = value[key]
    if value.get('reverse'):
        result['reverse'] = action(value['reverse'])
    return result


def build():
    version = json.loads((ROOT / 'packages/icons/package.json').read_text())['version']
    source_catalog = json.loads((ROOT / 'prototype/icons/catalog.json').read_text())
    icons, ids, inputs = [], set(), {}
    with tempfile.TemporaryDirectory(prefix='icon-packages-') as tmp:
        stage = Path(tmp)
        for package in PACKAGES:
            source = ROOT / 'packages' / package
            target = stage / package
            target.mkdir()
            for file in (source / 'src').glob('*'):
                if file.is_file() and (file.suffix == '.mjs' or file.name.endswith('.d.ts')):
                    copy(file, target / file.name)
            # The existing tested controller remains canonical during migration.
            if package == 'runtime':
                copy(ROOT / 'prototype/icons/03/controller.mjs', target / 'controller.mjs')
        for relative in source_catalog['sets']:
            if not __import__('re').fullmatch(r'\d{2}/assets/set\.json', relative):
                raise ValueError(f'Invalid catalog path {relative}')
            file = ROOT / 'prototype/icons' / relative
            inputs[str(file.relative_to(ROOT))] = sha(file.read_bytes())
            metadata = json.loads(file.read_text())
            category = next(c for c in source_catalog['categories'] if relative in c['sets'])
            for icon in metadata['icons']:
                identity = icon.get('icon', icon['name'])
                if identity in ids or not __import__('re').fullmatch(r'[a-z][a-z0-9-]*', identity):
                    raise ValueError(f'Invalid or duplicate ID {identity}')
                ids.add(identity)
                motion = next(m for m in icon['motions'] if m['motionId'] == 'strong')
                asset = file.parent / (motion['name'] + '.lottie')
                if asset.is_symlink() or asset.parent != file.parent:
                    raise ValueError('Invalid asset path')
                data = asset.read_bytes()
                if sha(data) != motion['sha256']:
                    raise ValueError(f'Stale asset: {identity}')
                motions = [action(m) for m in icon['motions']]
                with zipfile.ZipFile(asset) as archive:
                    manifest = json.loads(archive.read('manifest.json'))
                    names = {a['id'] for a in manifest['animations']}
                    needed = {a['name'] for m in motions for a in [m, *([m['reverse']] if m.get('reverse') else [])]}
                    if not needed <= names:
                        raise ValueError(f'Bundle lacks required actions: {identity}')
                descriptor = dict(schemaVersion=1, id=identity, name=identity, label=icon['label'], intent=icon.get('intent', ''),
                                  category=category['label'], categoryId=category['id'], family='generative-icons',
                                  stateful=bool(icon.get('stateful', False)), previewReset=bool(icon.get('previewReset', False)),
                                  startState=icon.get('startState', 'rest'), endState=icon.get('endState', 'rest'),
                                  width=icon['width'], height=icon['height'], styles=icon['styles'], weights=icon['strokeWidth']['presets'],
                                  motions=motions, license='MIT', aliases=icon.get('keywords', []),
                                  source=dict(set=metadata['id'], manifest=relative, sha256=inputs[str(file.relative_to(ROOT))]),
                                  asset=dict(path=f'assets/{identity}.lottie', sha256=sha(data), bytes=len(data)))
                icons.append(descriptor)
                copy(asset, stage / 'icons' / descriptor['asset']['path'])
                output = stage / 'icons/icons'
                output.mkdir(exist_ok=True)
                (output / f'{identity}.mjs').write_text('export default ' + json.dumps(descriptor) + ';\n')
                (output / f'{identity}.d.ts').write_text('declare const icon: ' + json.dumps(descriptor) + ';\nexport default icon;\n')
        catalog = dict(schemaVersion=1, version=version, categories=[{k: c[k] for k in ['id', 'label']} for c in source_catalog['categories']], icons=icons)
        write_json(stage / 'icons/catalog.json', catalog)
        # Only canonical public documentation enters the released data package.
        docs = ['using-packages.md', 'using-icons.md', 'package-contract.md', 'creating-icons.md', 'development.md', 'verification.md', 'limitations.md', 'design-principles.md', 'motion-principles.md']
        for name in docs:
            copy(ROOT / 'docs' / name, stage / 'icons/docs' / name)
        copy(ROOT / 'prototype/icons/guide/ai-operating-guide.md', stage / 'icons/docs/ai-operating-guide.md')
        for name in ['icon.schema.json', 'project.schema.json', 'release.schema.json']:
            copy(ROOT / 'schemas' / name, stage / 'icons/schemas' / name)
        for target in ['runtime.md', 'authoring.md']:
            copy(ROOT / 'skills/lottie-icons/references' / target, stage / 'cli/skill/references' / target)
        copy(ROOT / 'skills/lottie-icons/SKILL.md', stage / 'cli/skill/SKILL.md')
        revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
        dirty = bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip())
        # A build from uncommitted sources is never a release; pack-release refuses it by default.
        release = dict(schemaVersion=1, version=version, status='prerelease-dev' if dirty else 'release', sourceCommit=revision, sourceDirty=dirty,
                       runtimeVersion=version, cliVersion=version, playerVersion='0.80.0', exporter=dict(requiredFor='authoring-only', distribution='external-not-bundled'),
                       inputs=inputs,
                       files={str(p.relative_to(stage / 'icons')): sha(p.read_bytes()) for p in sorted((stage / 'icons').rglob('*')) if p.is_file()})
        write_json(stage / 'icons/release.json', release)
        for package in PACKAGES:
            target = ROOT / 'packages' / package
            if (target / 'dist').exists():
                shutil.rmtree(target / 'dist')
            shutil.copytree(stage / package, target / 'dist')
            copy(ROOT / 'LICENSE', target / 'LICENSE')
        # packages/icons/THIRD_PARTY_NOTICES.md is maintained per package: it lists only what the tarball bundles.
    state = 'uncommitted sources, status prerelease-dev' if dirty else 'clean source tree, status release'
    print(f'Built {len(icons)} icon descriptors/assets and four packages ({version}; {state}). Nothing was published.')


if __name__ == '__main__':
    build()
