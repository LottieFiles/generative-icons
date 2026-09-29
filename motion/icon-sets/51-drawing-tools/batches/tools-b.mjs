// Drawing & design tools batch B: rectangle-tool, ellipse-tool, polygon-tool, fill-bucket, eyedropper.
import {make,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,rotate,bothFill,geometricRearm,arcN,reveal,clamp01,pt} from './helpers.mjs';
import svgpath from 'svgpath';
const place=(d,angle,x,y)=>svgpath(d).rotate(angle).translate(x,y).toString();
const K=.5523,ellipse=(x,y,rx,ry)=>`M${x} ${y-ry} C${x+rx*K} ${y-ry} ${x+rx} ${y-ry*K} ${x+rx} ${y} C${x+rx} ${y+ry*K} ${x+rx*K} ${y+ry} ${x} ${y+ry} C${x-rx*K} ${y+ry} ${x-rx} ${y+ry*K} ${x-rx} ${y} C${x-rx} ${y-ry*K} ${x-rx*K} ${y-ry} ${x} ${y-ry} Z`;
const square=(c,k=1)=>rect(c[0]-1.4*k,c[1]-1.4*k,2.8*k,2.8*k,.6*k);

// ---------- RECTANGLE TOOL ----------
// A rectangle is dragged out from its origin handle to the far corner; the drag handle rides that corner.
// Light: a 0.56 s diagonal drag. Strong: the handle is grabbed, the corner shoots out in 0.2 s, overshoots
// by 6% and settles. The rectangle is retained; preview reset rearms.
const RO=[7,8],RW=18,RH=16;
const drag=F([[0,0],[.08,0],[.64,1,'smooth']],[[0,0],[.1,0,'hold'],[.3,1.06],[.42,.98],[.5,1]]);
const grab=F([[0,1],[.6,1]],[[0,1],[.06,1.4],[.14,1]]);
const rectD=(t,s)=>{const p=Math.max(.08,drag(t,s));return rect(RO[0],RO[1],RW*p,RH*p,1.6);};
const rectangleTool=geometricRearm(make('rectangle-tool','Rectangle tool',
 'Light: a rectangle is dragged out from its origin handle to the far corner in 0.56 s while the drag handle rides the corner. Strong: the handle is grabbed, the corner shoots out in 0.2 s, overshoots by 6% and settles. The rectangle is retained; preview reset rearms.',[
 ...shown(framed('box',rectD),(t,s)=>drag(t,s)>.08?1:0),
 ...bothFill('origin',square(RO)),
 ...bothFill('corner',(t,s)=>{const p=drag(t,s);return square([RO[0]+RW*p,RO[1]+RH*p],grab(t,s));})
],.66,.52));
rectangleTool.duotoneFill=true;

// ---------- ELLIPSE TOOL ----------
// An ellipse is swept clockwise from its top anchor by a travelling pen tip; the interior tints once the
// contour closes. Light: one 0.64 s sweep. Strong: the tip swings back 12°, sweeps 372° with a stop and
// settles onto the anchor. The ellipse is retained; preview reset rearms.
const EC=[16,16.5],ERX=10,ERY=7.5,ePt=a=>[EC[0]+ERX*Math.cos(a*Math.PI/180),EC[1]+ERY*Math.sin(a*Math.PI/180)];
const span=F([[0,0],[.08,0],[.72,360,'smooth']],[[0,0],[.08,-12],[.34,372,'hit'],[.44,360]]);
const ellipseArc=reveal(u=>ePt(-90+360*u),6);
const closed=(t,s)=>clamp01((span(t,s)-336)/24);
const tipShown=(t,s)=>{const a=span(t,s);return Math.abs(a)>2&&(s?t<.44:a<359.5)?1:0;};
const ellipseTool=geometricRearm(make('ellipse-tool','Ellipse tool',
 'Light: a pen tip sweeps an ellipse clockwise from its top anchor in 0.64 s and the interior tints as the contour closes. Strong: the tip swings back 12°, sweeps 372° with a stop and settles onto the anchor. The ellipse is retained; preview reset rearms.',[
 ...shown([fill('interior',ellipse(...EC,ERX,ERY),'tint'),fill('interior-solid',ellipse(...EC,ERX,ERY),'solid')],closed),
 ...shown(tube('rim',(t,s)=>ellipseArc(clamp01(span(t,s)/360))),(t,s)=>span(t,s)>2?1:0),
 ...bothFill('anchor',square(ePt(-90))),
 ...shown(bothFill('tip',(t,s)=>circle(...ePt(-90+span(t,s)),1.5)),tipShown)
],.74,.5));
ellipseTool.duotoneFill=true;

