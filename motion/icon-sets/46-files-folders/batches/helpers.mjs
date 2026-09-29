// Shared assembly for the Files & folders set (toggle/continuous helpers copied from 39-food-drink). No artwork.
export * from '../../library/ai-authoring.mjs';
import svgpath from 'svgpath';
import {make,timePair,drive,polygon,arc,hole,fill,stroke,circle,capsule} from '../../library/ai-authoring.mjs';
export const TAU=Math.PI*2,cycle=t=>t/2.4,mod=x=>((x%1)+1)%1;
export function toggle(name,label,intent,parts,light,strong,initialState,finalState,reverseLight=light,reverseStrong=strong){
 return {...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:finalState,reverseLabel:initialState,reverseTimings:timePair(reverseLight,reverseStrong)};
}
export const curve=(t,s,l,h)=>drive(t,s,l,h);
// Forward/reverse driver pair: r selects the authored reverse tracks.
export const FR=(lf,sf,lr,sr)=>(t,s,r)=>drive(t,s,r?lr:lf,r?sr:sf);
export const L=d=>d.replace(/^M/,'L');
export const clamp01=x=>Math.max(0,Math.min(1,x));
export const lerp=(a,b,u)=>[a[0]+(b[0]-a[0])*u,a[1]+(b[1]-a[1])*u];
// Fixed-topology circular arc: always n cubic segments, even when the sweep is zero.
export const arcN=(cx,cy,r,a,b,n=2)=>{const A=d=>d*Math.PI/180,p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let s=A(a),d='M'+p(s);for(let i=1;i<=n;i++){const e=A(a+(b-a)*i/n),k=4/3*Math.tan((e-s)/4),u=p(s),v=p(e);d+=' C'+[u[0]-k*r*Math.sin(s),u[1]+k*r*Math.cos(s),v[0]+k*r*Math.sin(e),v[1]-k*r*Math.cos(e),...v];s=e;}return d;};
// Rigid move of a path string (degrees, clockwise positive like rotate()).
export const movePath=(d,angle,cx,cy,dx=0,dy=0)=>svgpath(d).rotate(angle,cx,cy).translate(dx,dy).toString();
// Closed envelope of a U-shaped shackle (two legs and a semicircle) with round feet.
export function uTube(xl,xr,ytop,yfoot,h){const cx=(xl+xr)/2,r=(xr-xl)/2,start=[xl-h,yfoot].join(',');return 'M'+start+' L'+[xl-h,ytop]+' '+L(arcN(cx,ytop,r+h,180,360))+' L'+[xr+h,yfoot]+' '+L(arcN(xr,yfoot,h,0,180))+' L'+[xr-h,ytop]+' '+L(arcN(cx,ytop,r-h,360,180))+' L'+[xl+h,yfoot]+' '+L(arcN(xl,yfoot,h,0,180))+' L'+start+' Z';}

// Open polyline with fixed-radius rounded interior corners. Unlike polygon(), the
// first and last edges are not clamped, so a short or retracing end edge keeps the
// corner geometry identical to the closed shape it is attached to.
export function openRounded(points,radii){
 const n=points.length;let d='M'+points[0];
 for(let i=1;i<n-1;i++){
  const v=points[i],before=points[i-1],after=points[i+1],r=Array.isArray(radii)?radii[(i-1)%radii.length]:radii;
  const step=(p,clampLen)=>{const len=Math.hypot(p[0]-v[0],p[1]-v[1]);if(len<1e-9)return[...v];const a=clampLen?Math.min(r,len*.5):r;return[v[0]+(p[0]-v[0])*a/len,v[1]+(p[1]-v[1])*a/len];};
  const a=step(before,i>1),b=step(after,i<n-2);
  d+=` L${a} C${a[0]+(v[0]-a[0])*2/3} ${a[1]+(v[1]-a[1])*2/3} ${b[0]+(v[0]-b[0])*2/3} ${b[1]+(v[1]-b[1])*2/3} ${b}`;
 }
 return d+' L'+points[n-1];
}

