from pathlib import Path
import subprocess,sys
root=Path(__file__).resolve().parent
subprocess.run([sys.executable,str(root.parent/"library/package-set.py"),str(root)],check=True)
