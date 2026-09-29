import {icon,progress,kick,F,framed,box,tube,fill,bothFill,stroke,circle,rect,node,posed,shown,rigid,ringRect,polygon} from './helpers.mjs';
// A true normal-offset ribbon, including rounded end caps. Each boundary uses
// fixed cubic topology; the Solid width includes the outline's painted envelope.
const curveControls=[[7,24],[9,8],[23,24],[25,8]];
const centerSamples=Array.from({length:25},(_,j)=>{
 const u=j/24,v=1-u,[a,b,c,d]=curveControls;
 const p=[0,1].map(k=>v*v*v*a[k]+3*v*v*u*b[k]+3*v*u*u*c[k]+u*u*u*d[k]);
 const tangent=[0,1].map(k=>3*v*v*(b[k]-a[k])+6*v*u*(c[k]-b[k])+3*u*u*(d[k]-c[k]));
 const length=Math.hypot(...tangent),t=tangent.map(x=>x/length);
 return {p,t,n:[-t[1],t[0]]};
});
const smoothBoundary=points=>points.slice(1).map((b,j)=>{
 const a=points[j],prev=points[Math.max(0,j-1)],next=points[Math.min(points.length-1,j+2)];
 return ' C'+[a[0]+(b[0]-prev[0])/6,a[1]+(b[1]-prev[1])/6,b[0]-(next[0]-a[0])/6,b[1]-(next[1]-a[1])/6,...b].join(' ');
}).join('');
const ribbon=w=>{
 const top=centerSamples.map(({p,n})=>p.map((x,k)=>x-w*n[k]));
 const bottom=centerSamples.map(({p,n})=>p.map((x,k)=>x+w*n[k])).reverse();
 const cap=(sample,sign)=>{const {p,t,n}=sample,k=.552285;
  const point=(along,across)=>p.map((x,j)=>x+w*(along*t[j]+across*n[j]));
  return ' C'+[...point(sign*k,-sign),...point(sign,-sign*k),...point(sign,0)].join(' ')+' C'+[...point(sign,sign*k),...point(sign*k,sign),...point(0,sign)].join(' ');
 };
 return 'M'+top[0]+smoothBoundary(top)+cap(centerSamples.at(-1),1)+smoothBoundary(bottom)+cap(centerSamples[0],-1)+' Z';
};
const outline=icon('outline-stroke','Outline stroke','Convert a curved stroke to an editable closed ribbon. Light: the two boundaries separate quietly. Strong: the ribbon opens 1.4 units farther before settling into the same outlined path. Reverse closes the ribbon back onto the source stroke.',[
 ...shown(tube('source','M7 24 C9 8 23 24 25 8'),(t,s,r)=>1-progress(t,s,r)),
 ...shown(framed('ribbon',(t,s,r)=>ribbon(1.8+1.4*kick(t,s)),(t,s,r)=>ribbon(3.1+1.4*kick(t,s))),(t,s,r)=>progress(t,s,r)),
 ...[ [7,24],[25,8] ].flatMap(([x,y],j)=>bothFill('endpoint-'+j,node(x,y,1.35)))
],{state:['Live stroke','Outlined stroke']});
const offsetDrive=F([[0,0],[.26,.65],[.8,0]],[[0,0],[.1,-.25],[.3,1.55],[.48,.5],[.72,0]]);
const offset=icon('offset-path','Offset path','Adjust an outward offset around a fixed rounded source. Light: the outer contour expands 0.65 units and returns. Strong: it draws in slightly, expands 1.55 units, and settles. The central source never moves.',[
 ...framed('source',rect(12,12,8,8,1.6)),
 ...tube('offset',(t,s)=>{const d=offsetDrive(t,s);return rect(6.5-d,6.5-d,19+2*d,19+2*d,3.3+d);})
],{fixed:['source-frame']});
const joint=(t,s,r)=>progress(t,s,r)*5.5-kick(t,s)*1.1;
const left=(t,s,r)=>`M5.5 23 C5.5 15 7 10  ${10.5+joint(t,s,r)} 10`;
const right=(t,s,r)=>`M${21.5-joint(t,s,r)} 10 C25 10 26.5 15 26.5 23`;
const join=icon('join-paths','Join paths','Connect two open curve endpoints into a continuous arch. Light: the endpoints approach and join. Strong: they pull back 1.1 units, then meet decisively at the shared node. Reverse separates the same joint.',[
 ...tube('left',left),...tube('right',right),
 ...posed(framed('left-node',node(10.5,10,3)),rigid((t,s,r)=>({pivot:[10.5,10],dx:joint(t,s,r)}))),
 ...posed(framed('right-node',node(21.5,10,3)),rigid((t,s,r)=>({pivot:[21.5,10],dx:-joint(t,s,r)})))
],{state:['Open endpoints','Joined path']});
const jag=(t,s,r)=>1-progress(t,s,r)+.28*kick(t,s);
const simpleD=(t,s,r)=>{const j=jag(t,s,r);return `M5.5 24 C7 ${18-3*j} 8 ${14-4*j} 10 ${13-4*j} C12 ${12-4*j} 14 ${7+7*j} 16 ${7+7*j} C18 ${7+7*j} 20 ${12-4*j} 22 ${13-4*j} C24 ${14-4*j} 25 ${18-3*j} 26.5 24`;};
const simplify=icon('simplify-path','Simplify path','Reduce a bumpy five-node path to a smooth arch with three anchors. Light: unnecessary bends relax. Strong: the bends open farther before a rapid smooth reduction. Reverse restores the original editable path.',[
 ...tube('curve',simpleD),
 ...[[5.5,24],[26.5,24]].flatMap(([x,y],j)=>framed('end-'+j,node(x,y,1.4))),
 ...posed(framed('crest',node(16,7,1.4)),rigid((t,s,r)=>({pivot:[16,7],dy:7*jag(t,s,r)}))),
 ...[10,22].flatMap((x,j)=>posed(shown(framed('extra-'+j,node(x,13,1.25)),(t,s,r)=>1-progress(t,s,r)),rigid((t,s,r)=>({pivot:[x,13],dy:-4*jag(t,s,r)}))))
],{state:['Detailed path','Simplified path']});
const dashShift=F([[0,0],[.24,.9],[.72,0]],[[0,0],[.08,-.5],[.28,2.2],[.5,-.4],[.76,0]]);
const dash=icon('stroke-dash','Stroke dash','Adjust the spacing of two rows of dashes above a closed source-stroke swatch. Light: the dashes slide 0.9 units in sequence. Strong: they shift 2.2 units with a short preparation and settle. The complete source stroke stays fixed.',[
 ...[8,15].flatMap((y,row)=>[6.5,15,23.5].flatMap((x,col)=>posed(
  tube('dash-'+row+'-'+col,`M${x-1.5} ${y} L${x+1.5} ${y}`),
  rigid((t,s)=>({pivot:[x,y],dx:(col-1)*dashShift(Math.max(0,t-row*.04),s)}))
 ))),
 ...framed('swatch',rect(6,21,20,4.4,2.2))
],{fixed:['swatch-frame']});
export const icons=[outline,offset,join,simplify,dash];
