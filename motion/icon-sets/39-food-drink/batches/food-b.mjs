// Food & Drink batch B: cooking-pot, frying-pan, kettle, toaster, blender.
import {make,F,P,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,bothFill,rotate,holes} from './helpers.mjs';
import {wispD,recycle,L} from './kitchen.mjs';

// ---------- COOKING POT ----------
// Lid (flat underside y12.5) rests 1 unit clear of the pot rim at Bold. Boiling rattles the lid: it lifts
// and tilts about its own centre, then drops back. Handles are short stubs on the pot walls.
const potBody='M6.5 16 L25.5 16 L25.5 24.6 C25.5 26.2 24.2 27.5 22.6 27.5 L9.4 27.5 C7.8 27.5 6.5 26.2 6.5 24.6 Z';
const lidD='M6 12.5 C6 9.9 10.5 8.6 16 8.6 C21.5 8.6 26 9.9 26 12.5 Z';
const lidUp=F([[0,0],[.1,-.9],[.2,0,'in']],[[0,0],[.1,-2.2],[.2,0,'hit'],[.3,-1],[.38,0,'hit'],[.48,0]]);
const lidTip=F([[0,0],[.1,4],[.2,0,'in']],[[0,0],[.1,-8],[.2,0,'hit'],[.3,5],[.38,0,'hit'],[.48,0]]);
const potShake=F([[0,0],[.3,0]],[[0,0],[.2,0],[.23,.45],[.28,-.3],[.34,0]]);
const lidPose=rigid((t,s)=>({dy:lidUp(t,s),angle:lidTip(t,s),pivot:[16,10.5],dx:potShake(t,s)}));
const potPose=rigid((t,s)=>({dx:potShake(t,s)}));
const cookingPot=make('cooking-pot','Cooking pot',
 'Light: the lid bumps up 0.9 units, tips 4° and drops back. Strong: it jumps 2.2 units, tips 8°, lands hard, rattles once more and the pot shivers on the impact.',[
 ...posed([...framed('pot',potBody),...tube('handle-l','M3.9 18.6 L6.5 18.6'),...tube('handle-r','M25.5 18.6 L28.1 18.6')],potPose),
 ...posed([...framed('lid',lidD),...tube('knob-stem','M16 8.6 L16 7'),...tube('knob','M14.3 6.8 L17.7 6.8')],lidPose)
],.26,.5);
cookingPot.duotoneFill=true;

// ---------- FRYING PAN ----------
// Family-scale pan: a deep open body (x3.8–21.2, y12–22) with its handle to the right (to x28).
// The pancake is a disc seen side-on (flattened ellipse 8.1 × 3.5). It is tossed and turns a full
// revolution in the image plane while it is high enough to clear the pan (v1: a small pill that barely flipped).
const PC=[12.5,16.55],PRX=4.05,PRY=1.75,HANDLE_END=[28,12.2];
const toss=F([[0,0],[.06,0],[.3,-4.6,'out'],[.54,0,'in']],[[0,0],[.1,0,'hold'],[.3,-7.2,'out'],[.56,0,'hit'],[.66,-.5],[.74,0]]);
// Light turns the pancake over once, Strong twice; whole turns end on the exact start drawing.
const turn=F([[0,0],[.17,0],[.42,1,'smooth']],[[0,0],[.16,0,'hold'],[.46,2]]);
const flick=F([[0,0],[.06,-1.5],[.12,0],[.6,0]],[[0,0],[.06,3],[.12,-5,'launch'],[.24,0],[.56,0],[.6,2],[.68,0]]);
const panPose=rigid((t,s)=>({angle:flick(t,s),pivot:HANDLE_END}));
const cakePose=(x,y,t,s)=>{const [px,py]=rotate(...PC,flick(t,s),...HANDLE_END),a=(360*turn(t,s))%360,[rx,ry]=a<1e-9||a>360-1e-9?[x,y]:rotate(x,y,a,...PC);return [rx+(px-PC[0]),ry+toss(t,s)+(py-PC[1])];};
const K=.5523,pancake=`M${PC[0]} ${PC[1]-PRY} C${PC[0]+PRX*K} ${PC[1]-PRY} ${PC[0]+PRX} ${PC[1]-PRY*K} ${PC[0]+PRX} ${PC[1]} C${PC[0]+PRX} ${PC[1]+PRY*K} ${PC[0]+PRX*K} ${PC[1]+PRY} ${PC[0]} ${PC[1]+PRY} C${PC[0]-PRX*K} ${PC[1]+PRY} ${PC[0]-PRX} ${PC[1]+PRY*K} ${PC[0]-PRX} ${PC[1]} C${PC[0]-PRX} ${PC[1]-PRY*K} ${PC[0]-PRX*K} ${PC[1]-PRY} ${PC[0]} ${PC[1]-PRY} Z`;
const fryingPan=make('frying-pan','Frying pan',
 'Light: the pancake is tossed 4.6 units and turns over once in the air before landing. Strong: the pan dips and flicks, the pancake flies 7.2 units high, turns over twice and lands hard with a small bounce.',[
 ...posed([...tube('pan','M3.8 12 L5.3 19.2 C5.7 20.9 7.1 22 8.8 22 L16.2 22 C17.9 22 19.3 20.9 19.7 19.2 L21.2 12'),...tube('handle','M20.87 13.6 L28 12.2')],panPose),
 ...posed(framed('pancake',pancake),cakePose)
],.54,.74);
fryingPan.duotoneFill=true;

