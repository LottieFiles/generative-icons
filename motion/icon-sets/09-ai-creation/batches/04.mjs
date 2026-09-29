// Claude Code design; Codex integration and verification.
// authoring-04.mjs — storyboard, motion-generate, code-generate, code-explain, code-review. Unrendered draft.
import {make,rigid,tube,framed,bothFill,circle,rect,polygon,hole,track,motion,mix} from '../../library/ai-authoring.mjs';
import {flow} from '../../library/direction.mjs';
// Strong on flow, Light untouched on the existing motion track.
const M=(t,s,l,st)=>s?flow(st,t):motion(t,s,l,st);
const S=(a,b)=>`M${a[0]} ${a[1]} L${b[0]} ${b[1]}`;
// Three-point line with a rounded knee; topology M L C L at every frame.
const bend=(a,v,b,r=1.2)=>{const e=p=>{const d=Math.hypot(p[0]-v[0],p[1]-v[1]),k=Math.min(r,d*.4)/(d||1);return[v[0]+(p[0]-v[0])*k,v[1]+(p[1]-v[1])*k];},x=e(a),y=e(b),c=p=>[mix(p[0],v[0],2/3),mix(p[1],v[1],2/3)];return`M${a} L${x} C${c(x)} ${c(y)} ${y} L${b}`;};
// Polyline trimmed by length fraction; unreached segments collapse onto the pen point.
const trim=(pts,p)=>{const len=pts.slice(1).map((q,i)=>Math.hypot(q[0]-pts[i][0],q[1]-pts[i][1]));let left=p*len.reduce((a,b)=>a+b,0),d='M'+pts[0];pts.slice(1).forEach((q,i)=>{const f=Math.max(0,Math.min(1,left/len[i]));left-=len[i];d+=' L'+[mix(pts[i][0],q[0],f),mix(pts[i][1],q[1],f)];});return d;};
const cut=(c,u)=>{const q=(a,b)=>[mix(a[0],b[0],u),mix(a[1],b[1],u)],a=q(c[0],c[1]),b=q(c[1],c[2]),e=q(c[2],c[3]),f=q(a,b),g=q(b,e);return`M${c[0]} C${a} ${f} ${q(f,g)}`;};
// Reset phasing: overlapping parts never fade while another overlapping part is partial.
const phase=(icon,key,keys)=>{for(const p of icon.parts.filter(p=>p.name.startsWith(icon.name+'-'+key))){const v=p.visibility;p.visibility=(t,s,r)=>r?track(keys,t):v(t,s,r);}return icon;};

function storyboard(){
 const panel=(n,x,y)=>framed(n,rect(x,y,11.25,11.25,2.5),hole(rect(x-1.3,y-1.3,13.85,13.85,3.8),rect(x+1.3,y+1.3,8.65,8.65,1.2)));
 const mark=(n,pts,l,s0)=>tube(n,(t,s)=>trim(pts,M(t,s,[[l,0],[l+.18,1,'out']],[[s0,0],[s0+.08,1]])),
  {visibility:(t,s)=>M(t,s,[[l,0],[l+.03,1]],[[s0,0],[s0+.02,.45],[s0+.12,.45],[s0+.2,1]])});
 return make('storyboard','Storyboard','Panel one is drawn; panels two to four are sketched in order. Strong sketches each mark faintly, dwells, then inks it.',[
  ...panel('p1',3,3),...panel('p2',17.75,3),...panel('p3',3,17.75),...panel('p4',17.75,17.75),
  ...tube('m1','M6.5 10.75 L8.625 8 L10.75 10.75'),
  ...mark('m2',[[21.25,8.625],[25.5,8.625]],.02,.04),
  ...mark('m3',[[6.5,25.5],[10.75,21.25]],.22,.26),
  ...mark('m4',[[21.25,21.25],[21.25,25.5],[25.5,25.5]],.42,.48)],.65,.75,['m2','m3','m4']);
}