// ---------- POLYGON TOOL ----------
// A hexagon is drawn vertex by vertex; an anchor dot appears at each vertex the pen reaches and the
// interior tints when the contour closes. Light: one 0.64 s continuous run. Strong: six quick hops with a
// stop at each vertex, each anchor popping in oversized. The hexagon is retained; preview reset rearms.
const PC=[16,16.5],PR=10,V=Array.from({length:7},(_,k)=>pt(-90+60*k,PR,PC));
const run=F([[0,0],[.08,0],[.72,6,'smooth']],[[0,0],[.08,0,'hold'],[.14,1,'hit'],[.17,1,'hold'],[.23,2,'hit'],[.26,2,'hold'],[.32,3,'hit'],[.35,3,'hold'],[.41,4,'hit'],[.44,4,'hold'],[.5,5,'hit'],[.53,5,'hold'],[.59,6]]);
const perimeter=(t,s)=>{const q=clamp01(run(t,s)/6)*6,k=Math.min(5,Math.floor(q)),f=q-k,tip=[0,1].map(i=>V[k][i]+(V[k+1][i]-V[k][i])*f);return 'M'+V[0]+V.slice(1).map((v,i)=>' L'+(i+1<=q?v:tip)).join('');};
const arrived=k=>(t,s)=>run(t,s)>=k-1e-6?1:0;
const dotR=k=>(t,s)=>{const q=run(t,s);if(!s||k===0)return 1.5;const since=q-k;return 1.5*(since<0?1:1+.5*Math.max(0,1-since*1.4));};
const polygonTool=geometricRearm(make('polygon-tool','Polygon tool',
 'Light: a hexagon is drawn vertex by vertex in one 0.64 s run; an anchor appears at each vertex reached and the interior tints when the contour closes. Strong: six quick hops with a stop at each vertex, each anchor popping in oversized. The hexagon is retained; preview reset rearms.',[
 ...shown([fill('interior',polygon(V.slice(0,6),.001),'tint'),fill('interior-solid',polygon(V.slice(0,6),.001),'solid')],(t,s)=>clamp01((run(t,s)-5.5)/.5)),
 ...shown(tube('outline',perimeter),(t,s)=>run(t,s)>.02?1:0),
 ...Array.from({length:6},(_,k)=>shown(bothFill('anchor-'+k,(t,s)=>circle(...V[k],dotR(k)(t,s))),arrived(k))).flat()
],.74,.62));
polygonTool.duotoneFill=true;

// ---------- FILL BUCKET ----------
// A tilted paint bucket tips further to pour; a stream falls from its lip and a puddle spreads below.
// Light: a gentle 8° tip, a 0.46 s pour and a slow spread. Strong: the bucket rocks back, tips 16° with a
// stop and the puddle surges past full before settling. The puddle is retained; preview reset rearms.
const BC=[20,12],TILT=-32,bodyD=place(rect(-4.5,-4.5,9,9,1.6),TILT,...BC),handleD=place(arcN(0,-4.5,3.2,180,360,2),TILT,...BC);
const tilt=F([[0,TILT],[.08,TILT],[.3,-40,'smooth'],[.5,-40],[.66,TILT,'smooth']],[[0,TILT],[.06,-27],[.2,-48,'hit'],[.32,-44],[.46,-44,'hold'],[.6,TILT]]);
const spread=F([[0,0],[.16,0],[.66,1,'smooth']],[[0,0],[.2,0,'hold'],[.4,1.06],[.5,.98],[.58,1]]);
const pouring=F([[0,0],[.12,0],[.16,1,'out'],[.5,1],[.56,0,'in']],[[0,0],[.18,0,'hold'],[.21,1],[.44,1,'hold'],[.49,0]]);
const lip=(t,s)=>rotate(BC[0]-4.5,BC[1]+4.5,tilt(t,s),...BC);
const bucketPose=rigid((t,s)=>({angle:tilt(t,s)-TILT,pivot:BC}));
const puddleD=(t,s)=>{const k=Math.max(.03,spread(t,s));return ellipse(14.5,25.3,.2+7.8*k,.15+2.45*k);};
const fillBucket=geometricRearm(make('fill-bucket','Fill bucket',
 'Light: the paint bucket tips 8° further, a stream falls from its lip for 0.4 s and a puddle spreads below in 0.5 s. Strong: the bucket rocks back, tips 16° with a stop and the puddle surges past full before settling. The puddle is retained; preview reset rearms.',[
 ...posed([...framed('body',bodyD),...tube('handle',handleD)],bucketPose),
 ...shown(tube('stream',(t,s)=>{const [x,y]=lip(t,s);return `M${x} ${y+.6} L${x} 23.4`;}),pouring),
 ...shown(framed('puddle',puddleD),(t,s)=>spread(t,s)>.03?1:0)
],.7,.64));
fillBucket.duotoneFill=true;

