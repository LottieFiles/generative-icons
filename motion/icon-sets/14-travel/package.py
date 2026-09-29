from pathlib import Path
import subprocess,sys,json
root=Path(__file__).resolve().parent
manifest=root.parents[2]/"prototype/icons/14/assets/set.json"
record=json.loads(manifest.read_text());record["category"]="Travel";record["defaultMotion"]="strong"
manifest.write_text(json.dumps(record,indent=2)+"\n")
subprocess.run([sys.executable,str(root.parent/"library/package-set.py"),str(root)],check=True)

# Keep this set's full review/rebuild path with its editable source bundle.
from zipfile import ZipFile,ZipInfo,ZIP_DEFLATED
review=root.parent/'library/design-reviews/travel';project=root.parents[2]
extras=sorted(review.glob('*.py'))+sorted(review.glob('visual-*'))+sorted(review.glob('motion-before-*.png'))
extras += [review/name for name in ['asset-baseline.json','scope.json','live-interactions.json','delivery-verification.json','family-before-supervisor-refinement.png']]
extras += [root.parent/'library/director-revision/serve-verification.py',project/'prototype/icons/14/family-review.html']
extras += sorted((root/'evidence').glob('family-*.png'))+sorted((root/'evidence').glob('motion-review-*.png'))
with ZipFile(manifest.parent/'14-travel.zip','a') as archive:
 existing=set(archive.namelist())
 for source in extras:
  if not source.exists():continue
  name='source/'+str(source.relative_to(project))
  if name in existing:continue
  entry=ZipInfo(name,(2026,9,8,0,0,0));entry.compress_type=ZIP_DEFLATED;entry.external_attr=0o644<<16
  archive.writestr(entry,source.read_bytes(),compresslevel=9);existing.add(name)
 assert archive.testzip() is None
