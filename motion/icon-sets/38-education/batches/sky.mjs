// Weather shared geometry. The cloud is the family cloud from 05-interface (x 4.8–27.5, y 8.2–24),
// reused unchanged in shape; weather icons place it with one uniform scale and offset.
import svgpath from 'svgpath';
export const CLOUD='M9 24 C6 24 4.8 22 4.8 19.8 C4.8 17.6 6.3 15.6 8.7 15.3 C8.9 10 13 6.8 17.5 8.8 C20.4 8.4 23.1 11.1 23 14 C25.9 14.2 27.5 16.2 27.5 19 C27.5 22 25.3 24 22.5 24 L9 24 Z';
export const placeCloud=(k,dx,dy)=>svgpath(CLOUD).scale(k).translate(dx,dy).toString();
// Precipitation cloud: x 6.9–25.1, y 4.2–16.8; everything below y 18.8 is free for weather.
export const HIGH_CLOUD=placeCloud(.8,3.06,-2.36);
export const pt=(a,r,c=[16,16])=>[c[0]+r*Math.cos(a*Math.PI/180),c[1]+r*Math.sin(a*Math.PI/180)];
// Fixed-topology circular arc: always n cubic segments, so a sweeping or shrinking arc keeps its topology.
export const arcN=(cx,cy,r,a,b,n=4)=>{const A=d=>d*Math.PI/180,p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let s=A(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=A(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*r*Math.sin(s),u[1]+k*r*Math.cos(s),v[0]+k*r*Math.sin(e),v[1]-k*r*Math.cos(e),...v];s=e;}return d;};
// Hermite → cubic for a sampled curve fn(u) on [0,1]; n segments, derivative by central difference.
export function curveFrom(fn,n=4){const h=1e-4,dv=u=>{const a=fn(Math.max(0,u-h)),b=fn(Math.min(1,u+h));const w=Math.min(1,u+h)-Math.max(0,u-h);return[(b[0]-a[0])/w,(b[1]-a[1])/w];};let d='M'+fn(0);for(let i=0;i<n;i++){const u0=i/n,u1=(i+1)/n,p0=fn(u0),p1=fn(u1),d0=dv(u0),d1=dv(u1),k=(u1-u0)/3;d+=' C'+[p0[0]+d0[0]*k,p0[1]+d0[1]*k,p1[0]-d1[0]*k,p1[1]-d1[1]*k,...p1];}return d;}
