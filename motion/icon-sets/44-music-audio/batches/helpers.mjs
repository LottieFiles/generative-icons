// Shared assembly for the Music & audio set (assembly adapted from 39-food-drink helpers). No artwork.
export * from '../../library/ai-authoring.mjs';
import {make,timePair,drive,stroke,fill,capsule} from '../../library/ai-authoring.mjs';
import svgpath from 'svgpath';
export function toggle(name,label,intent,parts,light,strong,initialState,finalState,reverseLight=light,reverseStrong=strong){
 return {...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:finalState,reverseLabel:initialState,reverseTimings:timePair(reverseLight,reverseStrong)};
}
// Forward/reverse driver pair: the reverse argument selects the authored reverse tracks.
export const G=(fl,fs,rl,rs)=>(t,s,r)=>r?drive(t,s,rl,rs):drive(t,s,fl,fs);
export const L=d=>d.replace(/^M/,'L');
export const clamp01=x=>Math.max(0,Math.min(1,x));
export const pt=(a,r,c=[16,16])=>[c[0]+r*Math.cos(a*Math.PI/180),c[1]+r*Math.sin(a*Math.PI/180)];
// Fixed-topology circular arc: always n cubic segments, so an animated span keeps its commands.
export const arcN=(cx,cy,r,a,b,n=4)=>{const A=d=>d*Math.PI/180,p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let s=A(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=A(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*r*Math.sin(s),u[1]+k*r*Math.cos(s),v[0]+k*r*Math.sin(e),v[1]-k*r*Math.cos(e),...v];s=e;}return d;};
// Elliptical arc with the same fixed topology.
export const earcN=(cx,cy,rx,ry,a,b,n=4)=>{const A=d=>d*Math.PI/180,p=t=>[cx+rx*Math.cos(t),cy+ry*Math.sin(t)];let s=A(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=A(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*rx*Math.sin(s),u[1]+k*ry*Math.cos(s),v[0]+k*rx*Math.sin(e),v[1]-k*ry*Math.cos(e),...v];s=e;}return d;};
export const ellipse=(cx,cy,rx,ry)=>earcN(cx,cy,rx,ry,0,360,4)+' Z';
// Closed envelope around a circular arc with round caps and fixed topology (n outer/inner segments, 2 per cap).
export function arcTubeN(cx,cy,r,a,b,w,n=4){
 const h=w/2,sign=Math.sign(b-a)||1,start=pt(a,r,[cx,cy]),end=pt(b,r,[cx,cy]);
 return arcN(cx,cy,r+h,a,b,n)+' '+L(arcN(end[0],end[1],h,b,b+sign*180,2))+' '+L(arcN(cx,cy,r-h,b,a,n))+' '+L(arcN(start[0],start[1],h,a+sign*180,a+sign*360,2))+' Z';
}
// Deterministic counter for animated arc tubes: the inner contour is always reversed and
// always closed with an explicit edge, so floating error at the seam never changes topology
// and a zero-area (collapsed) counter never flips its point order between frames.
// Inner contours must be drawn with the same winding as the outer (rect, circle, arcTubeN all do).
function reverseFixed(d){
 const segments=svgpath(d).abs().unarc().unshort().segments;let first,last;const edges=[];
 for(const [cmd,...v] of segments){if(cmd==='M'){first=v;last=v;}else if(cmd==='L'||cmd==='C'){edges.push({cmd,start:last,v});last=v.slice(-2);}else if(cmd==='Z'){edges.push({cmd:'L',start:last,v:first});last=first;}else throw Error('Unsupported counter command '+cmd);}
 return 'M'+first+' '+edges.reverse().map(e=>e.cmd==='L'?'L'+e.start:'C'+[...e.v.slice(2,4),...e.v.slice(0,2),...e.start]).join(' ')+' Z';
}
export const cutRev=(outer,inner)=>outer.replace(/Z\s*$/,'')+' L'+outer.slice(1).trim().split(/[\s,]+/).slice(0,2).join(' ')+' '+reverseFixed(inner).replace(/^M/,'L');
export const cuts=(outer,...inners)=>inners.reduce((d,i)=>cutRev(d,i),outer);
// Dynamic line: fn(t,s,r) returns [a,b]; outline stroke plus a solid capsule.
export const dline=(name,fn,extra={})=>[stroke(name,(t,s,r)=>{const [a,b]=fn(t,s,r);return `M${a} L${b}`;},extra),fill(name+'-solid',(t,s,r)=>capsule(...fn(t,s,r),2.6),'solid',extra)];