// Single closed envelope around a polyline stroke: round caps, round outer joins,
// mitred inner joins. Used for Solid counters that must not overlap other holes.
export function polylineTube(pts,h){
 const n=pts.length,dirs=[];
 for(let i=0;i<n-1;i++){const dx=pts[i+1][0]-pts[i][0],dy=pts[i+1][1]-pts[i][1],l=Math.hypot(dx,dy);dirs.push(l<1e-6?null:[dx/l,dy/l]);}
 // A collapsed segment continues the previous direction so its joint stays straight.
 for(let i=0;i<dirs.length;i++)if(!dirs[i])dirs[i]=dirs.slice(0,i).reverse().find(Boolean)||dirs.slice(i+1).find(Boolean)||[1,0];
 const nrm=d=>[-d[1],d[0]],off=(p,nv,k)=>[p[0]+nv[0]*h*k,p[1]+nv[1]*h*k];
 const deg=v=>Math.atan2(v[1],v[0])*180/Math.PI;
 const joint=(i,side)=>{ // side +1 = left normal, -1 = right normal
  const p=pts[i],a=nrm(dirs[i-1]),b=nrm(dirs[i]),cross=dirs[i-1][0]*dirs[i][1]-dirs[i-1][1]*dirs[i][0];
  const outer=(cross>0?-1:1)===side;
  if(outer){const p0=off(p,a,side),p1=off(p,b,side),a0=deg([p0[0]-p[0],p0[1]-p[1]]),a1=deg([p1[0]-p[0],p1[1]-p[1]]),delta=((a1-a0+540)%360)-180;return {kind:'arc',d:L(arcN(p[0],p[1],h,a0,a0+delta))};}
  const dot=a[0]*b[0]+a[1]*b[1],m=[(a[0]+b[0])/(1+dot),(a[1]+b[1])/(1+dot)];return {kind:'point',p:[p[0]+m[0]*h*side,p[1]+m[1]*h*side]};
 };
 const segs=[],start=off(pts[0],nrm(dirs[0]),1).join(',');
 // left side forward; the explicit closing edge keeps hole() reversal topology constant
 segs.push('M'+start);
 for(let i=1;i<n-1;i++){const j=joint(i,1);segs.push(j.kind==='arc'?j.d:'L'+j.p);}
 const endL=off(pts[n-1],nrm(dirs[n-2]),1);segs.push('L'+endL);
 const aEnd=deg([endL[0]-pts[n-1][0],endL[1]-pts[n-1][1]]);segs.push(L(arcN(pts[n-1][0],pts[n-1][1],h,aEnd,aEnd-180)));
 for(let i=n-2;i>=1;i--){const j=joint(i,-1);if(j.kind==='arc'){ // reverse arc direction for the return side
   const p=pts[i],a=nrm(dirs[i]),b=nrm(dirs[i-1]),p0=off(p,a,-1),p1=off(p,b,-1),a0=deg([p0[0]-p[0],p0[1]-p[1]]),a1=deg([p1[0]-p[0],p1[1]-p[1]]),delta=((a1-a0+540)%360)-180;segs.push(L(arcN(p[0],p[1],h,a0,a0+delta)));
  }else segs.push('L'+j.p);}
 const startR=off(pts[0],nrm(dirs[0]),-1);segs.push('L'+startR);
 const aStart=deg([startR[0]-pts[0][0],startR[1]-pts[0][1]]);segs.push(L(arcN(pts[0][0],pts[0][1],h,aStart,aStart-180)));
 return segs.join(' ')+' L'+start+' Z';
}

