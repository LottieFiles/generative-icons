"""Build the downloadable AI guide from maintained documentation."""
from pathlib import Path
import hashlib,json,re
root=Path(__file__).resolve().parents[3]
skill=root/'.agents/skills/animated-icon-workflow'
out=root/'prototype/icons/guide';out.mkdir(exist_ok=True)
# The workflow, then the family's direction (what every library icon and the website's generator
# follow), the spec format, the production references and the contract.
spec=root/'crates/generative-icons/assets/skill/references'
files=[skill/'SKILL.md',root/'docs/motion-principles.md',root/'MOTION.md',root/'docs/design-principles.md',skill/'references/polish-review.md',spec/'authoring-v2.md',spec/'review.md',*[skill/'references'/n for n in ['generation-harness.md','production.md']],root/'motion/icon-sets/library/family-contract.json',root/'docs/using-packages.md',root/'docs/package-contract.md',root/'docs/development.md']
base='https://github.com/LottieFiles/generative-icons/blob/main/'
def public_links(text,source):
 def replace(m):
  label,target=m.groups()
  if target.startswith(('http:','https:','mailto:','#')):return m.group(0)
  location,_,fragment=target.partition('#')
  try:rel=(source.parent/location).resolve().relative_to(root)
  except ValueError:return m.group(0)
  return '['+label+']('+base+rel.as_posix()+('#'+fragment if fragment else '')+')'
 return re.sub(r'\[([^\]]+)\]\(([^)]+)\)',replace,text)
preamble='''# Generative Icons — AI operating guide

Give this file to your coding agent to create or revise icons that match the Generative Icons family by LottieFiles. It bundles the canonical workflow, rules and contract from the source repository into one file.

Repository: https://github.com/LottieFiles/generative-icons

## Quickstart

In an app, `npx generative-icons init` installs the generative-icons agent skill, which covers creating a missing icon in the app itself. This guide is the full family workflow used for the library's own sets; give it to your agent for deeper reference.

1. Search existing icons: `generative-icons search "<meaning>"`, then `generative-icons context "<meaning>"` for the reference packet.
2. Read the direction: the motion principles (events, not nudges; draw the subject once), the motion system, the drawing principles and the polish review, then the spec format. They follow the workflow below.
3. Fill in the task brief below and follow the workflow in the first source section.
4. Write each icon as a spec: `generative-icons create <id>`, then `check`, `draft`, `build` and `preview`. In the library repository, build it in a scratch project and add it with `generative-icons library adopt SET-ID --add <build>` (`create <library-id> --replace` and `library adopt` reanimate one), then `library verify` and `verify-visual`; a new set starts with `library new-set` and ends with `register`.

Building exports native Lottie on LottieFiles' hosted engine, so it needs a LottieFiles sign-in (`generative-icons login`). The hosted engine is internal until launch.

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
# The app skill is crates/generative-icons/assets/skill/, embedded in the CLI. skills/lottie-icons/
# is the retired lottie-icons skill, frozen byte for byte for init --migrate, so it is not regenerated.
print('Updated AI guide from',len(files),'canonical sources')
