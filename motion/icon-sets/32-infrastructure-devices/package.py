from pathlib import Path
from zipfile import ZipFile,ZipInfo,ZIP_DEFLATED
import subprocess,sys,json
root=Path(__file__).resolve().parent
project=root.parents[2]
subprocess.run([sys.executable,str(root.parent/'library/package-set.py'),str(root)],check=True)
# Keep the new generation harness and the actual motion design/review packet with the source.
review=root.parent/'library/design-reviews/infrastructure-devices'
extras=[root/'geometry-contract.json',root/'measure-motion.mjs',root/'evidence/motion-measurements.json']
extras += [root.parent/'library'/name for name in ['geometry-subpath.mjs','geometry-subpath.test.mjs','verify-filled-ellipsis.mjs','family-contract.json','generation-context.mjs','preflight.mjs','build-checked.mjs','director-revision/serve-verification.py']]
extras += [p for p in review.rglob('*') if p.is_file() and p.suffix in {'.json','.md','.mjs','.py','.html','.png'} and p.name not in {'assets-before.json','delivery.json','archive-verification.json'} and not any(x.startswith('invalid-') for x in p.parts)]
# Include current acceptance evidence referenced by the per-icon ledger.
finish=root.parent/'library/design-reviews/finish-pending-27'
extras += [finish/name for name in ['scope.md','icon-review-ledger.json','render-summary.json','native-change-scope.json','alarm-light-preservation.json','gallery-verification.json']]
extras += [p for p in (finish/'captures'/'infrastructure-devices').rglob('*') if p.is_file() and p.suffix in {'.json','.png'}]
archive=project/'prototype/icons/32/assets/32-infrastructure-devices.zip'
with ZipFile(archive,'a') as z:
 existing=set(z.namelist())
 for source in extras:
  name='source/'+str(source.relative_to(project))
  if source.exists() and name not in existing:
   entry=ZipInfo(name,(2026,9,15,0,0,0));entry.compress_type=ZIP_DEFLATED;entry.external_attr=0o644<<16
   z.writestr(entry,source.read_bytes(),compresslevel=9);existing.add(name)
 assert z.testzip() is None
