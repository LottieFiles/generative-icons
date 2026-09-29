// Drawing & design tools batch C: group-objects, align-objects, magic-wand, lasso-select, transform-tool.
import {make,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,rotate,bothFill,capsule,toggle,FR,pt} from './helpers.mjs';
import svgpath from 'svgpath';
const shell=(name,d)=>[stroke(name,d),fill(name+'-solid',d,'solid')];

// ---------- GROUP OBJECTS ----------
// A square and a circle sit apart; grouping pulls them together and a group frame fades in around them.
// Light: a 0.44 s glide and a fade. Strong: the objects flinch apart, snap together with a stop and the
// frame pops in oversized. Two states: ungrouped ↔ grouped; reverse dissolves the frame and parts them.
const FRAME=rect(5.5,6,21.5,19.4,2.6),FC=[16.25,15.7];
const gather=FR([[0,0],[.06,0],[.5,1,'smooth']],[[0,0],[.08,-.18],[.3,1.04,'hit'],[.4,.98],[.5,1]],[[0,1],[.06,1],[.5,0,'smooth']],[[0,1],[.08,1,'hold'],[.28,-.12],[.4,.04],[.5,0]]);
const frameA=FR([[0,0],[.3,0],[.55,1,'out']],[[0,0],[.28,0,'hold'],[.4,1]],[[0,1],[.04,1],[.3,0,'in']],[[0,1],[.04,1,'hold'],[.16,0]]);
const frameK=FR([[0,1],[.5,1]],[[0,1.08],[.28,1.08,'hold'],[.46,1]],[[0,1],[.5,1]],[[0,1],[.5,1]]);
const squarePose=rigid((t,s,r)=>{const g=gather(t,s,r);return {dx:2.8*g,dy:-2.8*g,pivot:[9.4,21.4]};});
const circlePose=rigid((t,s,r)=>{const g=gather(t,s,r);return {dx:-2.8*g,dy:2.8*g,pivot:[23.4,9.8]};});
const framePose=(x,y,t,s,r)=>{const k=frameK(t,s,r);return [FC[0]+(x-FC[0])*k,FC[1]+(y-FC[1])*k];};
const groupObjects=toggle('group-objects','Group objects',
 'Light: the square and the circle glide together in 0.44 s and a group frame fades in around them. Strong: they flinch apart, snap together with a stop and the frame pops in oversized. Reverse dissolves the frame and parts them.',[
 ...posed(shown(framed('frame',FRAME),frameA),framePose),
 ...posed(shell('square',rect(6.7,18.7,5.4,5.4,1)),squarePose),
 ...posed(shell('circle',circle(23.4,9.8,2.7)),circlePose)
],.58,.52,'ungrouped','grouped',.54,.52);
groupObjects.duotoneFill=true;

// ---------- ALIGN OBJECTS ----------
// Three blocks of different sizes sit ragged right of a vertical guide; aligning slides their left edges
// onto one line, top block first. Light: staggered 0.44 s slides. Strong: each block pulls back, whips to
// the line with a stop and settles. Two states: ragged ↔ aligned; reverse scatters them again.
const BARS=[[12.5,7.2,12,4.4],[15,14,7.5,3.2],[13.5,19.6,9.5,5.2]];
const slide=i=>{const d=.05*i;return FR([[0,0],[.06+d,0],[.5+d,1,'smooth']],[[0,0],[.06+d,-.25],[.26+d,1.12,'hit'],[.36+d,.97],[.44+d,1]],[[0,1],[.06+d,1],[.5+d,0,'smooth']],[[0,1],[.06+d,1,'hold'],[.26+d,-.1],[.36+d,.03],[.44+d,0]]);};
const alignObjects=toggle('align-objects','Align objects',
 'Light: three ragged blocks slide left in turn until their edges sit on the guide, 0.44 s each. Strong: each block pulls back, whips to the guide with a stop and settles. Reverse scatters them to their ragged positions.',[
 ...tube('guide','M6.5 5.5 L6.5 26.5'),
 ...BARS.flatMap(([x,y,w,h],i)=>{const run=slide(i);return posed(bothFill('block-'+i,rect(x,y,w,h,Math.min(1.8,h/2))),rigid((t,s,r)=>({dx:-(x-10)*run(t,s,r),pivot:[x,y]})));})
],.64,.58,'ragged','aligned',.64,.58);
alignObjects.duotoneFill=false;

