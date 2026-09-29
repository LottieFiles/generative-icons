// Independent clearance checks on exported world-space geometry. Generic source
// verification separately checks native transforms against these scene samples.
import fs from 'node:fs';
import assert from 'node:assert/strict';
const root=new URL('./',import.meta.url),read=n=>JSON.parse(fs.readFileSync(new URL('scenes/'+n+'.scene.json',root)));
const sample=(keys,t)=>{if(t<=keys[0][0])return keys[0][1];let i=keys.findIndex(k=>k[0]>=t);if(i<0)return keys.at(-1)[1];const[a,x]=keys[i-1],[b,y]=keys[i],q=(t-a)/(b-a);return x.map((v,j)=>v+(y[j]-v)*q);};
const part=(scene,n)=>{const p=scene.nodes.find(p=>p.name.startsWith(n+'-regular__')||p.name.startsWith(n+'__'));assert.ok(p,'Missing '+n);return p;};
const bounds=p=>({left:Math.min(...p.filter((_,i)=>i%2===0)),right:Math.max(...p.filter((_,i)=>i%2===0))});
function contour(node,t){const pts=sample(node.points,t),out=[];let at=0,last,first;for(const cmd of node.commands){if(cmd===0){last=pts.slice(at,at+=2);first=last;out.push(last);}else if(cmd===1){const end=pts.slice(at,at+=2);for(let i=1;i<=16;i++)out.push(last.map((v,j)=>v+(end[j]-v)*i/16));last=end;}else if(cmd===2){const c1=pts.slice(at,at+=2),c2=pts.slice(at,at+=2),end=pts.slice(at,at+=2);for(let i=1;i<=32;i++){const u=i/32,q=1-u;out.push(last.map((v,j)=>q*q*q*v+3*q*q*u*c1[j]+3*q*u*u*c2[j]+u*u*u*end[j]));}last=end;}else if(cmd===3){out.push(first);last=first;}}return out;}
const segmentDistance=(p,a,b)=>{const dx=b[0]-a[0],dy=b[1]-a[1],l=dx*dx+dy*dy,t=l?Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/l)):0;return Math.hypot(p[0]-a[0]-dx*t,p[1]-a[1]-dy*t);};
const report={result:'PASS',outlineWeight:2.5,minimumQueueGap:Infinity,minimumOfflineGap:Infinity,minimumCloudGap:Infinity,reconnectFrames:0};
for(const mode of ['','-strong']){
 const queued=read('queued'+mode),stop=part(queued,'queued-stop'),ticket=part(queued,'queued-ticket-front-frame');
 for(let f=0;f<=144;f++){const gap=sample(stop.points,f/60)[0]-bounds(sample(ticket.points,f/60)).right-2.5;report.minimumQueueGap=Math.min(report.minimumQueueGap,gap);assert.ok(gap>=1.99,'Ticket and stop merge at Bold');}
 for(const direction of ['','-disconnect']){
  const sc=read('reconnect'+direction+mode);
  for(let f=0;f<=144;f++){
   const at=n=>sample(part(sc,'reconnect-'+n).points,f/60),l=bounds(at('plug-left-frame')),r=bounds(at('plug-right-frame'));
   assert.ok(Math.abs(at('cable-left').at(-2)-l.left)<.0001,'Left cable detaches');assert.ok(Math.abs(at('cable-right')[0]-r.right)<.0001,'Right cable detaches');
   for(const n of ['pin-upper','pin-lower']){const p=at(n);assert.ok(Math.abs(p[0]-l.right)<.0001,'Pin detaches');assert.ok(p.at(-2)<=r.left+.0001,'Pin enters the receiving counter');}
   report.reconnectFrames++;
  }
 }
 const offline=read('offline'+mode);
 for(const n of ['wave-outer-left','wave-outer-right','wave-inner-left','wave-inner-right'])for(const p of contour(part(offline,'offline-'+n),0)){
  const gap=segmentDistance(p,[6,5],[26,27])-2.5;report.minimumOfflineGap=Math.min(report.minimumOfflineGap,gap);assert.ok(gap>.65,'Offline arc and slash lack a readable Bold gap');
 }
 const cloud=read('cloud-sync'+mode),edge=contour(part(cloud,'cloud-sync-cloud'),0);
 for(let f=0;f<=144;f++)for(const n of ['upload','download'])for(const p of contour(part(cloud,'cloud-sync-'+n),f/60)){
  let d=Infinity;for(let j=1;j<edge.length;j++)d=Math.min(d,segmentDistance(p,edge[j-1],edge[j]));const gap=d-2.5;
  if(gap<report.minimumCloudGap){report.minimumCloudGap=gap;report.cloudWorst={mode,part:n,frame:f,point:p};}
 }
}
console.log('Cloud clearance diagnostic',report.minimumCloudGap,report.cloudWorst);assert.ok(report.minimumCloudGap>.5,'Cloud clearance below 0.5 units');
fs.writeFileSync(new URL('evidence/attachment-verification.json',root),JSON.stringify(report,null,2)+'\n');console.log(report);
