// Food & Drink shared geometry helpers (arc and curve helpers adapted from 35-weather/batches/sky.mjs).
export const pt=(a,r,c=[16,16])=>[c[0]+r*Math.cos(a*Math.PI/180),c[1]+r*Math.sin(a*Math.PI/180)];
// Fixed-topology circular arc: always n cubic segments.
export const arcN=(cx,cy,r,a,b,n=4)=>{const A=d=>d*Math.PI/180,p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let s=A(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=A(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*r*Math.sin(s),u[1]+k*r*Math.cos(s),v[0]+k*r*Math.sin(e),v[1]-k*r*Math.cos(e),...v];s=e;}return d;};
// Hermite → cubic for a sampled curve fn(u) on [0,1]; n segments.
export function curveFrom(fn,n=4){const h=1e-4,dv=u=>{const a=fn(Math.max(0,u-h)),b=fn(Math.min(1,u+h));const w=Math.min(1,u+h)-Math.max(0,u-h);return[(b[0]-a[0])/w,(b[1]-a[1])/w];};let d='M'+fn(0);for(let i=0;i<n;i++){const u0=i/n,u1=(i+1)/n,p0=fn(u0),p1=fn(u1),d0=dv(u0),d1=dv(u1),k=(u1-u0)/3;d+=' C'+[p0[0]+d0[0]*k,p0[1]+d0[1]*k,p1[0]-d1[0]*k,p1[1]-d1[1]*k,...p1];}return d;}
export const L=d=>d.replace(/^M/,'L');
// A rising steam wisp: an S curve from its base (x, y) up by len.
export const wispD=(x,y,len,w=1.3)=>`M${x} ${y} C${x-w} ${y-len*.33} ${x+w} ${y-len*.66} ${x} ${y-len}`;
// Rise-and-recycle driver pair: the part rises `lift`, fades out, snaps home while invisible, fades back in.
export function recycle(P,d,lift,lp,sp){
 const rise=P([[0,0],[d,0],[d+lp[0],lift[0],'in'],[d+lp[0]+.02,lift[0]],[d+lp[0]+.03,0,'linear']],[[0,0],[sp[2]+d,0],[sp[2]+d+sp[0],lift[1],'in'],[sp[2]+d+sp[0]+.02,lift[1]],[sp[2]+d+sp[0]+.03,0,'linear']]);
 const show=P([[0,1],[d+lp[0]*.45,1],[d+lp[0],0,'in'],[d+lp[0]+.06,0],[d+lp[0]+lp[1],1,'out']],[[0,1],[sp[2]+d+sp[0]*.45,1],[sp[2]+d+sp[0],0,'in'],[sp[2]+d+sp[0]+.06,0],[sp[2]+d+sp[0]+sp[1],1,'out']]);
 return {rise,show};
}