function motionGenerate(){
 const C=[[7,23],[8,11],[16,6],[25,10]],at=u=>{const v=1-u;return[0,1].map(i=>v*v*v*C[0][i]+3*v*v*u*C[1][i]+3*v*u*u*C[2][i]+u*u*u*C[3][i]);};
 const U=(t,s)=>M(t,s,[[0,0],[.55,1]],[[0,0],[.1,0],[.38,1]]);
 const ball=rigid((t,s)=>{const p=at(U(t,s)),b=s?flow([[0,0],[.1,1],[.24,0]],t):0,o=s?flow([[0,0],[.34,0],[.44,1],[.54,-.35],[.62,.1],[.68,0]],t):0;return{dx:p[0]-7-.15*b+.73*o,dy:p[1]-23+1.2*b+.32*o};});
 const icon=make('motion-generate','Motion generate','A ball runs a path drawn 15% ahead of it; opaque in-betweens appear once the ball has cleared them. Strong pulls back first, overshoots the exit tangent by 0.8, then settles.',[
  ...tube('path',(t,s)=>cut(C,Math.min(1,U(t,s)+.15))),
  ...[0,.33,.66].flatMap((u,i)=>bothFill('g'+i,circle(...at(u),2.3),{visibility:(t,s)=>U(t,s)>=[.2,.57,.88][i]?1:0})),
  ...bothFill('ball',circle(7,23,3),{pose:ball})],.6,.75,['path','g0','g1','g2','ball']);
 phase(icon,'path',[[0,1],[.06,1],[.12,0,'out'],[.15,0],[.25,1,'out']]);
 phase(icon,'ball',[[0,1],[.05,0,'out'],[.25,0],[.28,1,'out']]);
 phase(icon,'g',[[0,1],[.05,0,'out']]);
 return Object.assign(icon,{duotoneFill:false});
}

function codeGenerate(){
 const count=(t,times)=>times.filter(x=>t>=x).length;
 const n1=(t,s)=>count(t,s?[.14,.16,.18,.26,.28]:[.04,.10,.16,.22,.28]),n2=(t,s)=>count(t,s?[.4,.42,.47,.49]:[.40,.45,.50,.55]);
 const cursor=(t,s)=>{const w=s?.32:.30,two=t>=w+.04,n=two?n2(t,s):n1(t,s),x=11.5+(two?1:1.1)*n+(n?3.5:0),y=two?19:13;return S([x,y-2],[x,y+2]);};
 const open=(t,s)=>s?flow([[0,0],[.035,-.2],[.14,1.1],[.25,1],[.64,1],[.76,-.1],[.86,0]],t):0;
 return Object.assign(make('code-generate','Code generate','Two lines are typed between brackets; the cursor blinks off to wrap, on at the next line. Strong opens the brackets, types with word pauses, blinks once, closes them.',[
  ...tube('lb',bend([10,9],[4,16],[10,23]),{pose:rigid((t,s)=>({dx:-.8*open(t,s)}))}),
  ...tube('rb',bend([22,9],[28,16],[22,23]),{pose:rigid((t,s)=>({dx:.8*open(t,s)}))}),
  ...tube('l1',(t,s)=>S([11.5,13],[11.5+1.1*n1(t,s),13]),{visibility:(t,s)=>n1(t,s)?1:0}),
  ...tube('l2',(t,s)=>S([11.5,19],[11.5+n2(t,s),19]),{visibility:(t,s)=>n2(t,s)?1:0}),
  ...tube('cursor',cursor,{visibility:(t,s)=>{const w=s?.32:.30;return(t>=w&&t<w+.08)||(s&&t>=.56&&t<.62)?0:1;}})],.6,.9,['l1','l2','cursor']),{duotoneFill:false});
}