// Closed silhouette of a circular arc arrow: tube body with a round tail cap and a
// filled triangular head at the end angle. One contour, so it can be a Solid counter.
export function arcArrowSolid(cx,cy,r,a,b,h,headLen,headHalf){
 const sign=Math.sign(b-a)||1,rad=d=>d*Math.PI/180,p=(rr,deg)=>[cx+rr*Math.cos(rad(deg)),cy+rr*Math.sin(rad(deg))];
 // head base sits headLen before the end along the arc; apex at the end point
 const baseDeg=b-sign*headLen/r*180/Math.PI,tip=p(r,b),outerBarb=p(r+headHalf,baseDeg),innerBarb=p(r-headHalf,baseDeg);
 const outer=arc(cx,cy,r+h,a,baseDeg),inner=arc(cx,cy,r-h,baseDeg,a),tail=p(r,a),start=outer.slice(1,outer.indexOf(' C'));
 return outer+' L'+outerBarb+' L'+tip+' L'+innerBarb+' '+L(inner)+' '+L(arcN(tail[0],tail[1],h,a+180,sign>0?a+360:a))+' L'+start+' Z';
}
// Outline arrowhead chevron at the end of arc (cx,cy,r) running from a to b.
export function arcHead(cx,cy,r,b,sign,len,splay){
 const rad=d=>d*Math.PI/180,tip=[cx+r*Math.cos(rad(b)),cy+r*Math.sin(rad(b))];
 const tangent=[-Math.sin(rad(b))*sign,Math.cos(rad(b))*sign]; // travel direction
 const back=[-tangent[0],-tangent[1]],rot=(v,deg)=>{const c=Math.cos(rad(deg)),s=Math.sin(rad(deg));return[v[0]*c-v[1]*s,v[0]*s+v[1]*c];};
 const b1=rot(back,splay),b2=rot(back,-splay);
 return `M${tip[0]+b1[0]*len} ${tip[1]+b1[1]*len} L${tip} L${tip[0]+b2[0]*len} ${tip[1]+b2[1]*len}`;
}

// Twelve-vertex plus sign with rounded corners.
export const cross=(cx,cy,arm,w,r=.5)=>polygon([[cx-w,cy-arm],[cx+w,cy-arm],[cx+w,cy-w],[cx+arm,cy-w],[cx+arm,cy+w],[cx+w,cy+w],[cx+w,cy+arm],[cx-w,cy+arm],[cx-w,cy+w],[cx-arm,cy+w],[cx-arm,cy-w],[cx-w,cy-w]],r);
// Five-point star.
export function star(cx,cy,ro,ri,r=.4){const pts=[];for(let i=0;i<10;i++){const a=(-90+i*36)*Math.PI/180,rr=i%2?ri:ro;pts.push([cx+rr*Math.cos(a),cy+rr*Math.sin(a)]);}return polygon(pts,r);}
// Magnifier silhouette: ring outer circle joined to a handle capsule, one contour.
export function lollipop(cx,cy,R,angleDeg,len,h){
 const rad=angleDeg*Math.PI/180,alpha=Math.asin(h/R)*180/Math.PI;
 const end=[cx+(R+len)*Math.cos(rad),cy+(R+len)*Math.sin(rad)],nv=[-Math.sin(rad)*h,Math.cos(rad)*h];
 const a0=angleDeg+alpha,a1=angleDeg-alpha+360;
 return arc(cx,cy,R,a0,a1)+' L'+[end[0]-nv[0],end[1]-nv[1]]+' '+L(arc(end[0],end[1],h,angleDeg-90,angleDeg+90))+' Z';
}

// Family page: 18 x 24 sheet with a 6-unit fold. Content sits inside x 10.5–21.5, y 11.5–24.5.
export const PAGE=polygon([[7,4],[19,4],[25,10],[25,28],[7,28]],[1.8,.9,.9,1.8,1.8]);
export const FOLD='M19 5 L19 9 C19 9.6 19.4 10 20 10 L24 10';
export const FOLD_TRI=polygon([[20,6],[22.5,8.5],[20,8.5]],.3);
// Outline page + fold, tint interior and a Solid fill carrying the fold counter plus
// any content counters (holes must not overlap each other).
export function page(name,cuts=()=>[],extra={}){
 return [fill(name+'-interior',PAGE,'tint',extra),stroke(name+'-page',PAGE,extra),stroke(name+'-fold',FOLD,extra),
  fill(name+'-solid',(t,s,r)=>{let d=hole(PAGE,FOLD_TRI);for(const c of cuts(t,s,r))d=hole(d,c);return d;},'solid',extra)];
}
export const scaleAbout=(x,y,k,cx,cy)=>[cx+(x-cx)*k,cy+(y-cy)*k];
