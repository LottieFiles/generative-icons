"""Local static preview with a bounded sink for native-renderer evidence."""
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
import json,base64,re,os,argparse
ROOT=Path(__file__).resolve().parents[4]
SETS={p.name[:2]:p.name for p in (ROOT/'motion/icon-sets').glob('[0-9][0-9]-*') if p.is_dir()}
class Handler(SimpleHTTPRequestHandler):
 def __init__(self,*a,**kw):super().__init__(*a,directory=str(ROOT/'prototype'),**kw)
 def do_POST(self):
  if self.path!='/_verification':self.send_error(404);return
  try:
   data=json.loads(self.rfile.read(int(self.headers['Content-Length']))); key=data['set']; folder=SETS[key]
   evidence=ROOT/'motion/icon-sets'/folder/'evidence';evidence.mkdir(exist_ok=True)
   batch=data['report'].get('batch','');suffix='-'+batch if re.fullmatch(r'[a-z0-9-]{1,30}',batch) else ''
   if data['report'].get('result')=='FAIL':
    from datetime import datetime,timezone
    stamp=datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    (evidence/f'failed-render-{stamp}{suffix}.json').write_text(json.dumps(data['report'],indent=2)+'\n')
   (evidence/('directed-pose-capture.json' if data['report'].get('mode')=='capture' else 'directed-browser-progress'+suffix+'.json')).write_text(json.dumps(data['report'],indent=2)+'\n')
   for item in data.get('images',[]):
    if not re.fullmatch(r'[a-z0-9-]+\.png',item['name']):raise ValueError('Image name')
    payload=base64.b64decode(item['data'].split(',',1)[1]);(ROOT/'prototype/icons'/key/'assets'/item['name']).write_bytes(payload);(evidence/item['name']).write_bytes(payload)
   if data['report'].get('result')=='PASS' and data['report'].get('mode')!='capture':(evidence/('browser-verification'+suffix+'.json')).write_text(json.dumps(data['report'],indent=2)+'\n')
   self.send_response(200);self.end_headers();self.wfile.write(b'ok')
  except Exception as e:self.send_error(400,str(e))
 def log_message(self,*args):pass
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--port',type=int,default=4191)
args=parser.parse_args()
print(f'Native verification: http://127.0.0.1:{args.port}/icons/NN/verify.html?save=1',flush=True)
ThreadingHTTPServer(('127.0.0.1',args.port),Handler).serve_forever()
