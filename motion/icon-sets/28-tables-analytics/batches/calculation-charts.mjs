// calculation-charts.mjs — Tables & analytics batch 1, revision 02
import {make,tube,framed,stroke,fill,annulus,circle,arc,mix,F,rigid,posed,shown,geometricRearm} from './helpers.mjs';

const seg=(a,b,p)=>`M${a[0]} ${a[1]} L${mix(a[0],b[0],p)} ${mix(a[1],b[1],p)}`;
const policy=(icon,duotoneFill)=>Object.assign(icon,{duotoneFill});

// ---------- FORMULA (state/output, duotone false) ----------
// f fixed: stem x=8, hook ends x=11.5 (Solid paint to 12.8). Caret rest 15.5 (paint 14.2..16.8).
// Caret final 28.2 (paint 26.9..29.5), Strong overshoot 28.6 (paint 29.9). x strokes 18.8..23.8
// (paint 17.5..25.1). x is only visible while caret >= 28.2, so painted gap >= 1.8 at all times.
const caretX=F([[0,15.5],[.06,15.5],[.34,28.2,'smooth']],
               [[0,15.5],[.06,15.2],[.14,15.2,'hold'],[.26,28.6],[.4,28.2]]);
const xa=F([[0,0],[.36,0],[.54,1,'smooth']],[[0,0],[.28,0,'hold'],[.36,1]]);
const xb=F([[0,0],[.52,0],[.7,1,'smooth']],[[0,0],[.34,0,'hold'],[.42,1]]);
const present=p=>(t,s)=>p(t,s)>0?1:0; // step: no partial opacity, hides only the zero-length cap dot
const formula=policy(geometricRearm(make('formula','Formula','The caret advances from f to the end of the line, then the x is typed into the cleared space behind it. Light: quiet glide, then two unhurried strokes. Strong: brief hold, fast launch with one bounded settle, then two fast strokes.',[
 ...tube('f-stem','M8 22.5 L8 10.5 C8 9 9.2 8.5 11.5 8.5'),
 ...tube('f-bar','M5.5 14.5 L11 14.5'),
 ...posed(tube('caret','M0 9.5 L0 22.5'),rigid((t,s)=>({dx:caretX(t,s)}))),
 ...shown(tube('x-a',(t,s)=>seg([18.8,12],[23.8,21],xa(t,s))),present(xa)),
 ...shown(tube('x-b',(t,s)=>seg([23.8,12],[18.8,21],xb(t,s))),present(xb)),
],.72,.5)),false);

// ---------- SUM (unchanged) ----------
const apex=F([[0,18],[.25,15.2,'smooth'],[.55,18,'smooth']],
             [[0,18],[.12,20],[.28,12.2,'hit'],[.42,18.9],[.56,18]]);
const sum=policy(make('sum','Sum','The sigma gathers: its apex pinches inward and returns. Light: shallow quiet pinch. Strong: arms open wider first, then snap deep inward with one bounded rebound.',[
 ...tube('sigma',(t,s)=>`M23 8 L9 8 L${apex(t,s)} 16 L9 24 L23 24`),
],.55,.56),false);

// ---------- PERCENTAGE (gesture, duotone false: rings are typographic counters) ----------
const ring=(name,x,y)=>[stroke(name,circle(x,y,3.5)),fill(name+'-solid',annulus(x,y,3.5,2.6),'solid')];
const slashA=F([[0,0],[.25,10,'smooth'],[.55,0,'smooth']],
               [[0,0],[.12,-8],[.26,30,'hit'],[.4,-3],[.55,0]]);
const percentage=policy(make('percentage','Percentage','The percent slash swings like a lever about the centre and returns; the two ring counters stay fixed and open. Light: small quiet swing. Strong: short wind-back, wide decisive swing, one damped return.',[
 ...ring('ring-a',10,10),
 ...ring('ring-b',22,22),
 ...posed(tube('slash','M22 7 L10 25'),rigid((t,s)=>({angle:slashA(t,s),pivot:[16,16]}))),
],.55,.56),false);

