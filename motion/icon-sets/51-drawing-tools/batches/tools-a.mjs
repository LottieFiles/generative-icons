// Drawing & design tools batch A: pen-tool, brush, eraser, corner-radius, line-tool.
import {make,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,capsule,holes,bothFill,rotate,toggle,geometricRearm,FR,reveal,bez,clamp01} from './helpers.mjs';
import svgpath from 'svgpath';
const place=(d,angle,x,y)=>svgpath(d).rotate(angle).translate(x,y).toString();

// ---------- PEN TOOL ----------
// A vector path ends at an anchor; the pen nib hovers 1.3 above it (diagonal). Clicking: the nib dips onto the
// anchor and a selection ring bursts from it. Light dips 1 unit with a soft ring; Strong lifts first, stabs
// 1.5 with a hard stop, the ring bursts wider and the nib rebounds. Gesture: nib returns, ring fades.
const A=[12.5,19.5],T0=[A[0]+2.7,A[1]-2.7],DIAG=Math.SQRT1_2;
const nibD=place(polygon([[0,0],[-2.7,-5.2],[0,-7.6],[2.7,-5.2]],[.6,.9,.9,.9]),45,...T0);
const slitD=place('M0 -1.5 L0 -4.4',45,...T0);
const barrelD=place(rect(-1.7,-12.8,3.4,5.4,1.2),45,...T0),barrelSolid=place(rect(-1.7,-12.8,3.4,5.7,1.2),45,...T0);
const dip=F([[0,0],[.08,0],[.22,1,'smooth'],[.3,1],[.5,0,'smooth']],[[0,0],[.1,-1.2],[.22,1.5,'hit'],[.34,-.4],[.46,.1],[.56,0]]);
const ringR=F([[0,2],[.2,2],[.5,5,'out']],[[0,2],[.2,2,'hold'],[.44,6.2]]);
const ringA=F([[0,0],[.2,0],[.24,1,'out'],[.5,0,'in']],[[0,0],[.2,0,'hold'],[.23,1],[.46,0]]);
const nibPose=rigid((t,s)=>({dx:-DIAG*dip(t,s),dy:DIAG*dip(t,s),pivot:T0}));
const penTool=make('pen-tool','Pen tool',
 'Light: the pen nib dips 1 unit onto the path\'s last anchor and a soft selection ring spreads from it. Strong: the nib lifts, stabs 1.5 units with a hard stop, the ring bursts to twice the size and the nib rebounds.',[
 ...tube('path','M5.5 26.5 C6.5 22.5 8.5 19.5 12.5 19.5'),
 ...bothFill('start',circle(5.5,26.5,1.3)),
 ...bothFill('anchor',circle(...A,1.3)),
 ...shown(tube('ring',(t,s)=>circle(...A,ringR(t,s))),ringA),
 ...posed([stroke('nib',nibD),fill('nib-solid',holes(nibD,place(capsule([0,-1.6],[0,-4.3],1.2),45,...T0)),'solid'),stroke('slit',slitD),
  stroke('barrel',barrelD),fill('barrel-solid',barrelSolid,'solid')],nibPose)
],.52,.58);
penTool.duotoneFill=false;

// ---------- BRUSH ----------
// A paintbrush (bristles, ferrule, handle at 60°) sweeps along an S-shaped swash that appears under its tip.
// Light: one calm 0.54 s sweep with a slight lean. Strong: the brush lifts, drops onto the canvas, sweeps in
// 0.24 s leaning into the stroke and flicks up at the end. The swash is retained; preview reset rearms.
const SW=bez([5.5,24.5],[10,19],[13,28.5],[19.5,22]),S0=SW(0);
const bristleD=place(polygon([[0,0],[-2,-3.8],[2,-3.8]],[.5,.8,.8]),30,...S0);
const ferruleD=place(rect(-2,-6.4,4,2.7,.5),30,...S0),ferruleSolid=place(rect(-2,-6.5,4,2.9,.5),30,...S0);
const handleD=place('M0 -6.4 L0 -13.2',30,...S0),handleSolid=place(capsule([0,-6.1],[0,-13.2],3),30,...S0);
const sweep=F([[0,0],[.06,0],[.6,1,'smooth']],[[0,0],[.12,0,'hold'],[.36,1],[.4,1]]);
const lift=F([[0,0],[.6,0]],[[0,0],[.06,-1.4],[.12,0,'hit'],[.36,0],[.44,-1.6],[.56,0]]);
const lean=F([[0,0],[.06,0],[.36,-5],[.6,0,'smooth']],[[0,0],[.06,-12],[.12,-4],[.36,10],[.48,-3],[.56,0]]);
const brushPose=rigid((t,s)=>{const p=SW(clamp01(sweep(t,s)));return {dx:p[0]-S0[0],dy:p[1]-S0[1]+lift(t,s),angle:lean(t,s),pivot:S0};});
const swashD=reveal(SW,4);
const brush=geometricRearm(make('brush','Brush',
 'Light: the brush sweeps down and across in one 0.54 s stroke and the paint swash appears under its bristles. Strong: it lifts, drops onto the canvas, sweeps in 0.24 s leaning into the stroke and flicks up at the end. The swash is retained; preview reset rearms.',[
 ...shown(tube('swash',(t,s)=>swashD(sweep(t,s))),(t,s)=>sweep(t,s)>.02?1:0),
 ...posed([...framed('bristles',bristleD).filter(p=>!p.name.endsWith('-interior')),stroke('ferrule',ferruleD),fill('ferrule-solid',ferruleSolid,'solid'),stroke('handle',handleD),fill('handle-solid',handleSolid,'solid')],brushPose)
],.62,.58));
brush.duotoneFill=false;