function codeExplain(){
 const grow=(t,s)=>M(t,s,[[0,0],[.30,1,'out']],[[0,0],[.1,0],[.4,1.08],[.5,.96],[.58,1]]);
 const bubble=o=>(t,s)=>{const g=grow(t,s),y0=mix(18,15.5,g),y1=mix(25,28,g);
  if(!o)return polygon([[4,y0],[10,y0],[12.5,y0-2.25],[15,y0],[28,y0],[28,y1],[4,y1]],[2.5,.6,.9,.6,2.5,2.5,2.5]);
  return hole(polygon([[2.7,y0-1.3],[9.5,y0-1.3],[12.5,y0-4],[15.5,y0-1.3],[29.3,y0-1.3],[29.3,y1+1.3],[2.7,y1+1.3]],[3.8,.6,1.2,.6,3.8,3.8,3.8]),rect(5.3,y0+1.3,21.4,y1-y0-2.6,1.2));};
 const pinch=(t,s)=>s?flow([[0,0],[.03,-.15],[.12,1.05],[.22,1],[.80,1],[.92,-.08],[1,0]],t):0;
 const y1=(t,s)=>M(t,s,[[.10,21.5],[.32,19.5,'out']],[[0,21.5],[.42,21.5],[.56,19.5]]);
 const e1=(t,s)=>M(t,s,[[.14,11.5],[.40,23,'out']],[[0,11.5],[.46,11.5],[.62,23]]);
 const e2=(t,s)=>M(t,s,[[.40,7.5],[.58,17,'out']],[[0,7.5],[.64,7.5],[.8,17]]);
 return make('code-explain','Code explain','A callout unfolds under a code line, then two explanation lines are written. Strong pinches the brackets to select the code, unfolds, annotates, then releases.',[
  ...tube('lb',bend([8,4],[4.5,8],[8,12]),{pose:rigid((t,s)=>({dx:.7*pinch(t,s)}))}),
  ...tube('rb',bend([24,4],[27.5,8],[24,12]),{pose:rigid((t,s)=>({dx:-.7*pinch(t,s)}))}),
  ...tube('code',S([11.5,8],[20.5,8])),
  ...framed('bubble',bubble(0),bubble(1)),
  ...tube('l1',(t,s)=>S([7.5,y1(t,s)],[e1(t,s),y1(t,s)])),
  ...tube('l2',(t,s)=>S([7.5,24],[e2(t,s),24]),{visibility:(t,s)=>t>=(s?.64:.40)?1:0})],.6,1,['bubble','l1','l2']);
}

function codeReview(){
 const p=(t,s)=>M(t,s,[[.05,0],[.40,1,'out']],[[0,0],[.16,-.06],[.5,1.06],[.6,.97],[.68,1]]);
 const o=(t,s)=>s?flow([[0,0],[.44,0],[.54,1],[.64,-.3],[.72,0]],t):0;
 const pinch=(t,s)=>s?flow([[0,0],[.1,1,'hit'],[.3,0]],t):0;
 const mark=(t,s)=>{const g=p(t,s),k=o(t,s),L=(a,b)=>[mix(a[0],b[0],g),mix(a[1],b[1],g)];return bend(L([17.75,9],[12,17]),L([16,16],[15.5,20.5]),L([14.25,23],[20.5+.2*k,12.5-.2*k]));};
 return Object.assign(make('code-review','Code review','The slash of a code sign bends into a check. Strong grips with the brackets, releases, morphs with a bounded tip overshoot, holds.',[
  ...tube('lb',bend([10,8],[4,16],[10,24]),{pose:rigid((t,s)=>({dx:.6*pinch(t,s)}))}),
  ...tube('rb',bend([22,8],[28,16],[22,24]),{pose:rigid((t,s)=>({dx:-.6*pinch(t,s)}))}),
  ...tube('mark',mark)],.5,.95,['mark']),{duotoneFill:false});
}

export const icons=[storyboard(),motionGenerate(),codeGenerate(),codeExplain(),codeReview()];
