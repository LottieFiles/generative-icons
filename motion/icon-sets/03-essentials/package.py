"""Deterministic dotLottie archives of unchanged Engine shape animations."""
import hashlib
import json
import shutil
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED, ZipInfo

root = Path(__file__).resolve().parent
out = root.parents[2] / 'prototype/icons/03/assets'
record = json.loads((out / 'set.json').read_text())

def write_file(archive, name, data):
    entry = ZipInfo(name, (2026, 9, 6, 0, 0, 0))
    entry.compress_type = ZIP_DEFLATED
    entry.external_attr = 0o644 << 16
    archive.writestr(entry, data, compresslevel=9)

for icon in record['icons']:
    for item in icon['animations']:
        name = item['name']
        target = out / f'{name}.lottie'
        manifest = json.loads((out / f'{name}.manifest.json').read_text())
        manifest['initial'] = {'animation': name}
        manifest['animations'] = [
            {'id': motion['name'], 'initialTheme': 'dark', 'themes': ['dark', 'light']}
            for motion in icon['animations']
        ]
        (out / f'{name}.manifest.json').write_text(json.dumps(manifest))
        with ZipFile(target, 'w') as archive:
            write_file(archive, 'manifest.json', json.dumps(manifest).encode())
            for motion in icon['animations']:
                animation = motion['name']
                write_file(archive, f'a/{animation}.json', (out / f'{animation}.json').read_bytes())
            for theme in ['dark', 'light']:
                write_file(archive, f't/{theme}.json', (out / f'{name}.theme-{theme}.json').read_bytes())
        with ZipFile(target) as archive:
            assert archive.testzip() is None
            for motion in icon['animations']:
                animation = motion['name']
                assert archive.read(f'a/{animation}.json') == (out / f'{animation}.json').read_bytes()
        item['bytes'] = target.stat().st_size
        item['sha256'] = hashlib.sha256(target.read_bytes()).hexdigest()
        print(f'{name}.lottie: {item["bytes"]:,} bytes; all actions preserved')
    lookup = {a['name']: a for a in icon['animations']}
    for motion in icon['motions']:
        motion.update({k: lookup[motion['name']][k] for k in ['bytes','sha256']})
        if 'reverse' in motion:
            motion['reverse'].update({k: lookup[motion['reverse']['name']][k] for k in ['bytes','sha256']})
    icon.update({key: icon['motions'][0][key] for key in ['bytes', 'sha256']})

motions = [motion for icon in record['icons'] for motion in icon['animations']]

encoded = json.dumps(record, indent=2) + '\n'
(out / 'set.json').write_text(encoded)
(root / 'evidence/build.json').write_text(encoded)
verified_images = []
verification = root / 'evidence/browser-verification.json'
if verification.exists():
    proof = json.loads(verification.read_text())
    hashes = {item['name']: item['sha256'] for item in proof.get('assets', [])}
    if proof.get('result') == 'PASS' and all(hashes.get(item['name']) == item['sha256'] for item in motions):
        for item in motions:
            for suffix in ['rest', 'poses', 'styles']:
                name = f'{item["name"]}-{suffix}.png'
                if (root / 'evidence' / name).exists():
                    shutil.copyfile(root / 'evidence' / name, out / name)
                    verified_images.append(name)

