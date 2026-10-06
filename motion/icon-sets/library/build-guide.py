"""Build the downloadable AI guide from maintained documentation."""
from pathlib import Path
import hashlib,json,re
root=Path(__file__).resolve().parents[3]
skill=root/'.agents/skills/animated-icon-workflow'
out=root/'prototype/icons/guide';out.mkdir(exist_ok=True)
files=[skill/'SKILL.md',*[skill/'references'/n for n in ['polish-review.md','generation-harness.md','production.md']],root/'motion/icon-sets/library/family-contract.json',root/'docs/using-packages.md',root/'docs/package-contract.md',root/'docs/development.md']
base='https://github.com/LottieFiles/gen-animated-icons/blob/main/'
def public_links(text,source):
 def replace(m):
  label,target=m.groups()
  if target.startswith(('http:','https:','mailto:','#')):return m.group(0)
  location,_,fragment=target.partition('#')
  try:rel=(source.parent/location).resolve().relative_to(root)
  except ValueError:return m.group(0)
  return '['+label+']('+base+rel.as_posix()+('#'+fragment if fragment else '')+')'
 return re.sub(r'\[([^\]]+)\]\(([^)]+)\)',replace,text)
preamble='''# Generative Animated Icons — AI operating guide

Give this file to your coding agent to create or revise icons that match the Generative Animated Icons family by LottieFiles. It bundles the canonical workflow, rules and contract from the source repository into one file.

Repository: https://github.com/LottieFiles/gen-animated-icons

## Quickstart

In an app, `npx gen-animated-icons init` installs the gen-animated-icons agent skill, which covers creating a missing icon in the app itself. This guide is the full family workflow used for the library's own sets; give it to your agent for deeper reference.

1. Search existing icons: `gen-animated-icons search "<meaning>"`, then `gen-animated-icons context "<meaning>"` for the reference packet.
2. Fill in the task brief below and follow the workflow in the first source section.
3. In an app: `gen-animated-icons create <id>`, then `check`, `build` and `preview`. In the library repository: `gen-animated-icons library new-set`, `preflight`, `build`, `verify`, `verify-visual` and `register`.

Building exports native Lottie on LottieFiles' hosted engine, so it needs a LottieFiles sign-in (`gen-animated-icons login`). The hosted engine is internal until launch.

## Task brief

- Icon name and meaning:
- Product context and user action:
- Existing neighbor IDs (search first):
- Initial, completed and reverse/reset states:
- Details to preserve or avoid:
- Animated Light/Strong pair, or explicitly static only:

The sections below are copied from the maintained guides; their paths and hashes are in sources.json. Relative links point to the source repository.
'''
parts=[preamble];manifest=[]
for p in files:
 b=p.read_bytes();rel=str(p.relative_to(root));manifest.append({'source':rel,'sha256':hashlib.sha256(b).hexdigest()})
 text=public_links(b.decode(),p) if p.suffix=='.md' else b.decode()
 parts.append('\n\n---\n\n## Source: '+rel+'\n\n'+text)
 (out/p.name).write_text(text)
(out/'ai-operating-guide.md').write_text(''.join(parts))
(out/'sources.json').write_text(json.dumps(manifest,indent=2)+'\n')
# The app skill is crates/gen-animated-icons/assets/skill/, embedded in the CLI. skills/lottie-icons/
# is the retired lottie-icons skill, frozen byte for byte for init --migrate, so it is not regenerated.
print('Updated AI guide from',len(files),'canonical sources')
