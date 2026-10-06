"""Check maintained documentation links, retired names, guide hashes and publication hygiene.

Hygiene scans every file Git would publish (tracked plus untracked, not ignored).
Generic leak patterns live here. Maintainers can add private terms, one regular
expression per line, to the ignored file .local/publication-blocklist.txt.
"""
from pathlib import Path
import hashlib,json,re,subprocess
root=Path(__file__).resolve().parents[1]
files=[*root.glob('*.md'),*(root/'docs').rglob('*.md'),*(root/'.agents').rglob('*.md'),*(root/'skills').rglob('*.md'),*root.glob('packages/*/README.md'),
 *root.glob('npm/**/README.md'),*root.glob('crates/README.md'),*(root/'crates/gen-animated-icons/assets/skill').rglob('*.md')]
errors=[]
for p in files:
 text=p.read_text();plain=re.sub(r'```.*?```','',text,flags=re.S)
 for target in re.findall(r'\]\(([^)]+)\)',plain):
  if target.startswith(('http:','https:','mailto:','#')):continue
  target=target.split('#')[0]
  if not (p.parent/target).exists():errors.append(f'{p.relative_to(root)}: missing {target}')
# Machine-specific paths and retired product names, as (fast literal needle, pattern).
# The public product name is "Generative Animated Icons".
PUBLIC=[('/Users/',r'/Users/[^/\s"]+/'),('/home/',r'/home/[^/\s"]+/'),('Users\\',r'[A-Za-z]:\\{1,2}Users\\'),
 ('/private/',r'/private/(?:tmp|var)/'),('/var/folders/',r'/var/folders/'),
 ('Lottie Generative Icons',r'Lottie Generative Icons'),('Motion for Developers',r'Motion for Developers'),
 ('Motion by LottieFiles',r'Motion by LottieFiles'),('LottieFiles motion',r'LottieFiles motion\b')]
blocklist=root/'.local/publication-blocklist.txt'
PRIVATE=[l.strip() for l in blocklist.read_text().splitlines() if l.strip() and not l.startswith('#')] if blocklist.is_file() else []
BINARY={'.lottie','.png','.jpg','.jpeg','.webp','.gif','.ttf','.otf','.woff','.woff2','.wasm','.zip','.mp4','.webm','.ico'}
try:
 listed=subprocess.run(['git','ls-files','-z','-co','--exclude-standard'],cwd=root,capture_output=True,check=True).stdout.decode().split('\0')
 public=[root/n for n in listed if n]
except (OSError,subprocess.CalledProcessError):
 public=files
# Retired setup, local-exporter, CLI and repository names. Maintained docs, templates and the generated
# AI guide point at gen-animated-icons, its repository and the hosted engine. The retired lottie-icons skill and package README
# are frozen, and the changelog keeps history, so they are exempt.
RETIRED=[('CREATOR_MCP_BIN',r'CREATOR_MCP_BIN'),('ENGINE_MCP_ROOT',r'ENGINE_MCP_ROOT'),('Engine exporter',r'Engine exporter'),
 ('npm run ',r'npm run (?:setup|doctor|test:tooling)\b'),('generative-icons-cli',r'@lottiefiles/generative-icons-cli'),
 ('lottie-icons',r'npx lottie-icons\b|\blottie-icons (?:init|info|search|add|diff|author)\b'),
 ('127.0.0.1',r'127\.0\.0\.1'),('localhost',r'\blocalhost\b'),('exporter-config',r'exporter-config|motion/src/doctor'),
 ('lottie-generative-icons',r'LottieFiles/lottie-generative-icons')]
EXEMPT=('skills/lottie-icons/','packages/cli/README.md','CHANGELOG.md')
templates=[*(root/'.github').rglob('*.md'),*(root/'.github').rglob('*.yml'),*root.glob('prototype/icons/guide/*.md'),root/'prototype/icons/guide/index.html']
retired=[(n,re.compile(p)) for n,p in RETIRED]
for p in [*files,*templates]:
 rel=p.relative_to(root).as_posix()
 if rel.startswith(EXEMPT) or not p.is_file():continue
 text=p.read_text()
 for needle,pattern in retired:
  if needle in text and (m:=pattern.search(text)):errors.append(f'{rel}: retired name {m.group(0)!r}; use gen-animated-icons and the hosted engine')
checks=[(n,re.compile(p),p) for n,p in PUBLIC]
if PRIVATE:checks.append((None,re.compile('|'.join(f'(?:{p})' for p in PRIVATE),re.I),'a private term from .local/publication-blocklist.txt'))
scanned=0
for p in public:
 if p.suffix.lower() in BINARY or p.resolve()==Path(__file__).resolve() or not p.is_file():continue
 try:text=p.read_text()
 except UnicodeDecodeError:continue
 scanned+=1
 for needle,pattern,label in checks:
  if (needle is None or needle in text) and pattern.search(text):errors.append(f'{p.relative_to(root)}: publication hygiene matches {label}')
for item in json.loads((root/'prototype/icons/guide/sources.json').read_text()):
 p=root/item['source']
 if hashlib.sha256(p.read_bytes()).hexdigest()!=item['sha256']:errors.append(f'Stale guide source: {item["source"]}')
assert not errors,'\n'.join(errors)
print(f'PASS: {len(files)} maintained documents (relative links, retired names, generated source hashes); {scanned} publishable files scanned for leaks ({len(PRIVATE)} private terms)')
