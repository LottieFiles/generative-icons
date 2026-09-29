"""Test public artifact isolation, path boundaries and failure preservation."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('public_site', Path(__file__).with_name('build-public-site.py'))
site = importlib.util.module_from_spec(spec)
spec.loader.exec_module(site)


class PublicSiteTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for relative in site.STATIC:
            self.write('prototype/' + relative, 'fixture')
        for notice in ('LICENSE', 'THIRD_PARTY_NOTICES.md'):
            self.write(notice, 'fixture notice')
        self.catalog = {'sets': ['01/assets/set.json'], 'categories': []}
        self.write('prototype/icons/catalog.json', json.dumps(self.catalog))
        action = {'name': 'test-icon', 'sha256': hashlib.sha256(b'fixture').hexdigest()}
        self.write('prototype/icons/01/assets/set.json', json.dumps({'icons': [{'motions': [action]}]}))
        for suffix in ('.json', '.lottie'):
            self.write('prototype/icons/01/assets/test-icon' + suffix, 'fixture')

    def write(self, name, text):
        p = self.root / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)
        return p

    def test_unlisted_wip_and_review_files_never_enter_public_build(self):
        excluded = ['prototype/unreleased/index.html', 'prototype/icons/01/verify.html',
                    'prototype/icons/01/assets/draft.json', 'prototype/icons/guide/wip.html',
                    '.local/unreleased/README.md']
        for name in excluded:
            self.write(name, 'UNRELEASED SENTINEL')
        def package(command, **kwargs):
            Path(command[-1]).write_bytes(b'fixture archive')
        with patch.object(site.subprocess, 'run', side_effect=package):
            output = self.root / 'dist/site'
            site.build(self.root, output)
        for name in excluded:
            self.assertFalse((output / name.removeprefix('prototype/')).exists())
        self.assertTrue((output / 'icons/01/assets/test-icon.lottie').is_file())
        report = json.loads((output / 'build-manifest.json').read_text())
        self.assertEqual(report['status'], 'site-build')
        self.assertFalse(any('unreleased' in name or 'wip' in name or '.local' in name for name in report['files']))

    def test_manifest_cannot_select_a_wip_or_parent_path(self):
        for name in ['../.local/unreleased/set.json', 'unreleased/assets/set.json']:
            self.catalog['sets'] = [name]
            self.write('prototype/icons/catalog.json', json.dumps(self.catalog))
            with self.assertRaisesRegex(ValueError, 'Invalid public manifest'):
                site.public_files(self.root)

    def test_symlink_and_changed_native_bytes_are_rejected(self):
        asset = self.root / 'prototype/icons/01/assets/test-icon.lottie'
        asset.write_text('changed')
        with self.assertRaisesRegex(ValueError, 'Stale catalog hash'):
            site.public_files(self.root)
        asset.write_text('fixture')
        target = self.root / 'prototype/index.html'
        target.unlink()
        target.symlink_to(self.write('.local/unreleased/private.html', 'fixture'))
        with self.assertRaisesRegex(ValueError, 'symlinked'):
            site.public_files(self.root)

    def test_existing_output_is_preserved(self):
        output = self.root / 'dist/site'
        marker = self.write('dist/site/keep.txt', 'previous build')
        with self.assertRaisesRegex(ValueError, 'already exists'):
            site.build(self.root, output)
        self.assertEqual(marker.read_text(), 'previous build')

    def test_failed_packaging_leaves_no_publishable_output(self):
        output = self.root / 'dist/site'
        with patch.object(site.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, 'package')):
            with self.assertRaises(subprocess.CalledProcessError):
                site.build(self.root, output)
        self.assertFalse(output.exists())


if __name__ == '__main__':
    unittest.main()