// ---------- ERASER ----------
// A tilted eraser block scrubs leftward 1 unit above a pencil line, which shortens behind its leading edge.
// Light: a 0.56 s slide with a light rub. Strong: the block lifts, drops with a stop and scrubs harder and
// faster. Two states: drawn ↔ erased; the reverse slides the block back and the line redraws.
const EC=[23,15],blockD=place(rect(-4.2,-2.2,8.4,4.4,1.2),-28,...EC),bandD=place('M-1.2 -2.2 L-1.2 2.2',-28,...EC);
const blockSolid=holes(blockD,place(capsule([-1.2,-1.4],[-1.2,1.4],1.4),-28,...EC));
const wipe=FR([[0,0],[.06,0],[.62,1,'smooth']],[[0,0],[.16,0,'hold'],[.5,1],[.56,1]],[[0,1],[.06,1],[.56,0,'smooth']],[[0,1],[.1,1,'hold'],[.42,0],[.48,0]]);
const hop=FR([[0,0],[.62,0]],[[0,0],[.08,-1.3],[.16,0,'hit'],[.5,0],[.56,0]],[[0,0],[.56,0]],[[0,0],[.1,-1.3],[.4,-1.3],[.48,0,'hit'],[.56,0]]);
const rub=(t,s,r)=>{const [a,b,amp,n]=r?[.06,.5,s?.9:.4,s?4:3]:[s?.16:.06,s?.5:.62,s?1.1:.5,s?4:3],u=(t-a)/(b-a);return u<=0||u>=1?0:amp*Math.sin(Math.PI*u)*Math.sin(2*Math.PI*n*u);};
const eraserPose=rigid((t,s,r)=>({dx:-11*wipe(t,s,r)+rub(t,s,r),dy:hop(t,s,r),angle:rub(t,s,r)*2.5,pivot:EC}));
const eraser=toggle('eraser','Eraser',
 'Light: the eraser block slides left across the line in 0.56 s with a light rub and the line shortens behind its edge. Strong: it lifts, drops with a stop and scrubs harder and faster. Reverse slides it back while the line redraws.',[
 ...tube('line',(t,s,r)=>`M5.5 22.5 L${26-12.5*wipe(t,s,r)} 22.5`),
 ...posed([stroke('block',blockD),fill('block-solid',blockSolid,'solid'),stroke('band',bandD)],eraserPose)
],.64,.58,'drawn','erased',.58,.58);
eraser.duotoneFill=false;

// ---------- CORNER RADIUS ----------
// A square's corners round from 0.6 to 6 while the radius handle slides inward along the corner diagonal.
// Light: a smooth 0.5 s change. Strong: it snaps past to 7.4 and settles at 6. Two states; reverse sharpens.
const radius=FR([[0,.6],[.06,.6],[.56,6,'smooth']],[[0,.6],[.08,.6,'hold'],[.3,7.4],[.42,5.6],[.52,6]],[[0,6],[.06,6],[.52,.6,'smooth']],[[0,6],[.08,6,'hold'],[.28,.3],[.4,.9],[.5,.6]]);
const handleAt=r=>8+3.3+.42*r;
const cornerRadius=toggle('corner-radius','Corner radius',
 'Light: the square\'s corners round from sharp to a 6-unit radius in 0.5 s while the radius handle slides inward along the diagonal. Strong: the radius snaps past 7 and settles at 6. Reverse sharpens the corners.',[
 ...framed('square',(t,s,r)=>rect(8,8,16,16,radius(t,s,r)),(t,s,r)=>{const q=radius(t,s,r),h=handleAt(q);return holes(rect(8,8,16,16,q),circle(h,h,1.7));}),
 fill('handle',(t,s,r)=>{const h=handleAt(radius(t,s,r));return circle(h,h,1.3);},'outline')
],.6,.56,'sharp','rounded',.56,.54);
cornerRadius.duotoneFill=true;

// ---------- LINE TOOL ----------
// A line is pulled from a start anchor to an end anchor; the end anchor appears when the line reaches it.
// Light: a 0.52 s draw and a fade-in. Strong: the start anchor is pressed, the line shoots across in 0.18 s
// and the end anchor pops in oversized and settles. The line is retained; preview reset rearms.
const LA=[7,24.5],LB=[25,7.5];
const draw=F([[0,0],[.08,0],[.6,1,'smooth']],[[0,0],[.1,0,'hold'],[.28,1],[.34,1]]);
const pressA=F([[0,1],[.6,1]],[[0,1],[.06,1.4],[.14,1]]);
const popB=F([[0,1],[.6,1]],[[0,1.6],[.28,1.6],[.38,1,'out']]);
const showB=F([[0,0],[.5,0],[.62,1,'out']],[[0,0],[.27,0],[.31,1,'out']]);
const square=(c,k)=>rect(c[0]-1.5*k,c[1]-1.5*k,3*k,3*k,.6*k);
const lineTool=geometricRearm(make('line-tool','Line tool',
 'Light: a line is pulled from the start anchor to the end anchor in 0.52 s and the end anchor fades in. Strong: the start anchor is pressed, the line shoots across in 0.18 s and the end anchor pops in oversized and settles. The line is retained; preview reset rearms.',[
 ...shown(tube('line',(t,s)=>{const p=draw(t,s);return `M${LA} L${LA[0]+(LB[0]-LA[0])*p} ${LA[1]+(LB[1]-LA[1])*p}`;}),(t,s)=>draw(t,s)>.01?1:0),
 ...bothFill('start',(t,s)=>square(LA,pressA(t,s))),
 ...shown(bothFill('end',(t,s)=>square(LB,popB(t,s))),showB)
],.64,.5));
lineTool.duotoneFill=false;

export const icons=[penTool,brush,eraser,cornerRadius,lineTool];