// ---------- EYEDROPPER ----------
// A dropper at 45° above a colour swatch: its bulb is squeezed, the tip dips onto the swatch, the bulb
// releases to draw the sample and the dropper lifts. Light: a soft squeeze and a 1.2 unit dip. Strong: a
// hard squeeze, a 1.7 unit stab with a stop, a snap release that wobbles the swatch, and a rebound.
const TP=[11.5,21],DG=Math.SQRT1_2,BULB=[TP[0]+15.8*DG,TP[1]-15.8*DG];
const coneD=place(polygon([[0,0],[-2.1,-3.2],[2.1,-3.2]],[.4,.6,.6]),45,...TP),coneSolid=place(polygon([[0,0],[-2.1,-3.5],[2.1,-3.5]],[.4,.6,.6]),45,...TP);
const tubeD=place(rect(-2.1,-11,4.2,7.9,.6),45,...TP),collarD=place(rect(-2.8,-13.2,5.6,2.4,.6),45,...TP),bulbD=place(circle(0,-15.8,2.9),45,...TP);
const dipE=F([[0,0],[.1,0],[.26,1.2,'smooth'],[.36,1.2],[.56,0,'smooth']],[[0,0],[.06,-.8],[.24,1.7,'hit'],[.36,1.7,'hold'],[.5,-.3],[.6,0]]);
const squeeze=F([[0,1],[.06,1],[.2,.86,'smooth'],[.34,.86],[.5,1,'smooth']],[[0,1],[.12,.74],[.24,.74,'hold'],[.3,1.12,'launch'],[.4,.96],[.5,1]]);
const wobble=F([[0,1],[.5,1]],[[0,1],[.24,1,'hold'],[.3,.84],[.38,1.08],[.48,1]]);
const dropPose=rigid((t,s)=>({dx:-DG*dipE(t,s),dy:DG*dipE(t,s),pivot:TP}));
const bulbPose=(x,y,t,s)=>{const k=squeeze(t,s),d=dipE(t,s);return [BULB[0]+(x-BULB[0])*k-DG*d,BULB[1]+(y-BULB[1])*k+DG*d];};
const eyedropper=make('eyedropper','Eyedropper',
 'Light: the dropper\'s bulb is squeezed, the tip dips 1.2 units onto the swatch, the bulb releases to draw the sample and the dropper lifts. Strong: a hard squeeze, a 1.7 unit stab with a stop, a snap release that wobbles the swatch, and a rebound.',[
 ...bothFill('swatch',(t,s)=>circle(8,24.5,2.3*wobble(t,s))),
 ...posed([stroke('cone',coneD),fill('cone-solid',coneSolid,'solid'),stroke('tube',tubeD),fill('tube-solid',tubeD,'solid'),stroke('collar',collarD),fill('collar-solid',collarD,'solid')],dropPose),
 ...posed([stroke('bulb',bulbD),fill('bulb-solid',bulbD,'solid')],bulbPose)
],.58,.62);
eyedropper.duotoneFill=false;

export const icons=[rectangleTool,ellipseTool,polygonTool,fillBucket,eyedropper];
