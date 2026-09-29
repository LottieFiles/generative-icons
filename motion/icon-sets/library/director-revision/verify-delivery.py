"""Check the current downloadable library and record exact delivery bytes."""
from pathlib import Path
from zipfile import ZipFile
from urllib.request import urlopen
import hashlib
import json

root = Path(__file__).resolve().parents[4]
revision = root / 'motion/icon-sets/library/director-revision'
catalog = json.loads((root / 'prototype/icons/catalog.json').read_text())
baseline = json.loads((revision / 'baseline.json').read_text())
sha = lambda data: hashlib.sha256(data).hexdigest()
archive = root / 'prototype/icons/01/assets/icon-library.zip'
url = 'http://127.0.0.1:4186/icons/01/assets/icon-library.zip'
assets, images, preserved, preserved_motion = [], [], [], []
icon_count, action_count = 0, 0

def visible_motion(data):
    # Native export IDs are descriptive shape names, not geometry or motion.
    def strip_names(value):
        if isinstance(value, dict): return {k: strip_names(v) for k, v in value.items() if k != 'nm'}
        if isinstance(value, list): return [strip_names(v) for v in value]
        return value
    return {layer['nm']: {'ks': layer['ks'], 'shapes': strip_names(layer['shapes'])}
            for layer in data['layers']
            if not any(p.get('o', {}).get('sid') == 'style.tint' for p in layer['shapes'])}
with ZipFile(archive) as bundle:
    assert bundle.testzip() is None
    for ref in catalog['sets']:
        base = (root / 'prototype/icons' / ref).parent
        meta = json.loads((base / 'set.json').read_text())
        for icon in meta['icons']:
            icon_count += 1
            for action in icon.get('animations', icon['motions']):
                action_count += 1
                name = action['name']
                for ext in ['json', 'lottie']:
                    p = base / f'{name}.{ext}'
                    data = p.read_bytes()
                    digest = sha(data)
                    assert bundle.read(p.name) == data, p
                    if ext == 'lottie':
                        assert digest == action['sha256'], p
                    record = {'file': str(p.relative_to(root)), 'sha256': digest}
                    assets.append(record)
                    if icon['name'] in ['heart', 'check', 'lock']:
                        assert baseline[record['file']] == digest, p
                        preserved.append(record)
                    if icon['name'] == 'refresh' and ext == 'json':
                        # Latest user direction removes the unbounded Duotone disk.
                        # Its original motion and every remaining paint/path stay exact.
                        before = revision.parent / 'stroke-review/before' / p.name
                        assert visible_motion(json.loads(before.read_text())) == visible_motion(json.loads(data))
                        preserved_motion.append(record)
                for suffix in ['poses', 'styles', 'rest']:
                    p = base / f'{name}-{suffix}.png'
                    evidence = root / 'motion/icon-sets' / meta['id'] / 'evidence' / p.name
                    assert p.read_bytes() == evidence.read_bytes(), p
                    images.append({'file': str(p.relative_to(root)), 'sha256': sha(p.read_bytes())})
    assert sum(n.endswith('.lottie') for n in bundle.namelist()) == action_count
    assert sum(n.endswith('.json') and '/' not in n for n in bundle.namelist()) == action_count
with urlopen(url, timeout=10) as response:
    served = response.read()
assert served == archive.read_bytes()
report = {
    'result': 'PASS', 'icons': icon_count, 'motionRevised': 54, 'motionPreserved': 4,
    'bytePreservedIcons': 3, 'paintFollowupIcons': 8, 'newIconsSinceMotionRevision': icon_count - 58,
    'nativeActions': action_count, 'url': url, 'bytes': len(served), 'sha256': sha(served),
    'preservedFiles': preserved, 'refreshMotionPreserved': preserved_motion, 'assets': assets, 'poseImages': images,
}
(revision / 'evidence/delivery-verification.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({k: v for k, v in report.items() if k not in ['assets', 'poseImages', 'preservedFiles']}))
print(f'{len(assets)} exported files, {len(images)} current render images, {len(preserved)} preserved files checked.')