with ZipFile(out / 'five-icons.zip', 'w') as bundle:
    for item in motions:
        for extension in ['lottie', 'json']:
            name = f'{item["name"]}.{extension}'
            write_file(bundle, name, (out / name).read_bytes())
    write_file(bundle, 'set.json', encoded.encode())
    write_file(bundle, 'README.txt', (root / 'DOWNLOAD_README.txt').read_bytes())
    for name in verified_images:
        write_file(bundle, name, (out / name).read_bytes())

    # Keep the editable authoring files and native scene data in their original layout.
    source_files = [root / name for name in ['definitions.mjs','build.mjs','verify.mjs','package.py','README.md','DOWNLOAD_README.txt','controller.test.mjs']]
    source_files += sorted((root / 'scenes').glob('*'))
    source_files += sorted((root / 'native').glob('*.provenance.json'))
    source_files += [root.parents[1] / 'src' / name for name in ['export.mjs','engine-scene.js']]
    source_files += [root.parents[1] / 'package.json']
    source_files += [root.parents[2] / 'prototype/icons/catalog.json']
    source_files += [root.parents[2] / 'prototype/icons/01' / name for name in ['app.js','index.html','styles.css','catalog.mjs']]
    source_files += [root.parents[1] / 'icon-sets/library/controller.test.mjs']
    source_files += [root.parents[2] / 'prototype/icons/03' / name for name in ['controller.mjs','app.js','index.html','styles.css']]
    for source in source_files:
        write_file(bundle, 'source/' + str(source.relative_to(root.parents[2])), source.read_bytes())
    write_file(bundle, 'source/REBUILD.txt', b'Install dependencies in source/motion with npm install. Set CREATOR_MCP_BIN to the Engine exporter executable. Run npm run build:essentials and npm run verify:essentials. The Engine exporter is an external prerequisite, not included in this archive. The cumulative preview also needs the Heart, Bell, and Search asset folders from the project. Generated scene files are independently editable Engine authoring data.\n')

    # Ship the current supervised motion review alongside editable source evidence.
    public_polish = root.parent / 'library/design-reviews/public-motion-polish'
    final_review = public_polish / 'final-review.json'
    if final_review.exists() and json.loads(final_review.read_text()).get('result') == 'PASS':
        for name in ['final-review.json','final-review.md','visual-observations.json','light-integrity-screening.json','current-motion-audit.json','traversal-screening.json','traversal-screen-review.json','light-timing-integrity.json','clock-sheet-coverage.json','clock-capture-method.json','pause-screen-review.json','README.md']:
            source = public_polish / name
            if source.exists():
                write_file(bundle, 'source/' + str(source.relative_to(root.parents[2])), source.read_bytes())

    # Include current shared authoring dependencies so the editable source can rebuild.
    source_dependencies = list((root.parent / 'library').glob('*.mjs'))
    source_dependencies += [root.parent / 'library/motion-directions.json']
    source_dependencies += [root / name for name in ['build.mjs','definitions.mjs','verify.mjs','package.py','README.md']]
    source_dependencies += list((root / 'scenes').glob('*.json'))
    source_dependencies += list((root / 'native').glob('*.json'))
    source_dependencies += [root.parents[1] / 'src' / name for name in ['export.mjs','engine-scene.js']]
    source_dependencies += [root.parents[1] / 'package.json']
    source_dependencies += list((root.parents[2] / '.agents/skills/animated-icon-workflow').rglob('*.md'))
    source_dependencies += list((root.parents[2] / '.agents/skills/animated-icon-workflow/scripts').glob('*.py'))
    source_dependencies += [root.parents[2] / 'docs/design-principles.md', root / 'evidence/browser-verification.json']
    included = set(bundle.namelist())
    for source in source_dependencies:
        if source.exists():
            name = 'source/' + str(source.relative_to(root.parents[2]))
            if name not in included:
                write_file(bundle, name, source.read_bytes())
                included.add(name)

    if final_review.exists() and json.loads(final_review.read_text()).get('result') == 'PASS':
        review_sources = set()
        for item in json.loads(final_review.read_text())['icons']:
            if item['set'] != root.name:
                continue
            proposal = root.parents[2] / item['claudeFirstDesign']['file']
            review_sources.update([proposal, proposal.parent / 'supervisor-review.md'])
            review_sources.update(proposal.parent.glob('*invocation*.json'))
            if item.get('finalVisualReview'):
                final_visual = root.parents[2] / item['finalVisualReview']
                review_sources.update([final_visual, final_visual.parent / 'proposal.json', final_visual.parent / 'accepted-replacements.json', final_visual.parent / 'visual-revision.md'])
                review_sources.update(final_visual.parent.glob('*invocation*.json'))
            if item.get('timingReview'):
                timing = root.parents[2] / item['timingReview']['file']
                review_sources.update([timing, timing.with_suffix('.md'), timing.parent.parent / 'supervisor-review.md'])
                review_sources.update(timing.parent.glob('*invocation*.json'))
            if item.get('secondaryReview'):
                secondary = root.parents[2] / item['secondaryReview']
                review_sources.update([secondary, secondary.parent / 'proposal.json'])
                review_sources.update(secondary.parent.glob('*invocation*.json'))
        included = set(bundle.namelist())
        for source in sorted(review_sources):
            if source.exists():
                name = 'source/' + str(source.relative_to(root.parents[2]))
                if name not in included:
                    write_file(bundle, name, source.read_bytes())
                    included.add(name)
