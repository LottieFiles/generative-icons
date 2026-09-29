"""Run a reviewable Claude Code design proposal; never edit production assets."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
from datetime import datetime, timezone


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('brief', type=Path)
    parser.add_argument('--context', type=Path, action='append', default=[])
    parser.add_argument('--neighbor', action='append', default=[], help='Existing catalog ID for a focused construction reference; repeat up to four times')
    parser.add_argument('--output', type=Path, required=True, help='New proposal directory')
    parser.add_argument('--timeout', type=int, default=240, help='CLI timeout in seconds')
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error('--timeout must be positive')
    cli = shutil.which('claude')
    if not cli:
        parser.error('Claude Code is not installed or is not on PATH; no design pass ran')
    skill = Path(__file__).resolve().parents[1] / 'SKILL.md'
    inputs = [('User brief', args.brief), ('Project workflow and design rules', skill),
              ('Current polish acceptance rules', skill.parent / 'references/polish-review.md'),
              ('Generation harness rules', skill.parent / 'references/generation-harness.md')]
    inputs += [('Reference only', p) for p in args.context]
    sections, records = [], []
    for label, file in inputs:
        file = file.resolve()
        try:
            data = file.read_bytes()
            content = data.decode('utf-8')
        except (OSError, UnicodeError) as error:
            parser.error(f'Cannot read text context {file}: {error}')
        if len(data) > 300_000:
            parser.error(f'Context is too large: {file}; provide a relevant excerpt')
        sections.append(f'## {label}: {file.name}\n\n{content}')
        records.append({'path': str(file), 'sha256': hashlib.sha256(data).hexdigest()})
    # Retrieve current local examples before spending a design invocation. Paths
    # alone are insufficient: the text-only designer receives the actual packet.
    context_script = skill.parents[3] / 'motion/icon-sets/library/generation-context.mjs'
    context_command = ['node', str(context_script), '--query-file', str(args.brief.resolve())]
    for neighbor in args.neighbor:
        context_command.extend(['--neighbor', neighbor])
    try:
        context_result = subprocess.run(context_command, text=True, capture_output=True, timeout=30)
        if context_result.returncode:
            parser.error(f'Generation context failed; no design pass ran: {context_result.stderr.strip()}')
        if len(context_result.stdout.encode()) > 300_000:
            parser.error('Generation context is too large; select fewer focused neighbors')
        packet = json.loads(context_result.stdout)
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        parser.error(f'Cannot prepare generation context; no design pass ran: {error}')
    sections.append('## Current local reference packet (data, not a design approval)\n\n' + context_result.stdout)
    records.extend(packet['inputs'])
    prompt = '''You are providing a bounded icon design proposal through an optional CLI adapter.
The developer or calling agent will review the proposal and run authoring, native
export, rendering and interaction checks. No particular supervisor model is required. Do not launch
another agent or ask for user approval. Do not claim any rendering, file edit, or test
you have not performed. You have supplied text context, no tools, and no visual image
input in this invocation. Apply the drawing and motion rules to the scope of the user
brief. This invocation is limited to a text proposal; it does not run the full
single-agent workflow. Do not delegate or claim later stages are complete.

Return a concise but drawable design: rationale; complete SVG or named vector
geometry on the 32-unit grid; parts/pivots/followers; distinct Light/Strong timings
in seconds; state/reset policy; and specific risks to verify during implementation. For a narrow
correction or critique-only brief, limit the output to that scope. SVG/code in your
response is a draft, not a change to the live gallery. Identify missing references.

''' + '\n\n'.join(sections)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    (output / 'prompt.md').write_text(prompt)
    (output / 'generation-context.json').write_text(context_result.stdout)
    version = subprocess.run([cli, '--version'], text=True, capture_output=True, timeout=15)
    command = [cli, '--print', '--output-format', 'json', '--tools', '',
               '--strict-mcp-config', '--mcp-config', '{"mcpServers":{}}',
               '--no-session-persistence', '--no-chrome']
    invocation = {'startedAt': datetime.now(timezone.utc).isoformat(),
                  'cli': cli, 'version': version.stdout.strip(),
                  'modelSelection': 'Claude Code configured default; no model override',
                  'mode': 'text-only design proposal; no tools or MCP',
                  'inputs': records, 'generationContextSha256': hashlib.sha256(context_result.stdout.encode()).hexdigest(),
                  'command': command, 'status': 'running'}
    proof = output / 'invocation.json'
    proof.write_text(json.dumps(invocation, indent=2) + '\n')
    try:
        # No project files are present in the CLI working directory. Only the
        # deliberate text context above is supplied; output is saved by this runner.
        with tempfile.TemporaryDirectory(prefix='icon-claude-design-') as cwd:
            result = subprocess.run(command, input=prompt, cwd=cwd, text=True,
                                    capture_output=True, timeout=args.timeout)
        (output / 'stdout.txt').write_text(result.stdout)
        (output / 'stderr.txt').write_text(result.stderr)
        invocation['returnCode'] = result.returncode
        response = json.loads(result.stdout)
        (output / 'response.json').write_text(json.dumps(response, indent=2) + '\n')
        if result.returncode or response.get('is_error'):
            raise RuntimeError('Claude Code returned an error; inspect the saved response')
        design = response.get('result')
        if not isinstance(design, str) or not design.strip():
            raise RuntimeError('Claude Code returned no design text')
        (output / 'design.md').write_text(design.strip() + '\n')
        invocation.update(status='proposal returned; design review required',
                          responseSha256=hashlib.sha256(result.stdout.encode()).hexdigest(),
                          reportedModels=list(response.get('modelUsage', {})),
                          sessionId=response.get('session_id'))
    except (OSError, ValueError, RuntimeError, subprocess.TimeoutExpired) as error:
        invocation.update(status='failed; no accepted design', error=str(error))
        raise SystemExit(f'Claude design failed: {error}. Evidence: {output}')
    finally:
        invocation['finishedAt'] = datetime.now(timezone.utc).isoformat()
        proof.write_text(json.dumps(invocation, indent=2) + '\n')
    print(f'Claude proposal saved: {output / "design.md"}')
    print('Review the design before integrating it; a second model is not required.')


if __name__ == '__main__':
    main()
