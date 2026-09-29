"""Filesystem and catalog failure tests; no model or native exporter required."""
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from zipfile import ZipFile
import sys

spec = importlib.util.spec_from_file_location('icon_set', Path(__file__).with_name('icon-set.py'))
tool = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tool)


class IconSetTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='icon-set-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for file in ['motion/icon-sets/34-scheduling-time/build.mjs',
                     'motion/icon-sets/34-scheduling-time/verify.mjs', 'prototype/icons/34/verify.html']:
            target = self.root / file
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(tool.ROOT / file, target)
        self.catalog = self.root / 'prototype/icons/catalog.json'
        self.catalog.write_text(json.dumps({'sets': [], 'categories': []}, indent=2) + '\n')
        self.before = self.catalog.read_bytes()

    def scaffold(self):
        return tool.scaffold(self.root, 'landscaping', 'Landscaping')

    def built_fixture(self):
        set_id = self.scaffold()
        manifest = self.root / 'prototype/icons/35/assets/set.json'
        manifest.parent.mkdir()
        manifest.write_text(json.dumps({'id': set_id, 'icons': [{'name': 'lawn-mower'}]}))
        return set_id

    def test_scaffold_creates_required_files_without_registering_or_overwriting(self):
        set_id = self.scaffold()
        folder = self.root / 'motion/icon-sets' / set_id
        for file in ['definitions.mjs', 'build.mjs', 'verify.mjs', 'package.py',
                     'DOWNLOAD_README.txt', 'batches/.gitkeep', 'evidence/design-review.md']:
            self.assertTrue((folder / file).is_file(), file)
        self.assertEqual(self.catalog.read_bytes(), self.before)
        self.assertFalse((self.root / 'prototype/icons/35/assets/set.json').exists())
        marker = folder / 'definitions.mjs'
        marker.write_text('user drawing')
        with self.assertRaisesRegex(ValueError, 'already exists'):
            self.scaffold()
        self.assertEqual(marker.read_text(), 'user drawing')

    def test_paths_and_existing_asset_numbers_are_preserved(self):
        with self.assertRaises(ValueError):
            tool.scaffold(self.root, '../escape')
        (self.root / 'prototype/icons/35').mkdir()
        self.assertEqual(self.scaffold(), '36-landscaping')

    def test_source_zip_keeps_empty_batches_and_exporter_dependencies(self):
        set_id = self.scaffold()
        for relative in ['LICENSE', 'THIRD_PARTY_NOTICES.md',
                         'motion/src/export.mjs', 'motion/src/exporter-config.mjs',
                         'motion/icon-sets/library/verify-filled-ellipsis.mjs']:
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(tool.ROOT / relative, target)
        assets = self.root / 'prototype/icons/35/assets'
        assets.mkdir()
        # This fixture exercises archive contents, not design acceptance.
        (assets / 'set.json').write_text(json.dumps({'icons': []}))
        result = subprocess.run([sys.executable, str(tool.ROOT / 'motion/icon-sets/library/package-set.py'),
                                 str(self.root / 'motion/icon-sets' / set_id)], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        with ZipFile(assets / f'{set_id}.zip') as archive:
            self.assertIsNone(archive.testzip())
            for relative in ['motion/icon-sets/35-landscaping/batches/.gitkeep',
                             'motion/src/exporter-config.mjs',
                             'motion/icon-sets/library/verify-filled-ellipsis.mjs', 'LICENSE']:
                self.assertEqual(archive.read('source/' + relative), (self.root / relative).read_bytes())
            self.assertEqual(archive.read('LICENSE'), (self.root / 'LICENSE').read_bytes())

    def test_registration_requires_a_built_nonempty_matching_manifest(self):
        set_id = self.scaffold()
        with self.assertRaisesRegex(ValueError, 'Build the complete set'):
            tool.register(self.root, set_id)
        manifest = self.root / 'prototype/icons/35/assets/set.json'
        manifest.parent.mkdir()
        for data in [{'id': set_id, 'icons': []}, {'id': '36-other', 'icons': [{}]}]:
            manifest.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError, 'must match'):
                tool.register(self.root, set_id)
        self.assertEqual(self.catalog.read_bytes(), self.before)

    def test_failed_source_or_native_verification_never_changes_catalog(self):
        set_id = self.built_fixture()
        for fail_at in [0, 1]:
            calls = []
            def run(command, **kwargs):
                calls.append(command)
                return subprocess.CompletedProcess(command, int(len(calls) - 1 == fail_at), '', 'fixture failure')
            with self.assertRaisesRegex(ValueError, 'Registration stopped'):
                tool.register(self.root, set_id, run=run)
            self.assertEqual(self.catalog.read_bytes(), self.before)

    def test_registration_is_idempotent_and_rejects_a_category_collision(self):
        set_id = self.built_fixture()
        def success(command, **kwargs):
            return subprocess.CompletedProcess(command, 0, '', '')
        tool.register(self.root, set_id, run=success)
        once = self.catalog.read_bytes()
        tool.register(self.root, set_id, run=success)
        self.assertEqual(self.catalog.read_bytes(), once)
        data = json.loads(once)
        self.assertEqual(data['sets'], ['35/assets/set.json'])
        data['categories'][0]['sets'] = ['34/assets/set.json']
        self.catalog.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, 'already belongs'):
            tool.register(self.root, set_id, run=success)


if __name__ == '__main__':
    unittest.main()