// ---------- LINE CHART (unchanged) ----------
const rise=F([[0,0],[.5,1,'smooth']],[[0,0],[.24,1]]);
const targets=[[10,21],[15,14],[19,18],[25,9]];
const trendD=(t,s)=>targets.map(([x,y],i)=>`${i?'L':'M'}${x} ${mix(21,y,rise(t-i*(s?.035:.07),s))}`).join(' ');
const lineChart=policy(geometricRearm(make('line-chart','Line chart','The time-series trend rises from the baseline into its values, left to right, and holds; axes stay fixed. Light: gentle staggered rise. Strong: fast tight-stagger snap to the same values, no overshoot (data).',[
 ...tube('axes','M6 5 L6 26 L27 26'),
 ...tube('trend',trendD),
],.75,.36)),false);

// ---------- PIE CHART (gesture, duotone true) ----------
// sector(): region between angles a..b with each radial edge inset e from its axis,
// exposed corners rounded by k; a reflex sector keeps its concave centre as a true vertex.
function sector(cx,cy,r,a,b,e,k){
 const D=Math.PI/180,S=p=>p[0]+' '+p[1],u=g=>[Math.cos(g*D),Math.sin(g*D)];
 const ua=u(a),ub=u(b),na=[-ua[1],ua[0]],nb=[ub[1],-ub[0]],s=Math.sqrt(r*r-e*e),m=e/Math.sin((b-a)*D);
 const A=[cx+m*(ua[0]+ub[0]),cy+m*(ua[1]+ub[1])];
 const P1=[cx+e*na[0]+s*ua[0],cy+e*na[1]+s*ua[1]],P2=[cx+e*nb[0]+s*ub[0],cy+e*nb[1]+s*ub[1]];
 const off=Math.asin(e/r)/D,t1=a+off,t2=b-off,dg=k/r/D,pt=g=>[cx+r*Math.cos(g*D),cy+r*Math.sin(g*D)];
 const toward=(p,q,d)=>{const l=Math.hypot(q[0]-p[0],q[1]-p[1]);return[p[0]+(q[0]-p[0])*d/l,p[1]+(q[1]-p[1])*d/l];};
 const cp=(p,v)=>`${mix(p[0],v[0],2/3)} ${mix(p[1],v[1],2/3)}`;
 const corner=(v,p,q)=>`L${S(p)} C${cp(p,v)} ${cp(q,v)} ${S(q)}`;
 const a1=toward(A,P1,k),a2=toward(A,P2,k),convex=(b-a)<180;
 return (convex?`M${S(a2)} C${cp(a2,A)} ${cp(a1,A)} ${S(a1)}`:`M${S(A)}`)
  +` ${corner(P1,toward(P1,A,k),pt(t1+dg))} ${arc(cx,cy,r,t1+dg,t2-dg).replace(/^M[^C]*/,'')}`
  +` ${corner(P2,pt(t2-dg),toward(P2,A,k))} Z`;
}
// Outline: r10, inset 1.9 => centreline separation 3.8 => painted gap 1.3 at Bold 2.5.
// Solid: r11.3, inset .65 => painted gap 1.3, rounding 2.4 matches the stroked round join.
const bodyO=sector(15,17,10,0,270,1.9,1.2),wedgeO=sector(15,17,10,270,360,1.9,1.2);
const bodyS=sector(15,17,11.3,0,270,.65,2.4),wedgeS=sector(15,17,11.3,270,360,.65,2.4);
const pop=F([[0,0],[.3,1.4,'smooth'],[.65,0,'smooth']],
            [[0,0],[.05,0,'hold'],[.19,4.4,'hit'],[.36,.9],[.52,0]]);
const pieChart=policy(make('pie-chart','Pie chart','The quarter wedge pops out of the pie along its bisector and settles back into its opening; the body stays fixed. Light: small quiet lift. Strong: decisive pop far out, one damped settle.',[
 ...framed('body',bodyO,bodyS),
 ...posed(framed('wedge',wedgeO,wedgeS),rigid((t,s)=>{const o=pop(t,s)*.7071;return {dx:o,dy:-o};})),
],.65,.52),true);

export const icons=[formula,sum,percentage,lineChart,pieChart];