// ---------- KETTLE ----------
// Kettle body, handle and a spout on the left. Steam jets from the spout tip, fades and recycles.
// Strong: the kettle rocks on its base as it boils and the jet shoots higher.
const X=1.5;
const kBody=`M${8.5+X} 26.5 L${21.5+X} 26.5 C${22.6+X} 26.5 ${23.3+X} 25.6 ${23+X} 24.5 L${20.5+X} 14.8 C${20.2+X} 13.6 ${19.3+X} 12.9 ${18+X} 12.9 L${12+X} 12.9 C${10.7+X} 12.9 ${9.8+X} 13.6 ${9.5+X} 14.8 L${7+X} 24.5 C${6.7+X} 25.6 ${7.4+X} 26.5 ${8.5+X} 26.5 Z`;
const rock=F([[0,0],[.6,0]],[[0,0],[.08,-3.5],[.18,3],[.28,-2.2],[.38,1.2],[.48,0]]);
const kPose=rigid((t,s)=>({angle:rock(t,s),pivot:[15+X,26.5]}));
const jet=recycle(P,0,[1.8,3.2],[.5,.2],[.3,.16,.08]);
const kettle=make('kettle','Kettle',
 'Light: a wisp of steam rises 1.8 units from the spout, fades and reforms. Strong: the kettle rocks on its base as it boils and the steam jets up 3.2 units before reforming.',[
 ...posed([...framed('body',kBody),...tube('spout',`M${8.3+X} 19.6 L${4.6+X} 15`),...tube('handle',`M${11+X} 12.9 C${11+X} 8.6 ${12.6+X} 7.4 ${15+X} 7.4 C${17.4+X} 7.4 ${19+X} 8.6 ${19+X} 12.9`)],kPose),
 ...posed(shown(tube('steam',wispD(4.8+X,11.3,4.4,1.1)),jet.show),(x,y,t,s)=>{const [px,py]=rotate(4.6+X,15,rock(t,s),15+X,26.5);return [x+(px-4.6-X),y+(py-15)-jet.rise(t,s)];})
],.7,.54);
kettle.duotoneFill=true;

