"""Build the downloadable AI guide from maintained documentation."""
from pathlib import Path
import hashlib,json,re
root=Path(__file__).resolve().parents[3]
skill=root/'.agents/skills/animated-icon-workflow'
out=root/'prototype/icons/guide';out.mkdir(exist_ok=True)
files=[skill/'SKILL.md',*[skill/'references'/n for n in ['polish-review.md','generation-harness.md','claude-design.md','production.md']],root/'motion/icon-sets/library/family-contract.json',root/'docs/using-packages.md',root/'docs/package-contract.md',root/'docs/development.md']
base='https://github.com/LottieFiles/lottie-generative-icons/blob/main/'
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

Repository: https://github.com/LottieFiles/lottie-generative-icons

## Quickstart

1. Clone the repository and run `npm run setup`, then `npm run doctor`.
2. Search existing icons: `node motion/icon-sets/library/generation-context.mjs --query "<meaning>" --search`.
3. Fill in the task brief below and follow the workflow in the first source section.
4. Author source, run preflight, build and verify, then open a pull request.

Requirements: Node 22+, Python 3 and unzip. Native export also needs the Engine exporter, which is not publicly distributed; without it, author the source, run preflight and open a pull request so a maintainer can run the export.

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
# The app skill's runtime reference is generated from the package guide.
# skills/lottie-icons/references/authoring.md is maintained by hand for app context.
portable=root/'skills/lottie-icons/references';portable.mkdir(parents=True,exist_ok=True)
for source,name in [('docs/using-packages.md','runtime.md')]:
 p=root/source
 (portable/name).write_text(public_links(p.read_text(),p))
print('Updated AI guide from',len(files),'canonical sources')
