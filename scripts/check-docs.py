"""Check maintained documentation links, guide hashes and publication hygiene.

Hygiene scans every file Git would publish (tracked plus untracked, not ignored).
Generic leak patterns live here. Maintainers can add private terms, one regular
expression per line, to the ignored file .local/publication-blocklist.txt.
"""
from pathlib import Path
import hashlib,json,re,subprocess
root=Path(__file__).resolve().parents[1]
files=[*root.glob('*.md'),*(root/'docs').rglob('*.md'),*(root/'.agents').rglob('*.md'),*(root/'skills').rglob('*.md'),*root.glob('packages/*/README.md')]
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
print(f'PASS: {len(files)} maintained documents (relative links, generated source hashes); {scanned} publishable files scanned for leaks ({len(PRIVATE)} private terms)')
