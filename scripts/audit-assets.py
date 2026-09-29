"""Verify the current catalog, distributed native bytes and archive integrity."""
from pathlib import Path
import hashlib,json,zipfile
root=Path(__file__).resolve().parents[1];icons=root/'prototype/icons';catalog=json.loads((icons/'catalog.json').read_text());ids=set();actions=0
for name in catalog['sets']:
 manifest=icons/name;data=json.loads(manifest.read_text())
 for icon in data['icons']:
  assert icon['name'] not in ids,('Duplicate icon',icon['name']);ids.add(icon['name'])
  for action in icon.get('animations',icon['motions']):
   p=manifest.parent/(action['name']+'.lottie');assert hashlib.sha256(p.read_bytes()).hexdigest()==action['sha256'],('Changed native package',p)
   with zipfile.ZipFile(p) as z:assert z.testzip() is None,('Invalid archive',p)
   assert (manifest.parent/(action['name']+'.json')).exists();actions+=1
print(json.dumps({'result':'PASS','sets':len(catalog['sets']),'icons':len(ids),'actions':actions,'scope':'Catalog IDs, native package hashes, ZIP integrity and loose JSON presence. Not a visual or motion-quality review.'},indent=2))
