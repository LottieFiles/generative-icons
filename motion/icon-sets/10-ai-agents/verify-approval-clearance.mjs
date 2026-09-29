// Independent point-to-segment clearance on exported centerlines at 240 Hz.
import fs from 'node:fs';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
const root=new URL('./',import.meta.url),path=new URL('scenes/approval-step-strong.scene.json',root),bytes=fs.readFileSync(path),scene=JSON.parse(bytes);
const gate=scene.nodes.find(n=>n.name.includes('gate-bold__')),token=scene.nodes.find(n=>n.name.includes('token-frame-bold__'));
const at=(keys,t)=>{let i=0;while(i<keys.length-2&&keys[i+1][0]<t)i++;const[a,v]=keys[i],[b,w]=keys[i+1];const q=Math.max(0,Math.min(1,(t-a)/(b-a)));return v.map((x,j)=>x+(w[j]-x)*q)};
let minimum=Infinity,atSeconds=0;
for(let k=0;k<=240;k++){
 const t=k/240,[ax,ay,bx,by]=at(gate.points,t),p=at(token.points,t),xs=p.filter((_,i)=>i%2===0),ys=p.filter((_,i)=>i%2===1),cx=(Math.min(...xs)+Math.max(...xs))/2,cy=(Math.min(...ys)+Math.max(...ys))/2;
 const u=Math.max(0,Math.min(1,((cx-ax)*(bx-ax)+(cy-ay)*(by-ay))/((bx-ax)**2+(by-ay)**2)));
 const gap=Math.hypot(cx-ax-u*(bx-ax),cy-ay-u*(by-ay))-3.25-2.5;
 if(gap<minimum){minimum=gap;atSeconds=t}
}
assert(minimum>=.75,`Approval token/gate Bold gap closed: ${minimum}`);
const proof={result:'PASS',checks:241,minimumBoldGap:minimum,atSeconds,sceneSha256:crypto.createHash('sha256').update(bytes).digest('hex')};
fs.writeFileSync(new URL('evidence/approval-clearance-verification.json',root),JSON.stringify(proof,null,2)+'\n');console.log(proof);
