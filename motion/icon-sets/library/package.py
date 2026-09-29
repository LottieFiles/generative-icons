"""Package the current catalog. Add future set manifests to prototype/icons/catalog.json."""
from pathlib import Path
from zipfile import ZipFile,ZipInfo,ZIP_DEFLATED
import argparse,hashlib,json
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output',type=Path,help='Write an isolated archive without changing repository delivery evidence')
args=parser.parse_args()
root=Path(__file__).resolve().parents[3]
icons=root/'prototype/icons'
catalog=json.loads((icons/'catalog.json').read_text())
records=[]
target=args.output if args.output else icons/'01/assets/icon-library.zip'
def write(z,name,data):
 info=ZipInfo(name,(2026,9,7,0,0,0));info.compress_type=ZIP_DEFLATED;info.external_attr=0o644<<16;z.writestr(info,data,compresslevel=9)
with ZipFile(target,'w') as z:
 for notice in ['LICENSE','THIRD_PARTY_NOTICES.md']:
  write(z,notice,(root/notice).read_bytes())
 for manifest in catalog['sets']:
  folder=(icons/manifest).parent
  data=json.loads((icons/manifest).read_text())
  proof=json.loads((root/'motion/icon-sets'/data['id']/'evidence/browser-verification.json').read_text())
  assert proof.get('result')=='PASS',('Native proof incomplete',data['id'])
  verified={a['name']:a['sha256'] for a in proof['assets']}
  for icon in data['icons']:
   records.append(icon)
   for action in icon.get('animations',icon['motions']):
    for ext in ['lottie','json']:
     f=folder/f'{action["name"]}.{ext}'
     if ext=='lottie':assert hashlib.sha256(f.read_bytes()).hexdigest()==action['sha256']==verified.get(action['name']),('Native proof stale',action['name'])
     write(z,f.name,f.read_bytes())
  write(z,'metadata/'+data['id']+'.json',json.dumps(data,indent=2).encode())
 write(z,'README.txt',f'Generative Animated Icons by LottieFiles\n{len(records)} original icons, Light and Strong motion. Stateful icons include explicit reverse or preview-reset actions.\nNative vector Lottie: 32 x 32, 60 fps. Each .lottie contains all actions for its icon; the filename selects its initial action. Each .json contains one action. Named color and style slots remain editable. Metadata records native provenance, timings, and hashes.\nGallery default: Strong motion, Outline, Regular weight, dark theme. Loading indicators marked continuous:true use native player looping; use frame0 for reduced motion. Themes: dark/light. Style slots: style.outline.light, style.outline.regular, style.outline.bold, style.solid, style.tint (Duotone: 18). Stroke-width token: style.outline.width (the Regular outline at any weight, 1 to 3). Primary color slot: color.primary.\nSource and documentation: https://github.com/LottieFiles/lottie-generative-icons\n'.encode())
with ZipFile(target) as z:
 assert z.testzip() is None
 count=sum(n.endswith('.lottie') for n in z.namelist())
proof={'icons':len(records),'actions':count,'bytes':target.stat().st_size,'sha256':hashlib.sha256(target.read_bytes()).hexdigest(),'path':(target.resolve().relative_to(root).as_posix() if target.resolve().is_relative_to(root) else target.name)}
if args.output is None:
 (root/'motion/icon-sets/library/evidence/delivery.json').write_text(json.dumps(proof,indent=2)+'\n')
print(json.dumps(proof))
