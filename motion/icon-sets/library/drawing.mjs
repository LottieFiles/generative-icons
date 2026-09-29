import svgpath from 'svgpath';

export const mix=(a,b,p)=>a+(b-a)*p;
// Each interval is a deliberate eased move or a still hold. Sample once at 60 fps.
export function phrase(keys,t){if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i];return mix(a[1],b[1],(1-Math.cos(Math.PI*(t-a[0])/(b[0]-a[0])))/2);}
export function rotate(x,y,a,cx=16,cy=16){const r=a*Math.PI/180,c=Math.cos(r),s=Math.sin(r);return[cx+(x-cx)*c-(y-cy)*s,cy+(x-cx)*s+(y-cy)*c];}
export const circle=(x,y,r)=>`M${x} ${y-r} C${x+r*.552285} ${y-r} ${x+r} ${y-r*.552285} ${x+r} ${y} C${x+r} ${y+r*.552285} ${x+r*.552285} ${y+r} ${x} ${y+r} C${x-r*.552285} ${y+r} ${x-r} ${y+r*.552285} ${x-r} ${y} C${x-r} ${y-r*.552285} ${x-r*.552285} ${y-r} ${x} ${y-r} Z`;
export const rect=(x,y,w,h,r)=>{r=Math.min(r,w/2,h/2);return `M${x+r} ${y} L${x+w-r} ${y} C${x+w-r*.448} ${y} ${x+w} ${y+r*.448} ${x+w} ${y+r} L${x+w} ${y+h-r} C${x+w} ${y+h-r*.448} ${x+w-r*.448} ${y+h} ${x+w-r} ${y+h} L${x+r} ${y+h} C${x+r*.448} ${y+h} ${x} ${y+h-r*.448} ${x} ${y+h-r} L${x} ${y+r} C${x} ${y+r*.448} ${x+r*.448} ${y} ${x+r} ${y} Z`};
export function polygon(vertices,r){const corners=vertices.map((v,i)=>{const before=vertices[(i+vertices.length-1)%vertices.length],after=vertices[(i+1)%vertices.length];const radius=Array.isArray(r)?r[i%r.length]:r;const step=p=>{const d=Math.hypot(p[0]-v[0],p[1]-v[1]);if(d<1e-9)return [...v];const a=Math.min(radius,d*.35);return[v[0]+(p[0]-v[0])*a/d,v[1]+(p[1]-v[1])*a/d];};return {v,a:step(before),b:step(after)};});return corners.map(({v,a,b},i)=>`${i?'L':'M'}${a} C${mix(a[0],v[0],2/3)} ${mix(a[1],v[1],2/3)} ${mix(b[0],v[0],2/3)} ${mix(b[1],v[1],2/3)} ${b}`).join(' ')+' Z';}
export function capsule(a,b,width){const len=Math.hypot(b[0]-a[0],b[1]-a[1]);return svgpath(rect(-width/2,-width/2,len+width,width,width/2)).rotate(Math.atan2(b[1]-a[1],b[0]-a[0])*180/Math.PI).translate(...a).toString();}
// Reverse a closed contour without reflecting its geometry. One final Close
// and a retraced zero-width seam preserve transparent holes in this exporter.
export function reverseContour(d){const segments=svgpath(d).abs().unarc().unshort().segments;let first,last;const edges=[];for(const [cmd,...v]of segments){if(cmd==='M'){first=v;last=v;}else if(cmd==='L'||cmd==='C'){edges.push({cmd,start:last,v});last=v.slice(-2);}else if(cmd==='Z'){if(last[0]!==first[0]||last[1]!==first[1])edges.push({cmd:'L',start:last,v:first});last=first;}else throw Error('Unsupported hole command '+cmd);}return 'M'+first+' '+edges.reverse().map(e=>e.cmd==='L'?'L'+e.start:'C'+[...e.v.slice(2,4),...e.v.slice(0,2),...e.start]).join(' ')+' Z';}
function winding(d){const p=svgpath(d).abs().unarc().unshort().segments.filter(s=>s[0]!=='Z').map(s=>s.slice(-2));return Math.sign(p.reduce((sum,a,i)=>{const b=p[(i+1)%p.length];return sum+a[0]*b[1]-b[0]*a[1];},0))||1;}
export const hole=(outer,inner)=>outer.replace(/Z\s*$/,'')+' L'+svgpath(outer).abs().segments[0].slice(1)+' '+(winding(outer)===winding(inner)?reverseContour(inner):inner).replace(/^M/,'L');
export const fill=(name,d,style='solid',extra={})=>({name,d,style,...extra});
export const stroke=(name,d,extra={})=>({name,d,style:'outline',type:'stroke',width:2,...extra});
export const line=(name,a,b,extra={})=>[stroke(name,`M${a} L${b}`,extra),fill(name+'-solid',capsule(a,b,2.6),'solid',extra)];
export const rigid=(fn)=>Object.assign((x,y,t,s,r)=>{const {dx=0,dy=0,angle=0,pivot=[16,16]}=fn(t,s,r);const p=rotate(x,y,angle,...pivot);return[p[0]+dx,p[1]+dy];},{transform:fn});
const gesture=(t,s,light,strong)=>phrase(s?strong:light,t);

const pulse=(t,s,delay=0)=>gesture(t-delay,s,[[0,0],[.16,1],[.21,1],[.48,0]],[[0,0],[.24,1],[.32,1],[.76,0]]);
export const bothFill=(name,d,extra={})=>[fill(name,d,'outline',extra),fill(name+'-solid',d,'solid',extra)];
export const tube=(name,d,extra={})=>[stroke(name,d,extra),{name:name+'-solid',d,style:'solid',type:'stroke',width:2.6,...extra}];
export const annulus=(x,y,r,w)=>hole(circle(x,y,r+w/2),circle(x,y,r-w/2));
export const framed=(name,d,solid=d,extra={})=>[fill(name+'-interior',d,'tint',extra),stroke(name+'-frame',d,extra),fill(name+'-solid',solid,'solid',extra)];
// Explicit cubic segments keep arc topology fixed while its radius changes.
export const arc=(cx,cy,r,a,b)=>{const n=Math.max(1,Math.ceil(Math.abs(b-a)/90-1e-8)),angle=d=>d*Math.PI/180,p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let start=angle(a),d='M'+p(start);for(let i=1;i<=n;i++){const end=angle(a+(b-a)*i/n),k=4/3*Math.tan((end-start)/4),u=p(start),v=p(end);d+=' C'+[u[0]-k*r*Math.sin(start),u[1]+k*r*Math.cos(start),v[0]+k*r*Math.sin(end),v[1]-k*r*Math.cos(end),...v];start=end;}return d;};
export const holes=(outer,...cuts)=>cuts.reduce((d,c)=>hole(d,c),outer);
const frame=rect(5.5,5.5,21,21,2.7);

