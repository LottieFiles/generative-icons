from pathlib import Path
from zipfile import ZipFile,ZipInfo,ZIP_DEFLATED
import subprocess,sys,json
root=Path(__file__).resolve().parent
project=root.parents[2]
subprocess.run([sys.executable,str(root.parent/'library/package-set.py'),str(root)],check=True)
# Keep the new generation harness and the actual motion design/review packet with the source.
review=root.parent/'library/design-reviews/tables-analytics'
extras=[root/'geometry-contract.json',root/'measure-motion.mjs',root/'evidence/motion-measurements.json']
extras += [root.parent/'library'/name for name in ['verify-filled-ellipsis.mjs','family-contract.json','generation-context.mjs','preflight.mjs','build-checked.mjs','director-revision/serve-verification.py']]
extras += [p for p in review.rglob('*') if p.is_file() and p.suffix in {'.json','.md','.mjs','.py','.html','.png'} and p.name not in {'assets-before.json','delivery.json','archive-verification.json'} and not any(x.startswith('invalid-') for x in p.parts)]
archive=project/'prototype/icons/28/assets/28-tables-analytics.zip'
with ZipFile(archive,'a') as z:
 existing=set(z.namelist())
 for source in extras:
  name='source/'+str(source.relative_to(project))
  if source.exists() and name not in existing:
   entry=ZipInfo(name,(2026,9,15,0,0,0));entry.compress_type=ZIP_DEFLATED;entry.external_attr=0o644<<16
   z.writestr(entry,source.read_bytes(),compresslevel=9);existing.add(name)
 assert z.testzip() is None