// ---------- MAGIC WAND ----------
// A wand with a four-point star at its tip taps and sparkles pop out. Light: a 6° tap and two soft
// sparkles while the star twinkles. Strong: the wand winds back 8°, flicks 16° with a stop, the star spins
// a full turn and three sparkles burst in sequence. Gesture: everything settles, sparkles fade.
const WB=[7,25],SC=[20.8,11.2];
const star=(c,ro,ri,rot=0)=>polygon(Array.from({length:8},(_,i)=>pt(rot-90+45*i,i%2?ri:ro,c)),.3);
const wand=F([[0,0],[.06,0],[.22,-6,'smooth'],[.5,0,'smooth']],[[0,0],[.1,8],[.24,-16,'hit'],[.38,4],[.5,-1],[.58,0]]);
const twinkle=F([[0,1],[.1,1],[.24,1.25,'smooth'],[.44,1,'smooth']],[[0,1],[.1,.9],[.28,1.4,'hit'],[.42,.95],[.52,1]]);
const spin=F([[0,0],[.5,0]],[[0,0],[.1,-25],[.42,360,'hit'],[.5,360]]);
const starPose=(x,y,t,s)=>{const a=wand(t,s),c=rotate(...SC,a,...WB),k=twinkle(t,s),[px,py]=rotate(...rotate(x,y,a,...WB),spin(t,s),...c);return [c[0]+(px-c[0])*k,c[1]+(py-c[1])*k];};
const sparkle=(name,c,d,lightAt)=>{
 const t0=(s)=>s?d:lightAt;
 const scale=(t,s)=>{const a=t0(s);return a==null?.35:F([[0,.35],[a,.35],[a+.1,1.15,'out'],[a+.28,.9,'smooth']],[[0,.35],[a,.35,'hold'],[a+.08,1.2],[a+.28,.9]])(t,s);};
 const alpha=(t,s)=>{const a=t0(s);return a==null?0:F([[0,0],[a,0],[a+.06,1,'out'],[a+.2,1],[a+.28,0,'in']],[[0,0],[a,0,'hold'],[a+.05,1],[a+.2,1,'hold'],[a+.28,0]])(t,s);};
 return shown(bothFill(name,(t,s)=>{const k=scale(t,s);return d==null?circle(...c,k):star(c,1.8*k,.65*k);}),alpha);
};
const magicWand=make('magic-wand','Magic wand',
 'Light: the wand taps 6° and two soft sparkles pop while the tip star twinkles. Strong: the wand winds back 8°, flicks 16° with a stop, the star spins a full turn and three sparkles burst in sequence before fading.',[
 ...posed(tube('stick',`M${WB} L${SC[0]-1.7} ${SC[1]+1.7}`),rigid((t,s)=>({angle:wand(t,s),pivot:WB}))),
 ...posed(bothFill('star',star(SC,3.6,1.6)),starPose),
 ...sparkle('sparkle-a',[26.5,7.5],.24,.14),
 ...sparkle('sparkle-b',[26,16.2],.29,.2),
 ...shown(bothFill('sparkle-c',(t,s)=>{const a=s?.34:null,k=a==null?.35:F([[0,.35]],[[0,.35],[a,.35,'hold'],[a+.08,1.2],[a+.28,.9]])(t,s);return circle(17,6,k);}),(t,s)=>s?F([[0,0]],[[0,0],[.34,0,'hold'],[.39,1],[.54,1,'hold'],[.62,0]])(t,s):0)
],.52,.62);
magicWand.duotoneFill=false;