// ---------- TOASTER ----------
// Toast is drawn only above the toaster top (an open outline whose ends sit on the top edge; the closed
// fill is hidden inside the body). Popping raises the visible height from 1.8 to 7.4 units; the lever jumps.
const T=14,SLICES=[[8.4,14.2],[17.8,23.6]];
const toastD=([x0,x1],h,closed)=>{const top=T-h,xm=(x0+x1)/2;const d=`M${x0} ${T} L${x0} ${top+1.5} C${x0} ${top+.2} ${x0+1} ${top-.5} ${xm} ${top-.5} C${x1-1} ${top-.5} ${x1} ${top+.2} ${x1} ${top+1.5} L${x1} ${T}`;return closed?d+' Z':d;};
const pop=k=>F([[0,1.8],[.06+k*.04,1.8],[.4+k*.04,7.4,'out']],[[0,1.8],[.12+k*.03,1.8,'hold'],[.3+k*.03,9.2],[.44+k*.03,6.9],[.54+k*.03,7.4]]);
const lever=F([[0,0],[.06,0],[.3,-4.5,'out']],[[0,0],[.12,0,'hold'],[.22,-4.5,'hit']]);
const jolt=F([[0,1],[.5,1]],[[0,1],[.06,.95],[.12,1],[.5,1]]);
const bodyPose=(x,y,t,s)=>[x,26.5+(y-26.5)*jolt(t,s)];
const toaster=make('toaster','Toaster',
 'Light: two slices rise smoothly out of the slots and the lever springs up. Strong: the toaster jolts, the slices pop 1.8 units past their rest, drop back with one small bounce and hold. Toast is retained; preview reset rearms.',[
 ...posed([...framed('body',rect(5.5,T,21,12.5,3.2)),stroke('slot-bar','M9 20.4 L23 20.4')],bodyPose),
 // Toast rides the body's jolt so its feet stay on the top edge.
 ...SLICES.flatMap((sl,k)=>{const h=pop(k);return posed([fill('toast-'+k+'-interior',(t,s)=>toastD(sl,h(t,s),true),'tint'),stroke('toast-'+k,(t,s)=>toastD(sl,h(t,s),false)),fill('toast-'+k+'-solid',(t,s)=>toastD(sl,h(t,s),true),'solid')],bodyPose);}),
 ...posed(tube('lever','M26.5 23 L28.3 23'),rigid((t,s)=>({dy:lever(t,s)})))
],.44,.6,['toast-0-interior','toast-0','toast-0-solid','toast-1-interior','toast-1','toast-1-solid','lever']);
toaster.duotoneFill=true;

// ---------- BLENDER ----------
// Tapered jar under a lid band, on a filled motor base with a button counter. A cross blade low in the
// jar spins (side view: its width follows cos) and pulls the drink into a shallow vortex; the whole
// appliance buzzes. v1 (a line blade on a stem, no lid, outline base) read as a "T" in a box.
const JL=[9,7.3],JR=[23,7.3],JBR=[21.5,20.8],JBL=[10.5,20.8];
const jar=polygon([JL,JR,JBR,JBL],[.6,.6,.8,.8]);
const lid=rect(8.4,4.8,15.2,2.5,1.25);
const base=holes(rect(9,20.8,14,7,2),circle(16,24.3,1.4));
const wallX=(y,side)=>side<0?JL[0]+(y-JL[1])/(JBL[1]-JL[1])*(JBL[0]-JL[0]):JR[0]+(y-JR[1])/(JBR[1]-JR[1])*(JBR[0]-JR[0]);
const SY=10.8,BY=16.6;
const depth=F([[0,0],[.08,0],[.32,.8],[.8,0,'smooth']],[[0,0],[.06,0],[.22,1.2],[.5,.95],[.8,0]]);
const spinA=F([[0,0],[.06,0],[.84,720,'smooth']],[[0,0],[.04,0],[.8,1440,'smooth']]);
const buzz=(t,s)=>{const e=Math.max(0,Math.min(1,t/.1))*Math.max(0,1-(t-.1)/(s?.66:.68));return (s?.55:.2)*Math.sin(t*95)*e;};
const surfD=(t,s)=>{const xl=wallX(SY,-1),xr=wallX(SY,1),d=depth(t,s);return `M${xl} ${SY} C${xl+2.5} ${SY} ${14} ${SY+d} 16 ${SY+d} C18 ${SY+d} ${xr-2.5} ${SY} ${xr} ${SY}`;};
const liquidD=(t,s)=>surfD(t,s)+` L${JBR} L${JBL} Z`;
const bladeD=sign=>(t,s)=>{const c=Math.cos(spinA(t,s)*Math.PI/180)*2.2;return `M${16-c} ${BY-1.1*sign} L${16+c+.001} ${BY+1.1*sign}`;};
const bPose=rigid((t,s)=>({dx:buzz(t,s)}));
const blender=make('blender','Blender',
 'Light: the cross blade spins two turns and pulls the drink into a shallow vortex that relaxes. Strong: it spins four turns, the vortex deepens and the whole blender buzzes before settling.',[
 ...posed([...framed('lid',lid),stroke('jar',jar),{name:'jar-solid',d:jar,style:'solid',type:'stroke',width:2.6},
  fill('drink-interior',liquidD,'tint'),fill('drink-solid',liquidD,'solid'),...tube('surface',surfD),
  stroke('shaft',`M16 ${BY} L16 20.8`),stroke('blade-a',bladeD(1)),stroke('blade-b',bladeD(-1)),
  ...bothFill('base',base)],bPose)
],.84,.8);
blender.duotoneFill=true;

export const icons=[cookingPot,fryingPan,kettle,toaster,blender];
