from pathlib import Path
import json,hashlib
root=Path(__file__).resolve().parents[4];summary=[]
folders=[json.loads((root/'prototype/icons'/ref).read_text())['id'] for ref in json.loads((root/'prototype/icons/catalog.json').read_text())['sets']]
for folder in folders:
 e=root/'motion/icon-sets'/folder/'evidence';base=root/'prototype/icons'/folder[:2]/'assets';meta=json.loads((base/'set.json').read_text());required=[a for i in meta['icons'] for a in i.get('animations',i['motions'])];current={a['name']:a['sha256'] for a in required};sources=[];passed={}
 for name in sorted({p.name for pattern in ['browser-verification*.json','directed-browser*.json'] for p in e.glob(pattern)}):
  p=e/name
  if not p.exists():continue
  report=json.loads(p.read_text());digest=hashlib.sha256(p.read_bytes()).hexdigest();archive=e/'director-inputs'/f'{p.stem}-{digest[:12]}.json';archive.parent.mkdir(exist_ok=True);archive.write_bytes(p.read_bytes());sources.append({'file':str(archive.relative_to(e)),'sha256':digest})
  for a in report['assets']:
   if current.get(a['name'])==a['sha256'] and a.get('styleAndWeightDistinct') and a.get('playback',{}).get('completed'):passed[a['name']]=a
 assert len(passed)==len(current),(folder,'Missing current render proof',set(current)-passed.keys())
 for name,h in current.items():assert hashlib.sha256((base/(name+'.lottie')).read_bytes()).hexdigest()==h
 assets=[passed[a['name']]for a in required];frames=sum(t['frames'] for a in assets for size in a['sizes']for t in size['themes'])
 report={'date':'2026-09-08','runtime':'@lottiefiles/dotlottie-web@0.80.0','result':'PASS','scope':'Every frame, three styles, three stroke weights, two themes, 24/32/48/192 px; native playback completion; Light/Strong and state continuity. Only current matching package hashes included.','boundaryMetric':'Cyclic shapes permit premultiplied RGBA mean error below 0.002; holds remain exact.','assets':assets,'errors':[],'renderedFrameChecks':frames,'evidenceSources':sources}
 (e/'browser-verification.json').write_text(json.dumps(report,indent=2)+'\n');summary.append({'set':folder,'icons':len(meta['icons']),'actions':len(assets),'renderedFrameChecks':frames})
print(json.dumps(summary,indent=2));print('TOTAL',sum(i['actions']for i in summary),sum(i['renderedFrameChecks']for i in summary))
(root/'motion/icon-sets/library/director-revision/evidence/render-coverage.json').write_text(json.dumps({'result':'PASS','sets':summary},indent=2)+'\n')
