"""Exercise the handoff with a fake CLI; no model request or production edit."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


class HandoffTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='icon-handoff-test-')
        self.root = Path(self.temp.name)
        original = Path(__file__).resolve().parents[1]
        self.skill = self.root / '.agents/skills/animated-icon-workflow'
        self.skill.mkdir(parents=True)
        for relative in ['SKILL.md', 'scripts/claude_design.py', 'references/polish-review.md', 'references/generation-harness.md']:
            target = self.skill / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(original / relative, target)
        (self.root / 'motion').symlink_to(original.parents[2] / 'motion')
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        self.called = self.root / 'cli-called.txt'
        cli = self.bin / 'claude'
        cli.write_text(f'''#!{sys.executable}
import json, os, sys
from pathlib import Path
with Path(os.environ['ICON_TEST_CALLED']).open('a') as f: f.write('called\\n')
if '--version' in sys.argv: print('isolated-test-cli'); sys.exit(0)
sys.stdin.read()
print(json.dumps({{"result":"Fixture proposal only", "modelUsage":{{"test-stub":{{}}}}, "session_id":"test"}}))
''')
        cli.chmod(0o755)
        self.env = {**os.environ, 'PATH': str(self.bin) + os.pathsep + os.environ['PATH'], 'ICON_TEST_CALLED': str(self.called)}
        self.brief = self.root / 'brief.md'
        self.brief.write_text('Review a notification icon using Bell as a construction reference.')
        self.output = self.root / 'proposal'

    def tearDown(self):
        self.temp.cleanup()

    def run_cli(self, *extra):
        return subprocess.run([sys.executable, str(self.skill / 'scripts/claude_design.py'), str(self.brief), '--neighbor', 'bell', '--output', str(self.output), *extra], env=self.env, text=True, capture_output=True)

    def test_actual_contract_rules_and_geometry_reach_prompt_with_hashes(self):
        result = self.run_cli()
        self.assertEqual(result.returncode, 0, result.stderr)
        prompt = (self.output / 'prompt.md').read_text()
        for relative in ['SKILL.md', 'references/polish-review.md', 'references/generation-harness.md']:
            self.assertIn((self.skill / relative).read_text(), prompt)
        packet_bytes = (self.output / 'generation-context.json').read_bytes()
        packet = json.loads(packet_bytes)
        self.assertEqual(packet['contract']['grid'], 32)
        self.assertTrue(packet['references'][0]['initialOutlineGeometry'])
        self.assertIn(packet_bytes.decode(), prompt)
        invocation = json.loads((self.output / 'invocation.json').read_text())
        self.assertEqual(invocation['generationContextSha256'], hashlib.sha256(packet_bytes).hexdigest())
        for record in invocation['inputs']:
            self.assertEqual(record['sha256'], hashlib.sha256(Path(record['path']).read_bytes()).hexdigest())
        self.assertEqual(invocation['reportedModels'], ['test-stub'])

    def test_missing_canonical_rules_fails_before_calling_claude(self):
        (self.skill / 'references/generation-harness.md').unlink()
        result = self.run_cli()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.called.exists())
        self.assertFalse(self.output.exists())

    def test_unknown_neighbor_fails_before_calling_claude(self):
        result = self.run_cli('--neighbor', 'absent-reference-zz')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Unknown neighbor', result.stderr)
        self.assertFalse(self.called.exists())

    def test_existing_proposal_cannot_be_overwritten(self):
        self.output.mkdir()
        marker = self.output / 'design.md'
        marker.write_text('Accepted prior proposal')
        result = self.run_cli()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(marker.read_text(), 'Accepted prior proposal')
        self.assertFalse(self.called.exists())


if __name__ == '__main__':
    unittest.main()