// ---------- LASSO SELECT ----------
// A lasso loop encloses an object, its tail trailing to a handle. Selecting cinches the loop around the
// object while the tail is pulled, then the rope relaxes. Light: an 18% cinch and a 1.5 unit pull. Strong:
// the loop widens first, snaps to 76% with a stop while the tail is yanked 3 units, then relaxes.
const LC=[14,14.5],LRX=8.5,LRY=6.6,LTILT=-12,E0=[24.5,25];
const loopPt=(deg,k)=>{const p=rotate(LC[0]+LRX*Math.cos(deg*Math.PI/180),LC[1]+LRY*Math.sin(deg*Math.PI/180),LTILT,...LC);return [LC[0]+(p[0]-LC[0])*k,LC[1]+(p[1]-LC[1])*k];};
const cinch=F([[0,1],[.06,1],[.3,.82,'smooth'],[.4,.82],[.6,1,'smooth']],[[0,1],[.08,1.07],[.24,.76,'hit'],[.36,.76,'hold'],[.52,1.03],[.62,1]]);
const pull=F([[0,0],[.06,0],[.3,1.5,'smooth'],[.4,1.5],[.6,0,'smooth']],[[0,0],[.08,-.8],[.24,3,'hit'],[.36,3,'hold'],[.52,-.3],[.62,0]]);
// Exact scaled/tilted ellipse: four cubic quadrants built in the loop's own frame, then tilted and cinched.
const loopD=(t,s)=>{const k=cinch(t,s),K=.5523,p=a=>loopPt(a,k);
 const w=v=>{const r=rotate(LC[0]+v[0],LC[1]+v[1],LTILT,...LC);return [LC[0]+(r[0]-LC[0])*k,LC[1]+(r[1]-LC[1])*k];};
 const q=a0=>{const A=a0*Math.PI/180,B=(a0+90)*Math.PI/180,c1=[LRX*(Math.cos(A)-K*Math.sin(A)),LRY*(Math.sin(A)+K*Math.cos(A))],c2=[LRX*(Math.cos(B)+K*Math.sin(B)),LRY*(Math.sin(B)-K*Math.cos(B))];return ' C'+[...w(c1),...w(c2),...p(a0+90)];};
 return 'M'+p(0)+q(0)+q(90)+q(180)+q(270)+' Z';};
const knot=(t,s)=>loopPt(45,cinch(t,s));
const tailD=(t,s)=>{const k=knot(t,s),u=pull(t,s),e=[E0[0]+.6*u,E0[1]+.8*u];return `M${k} C${k[0]+2} ${k[1]+3} ${e[0]-4.5} ${e[1]-1.5} ${e}`;};
const lassoSelect=make('lasso-select','Lasso select',
 'Light: the lasso loop cinches 18% around the object while its tail is pulled 1.5 units, then relaxes. Strong: the loop widens first, snaps to 76% with a stop as the tail is yanked 3 units, then relaxes with a small rebound.',[
 fill('loop-interior',loopD,'tint'),...tube('loop',loopD),
 ...tube('tail',tailD),
 ...bothFill('object',rect(12,12.5,4,4,.9))
],.62,.64);
lassoSelect.duotoneFill=true;

// ---------- TRANSFORM TOOL ----------
// A selection box with four corner handles around an object; dragging the far handle scales the box and
// its object from the origin corner, then the drag is released back. Light: a 14% scale over 0.56 s. Strong:
// a short back-step, a 26% scale with a stop, a hold and a spring return.
const TO=[8,8],scaleT=F([[0,1],[.06,1],[.3,1.14,'smooth'],[.4,1.14],[.62,1,'smooth']],[[0,1],[.06,.96],[.22,1.26,'hit'],[.34,1.26,'hold'],[.5,.98],[.6,1]]);
const grow=(x,y,t,s)=>{const k=scaleT(t,s);return [TO[0]+(x-TO[0])*k,TO[1]+(y-TO[1])*k];};
const handle=([x,y])=>(t,s)=>{const k=scaleT(t,s),c=[TO[0]+(x-TO[0])*k,TO[1]+(y-TO[1])*k];return rect(c[0]-1.3,c[1]-1.3,2.6,2.6,.55);};
const transformTool=make('transform-tool','Transform tool',
 'Light: the far corner handle is dragged and the selection box scales 14% from its origin corner with the object inside, then eases back. Strong: a short back-step, a 26% scale with a stop, a hold and a spring return.',[
 ...posed([...tube('box',rect(8,8,16,16,1)),...shell('object',circle(16,16,4))],grow),
 ...[[8,8],[24,8],[8,24],[24,24]].flatMap((c,i)=>bothFill('handle-'+i,handle(c)))
],.64,.62);
transformTool.duotoneFill=false;

export const icons=[groupObjects,alignObjects,magicWand,lassoSelect,transformTool];
