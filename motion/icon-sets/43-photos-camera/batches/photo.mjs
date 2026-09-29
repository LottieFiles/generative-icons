// Photos & camera shared geometry helpers: photo cards, landscape glyphs and fixed-topology arcs.
import {rect,holes,framed,polygon,circle,bothFill,stroke,fill} from './helpers.mjs';
export const pt=(a,r,c=[16,16])=>[c[0]+r*Math.cos(a*Math.PI/180),c[1]+r*Math.sin(a*Math.PI/180)];
// Fixed-topology circular arc: always n cubic segments.
export const arcN=(cx,cy,r,a,b,n=4)=>{const A=d=>d*Math.PI/180,p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let s=A(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=A(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*r*Math.sin(s),u[1]+k*r*Math.cos(s),v[0]+k*r*Math.sin(e),v[1]-k*r*Math.cos(e),...v];s=e;}return d;};
export const L=d=>d.replace(/^M/,'L');
export const clamp01=x=>Math.max(0,Math.min(1,x));
// A photo card: Outline centreline rect, Duotone interior, Solid ring (outer +1.3, inner -1.3) with a real counter.
export const ringD=(x,y,w,h,r)=>holes(rect(x-1.3,y-1.3,w+2.6,h+2.6,r+1.3),rect(x+1.3,y+1.3,w-2.6,h-2.6,Math.max(.5,r-1.3)));
export const cardParts=(name,x,y,w,h,r=2.2,extra={})=>framed(name,rect(x,y,w,h,r),ringD(x,y,w,h,r),extra);
// A closed mountain glyph: stroked in Outline, filled in Solid (as the library's Image icon does).
export const hillsParts=(name,pts,r=.6,extra={})=>{const d=typeof pts==='function'?(t,s,rv)=>polygon(pts(t,s,rv),r):polygon(pts,r);return [stroke(name,d,extra),fill(name+'-solid',d,'solid',extra)];};
export const sunParts=(name,x,y,rad,extra={})=>bothFill(name,circle(x,y,rad),extra);
// The visible L of a card sitting behind a front card (x0,y0,w,h) and offset up-right by o.
// Its ends sit on the front card's top and right edges, so nothing shows through the front.
export const backCardD=(x0,y0,w,h,o)=>{const r=Math.min(2,Math.max(.01,o)),k=.448;return `M${x0+o} ${y0} L${x0+o} ${y0-o+r} C${x0+o} ${y0-o+r*k} ${x0+o+r*k} ${y0-o} ${x0+o+r} ${y0-o} L${x0+w+o-r} ${y0-o} C${x0+w+o-r*k} ${y0-o} ${x0+w+o} ${y0-o+r*k} ${x0+w+o} ${y0-o+r} L${x0+w+o} ${y0+h-o-r} C${x0+w+o} ${y0+h-o-r*k} ${x0+w+o-r*k} ${y0+h-o} ${x0+w+o-r} ${y0+h-o} L${x0+w} ${y0+h-o}`;};
// Solid L region of that back card, overlapping the front card's Solid rim by 0.3 so the stack is one surface.
export const backCardSolidD=(x0,y0,w,h,o)=>polygon([[x0+o-1.3,y0-o-1.3],[x0+w+o+1.3,y0-o-1.3],[x0+w+o+1.3,y0+h-o+1.3],[x0+w+1,y0+h-o+1.3],[x0+w+1,y0-1],[x0+o-1.3,y0-1]],[3.3,3.3,3.3,0,0,0]);
