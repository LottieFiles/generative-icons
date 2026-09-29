"""Serve the prebuilt gallery without build dependencies."""
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
import argparse, subprocess, sys
p=argparse.ArgumentParser();p.add_argument('--port',type=int,default=4189);a=p.parse_args()
project=Path(__file__).resolve().parents[1]
root=project/'prototype'
if not (root/'icons/01/assets/icon-library.zip').exists():
 subprocess.run([sys.executable,str(project/'motion/icon-sets/library/package.py')],check=True)
class Handler(SimpleHTTPRequestHandler):
 def __init__(self,*args,**kwargs):super().__init__(*args,directory=str(root),**kwargs)
 def log_message(self,*args):pass
 def end_headers(self):self.send_header('Cache-Control','no-store');super().end_headers()
print(f'Generative Animated Icons: http://127.0.0.1:{a.port}/icons/01/',flush=True)
ThreadingHTTPServer(('127.0.0.1',a.port),Handler).serve_forever()
