from pathlib import Path
import subprocess,sys,json
root=Path(__file__).resolve().parent
# The existing shared builder is retained byte-for-byte for older provenance.
# Apply this set's category to its top-level manifest before packaging.
manifest=root.parents[2]/"prototype/icons/11/assets/set.json"
record=json.loads(manifest.read_text());record["category"]="Feedback & status"
manifest.write_text(json.dumps(record,indent=2)+"\n")
subprocess.run([sys.executable,str(root.parent/"library/package-set.py"),str(root)],check=True)
