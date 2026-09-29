"""Package only the four AI sets after their complete scope and current bytes are verified."""
from pathlib import Path
from zipfile import ZipFile,ZipInfo,ZIP_DEFLATED
import hashlib,json
root=Path(__file__).resolve().parents[3]
base=root/'prototype/icons'
scope=json.loads((root/'motion/icon-sets/library/design-reviews/ai-category/scope.json').read_text())
target=base/'01/assets/ai-icons.zip'
def put(z,name,data):
 i=ZipInfo(name,(2026,9,8,0,0,0));i.compress_type=ZIP_DEFLATED;i.external_attr=0o644<<16;z.writestr(i,data,compresslevel=9)
records=[]
with ZipFile(target,'w') as z:
 for entry in scope['sets']:
  out=base/entry['id'][:2]/'assets';meta=json.loads((out/'set.json').read_text())
  assert [i['name'] for i in meta['icons']]==entry['icons'],('Incomplete AI set',entry['id'])
  proof=json.loads((root/'motion/icon-sets'/entry['id']/'evidence/browser-verification.json').read_text())
  passed={a['name']:a['sha256'] for a in proof['assets']}
  assert proof['result']=='PASS'
  for icon in meta['icons']:
   records.append(icon)
   for action in icon['animations']:
    data=(out/(action['name']+'.lottie')).read_bytes()
    assert hashlib.sha256(data).hexdigest()==action['sha256']==passed[action['name']]
    for ext in ['json','lottie']:put(z,action['name']+'.'+ext,(out/(action['name']+'.'+ext)).read_bytes())
  put(z,'metadata/'+entry['id']+'.json',(out/'set.json').read_bytes())
 put(z,'README.txt',f'''AI animated icons / LottieFiles Motion
{len(records)} icons in four sets: conversation and reasoning, knowledge and retrieval, creation and multimodal, agents and model operations.
Each icon has Light and Strong motion. Retained results use a separate reset action; cyclical icons return to an equivalent drawing. The filename selects the initial animation, and each .lottie contains all of that icon's actions. Each JSON contains one action.
Native editable vector paths and transforms,32x32 at60fps. Outline weights1.5/2/2.5. Solid is a separately considered drawing. Duotone18percent fills only actual enclosed surfaces; open icons intentionally match Outline. Named style and primary-color slots support dark/light themes. Runtime reference:@lottiefiles/dotlottie-web0.80.0.
Shared gallery:http://127.0.0.1:4186/icons/01/
Design:Claude Code proposals and revisions, supervised and visually reviewed by Codex. Export:Engine MCP. Technical evidence is included in each set's separate source archive.
'''.encode())
with ZipFile(target) as z:
 assert z.testzip() is None
 actions=sum(n.endswith('.lottie') for n in z.namelist())
report=dict(result='PASS',icons=len(records),actions=actions,bytes=target.stat().st_size,sha256=hashlib.sha256(target.read_bytes()).hexdigest(),path=str(target))
(root/'motion/icon-sets/library/design-reviews/ai-category/delivery-ai.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
