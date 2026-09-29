// Video editing (53). Timeline clips, frames and editor tools drawn on the shared 32-unit grid.
// Clips are horizontal rounded bars; frames are rounded panels whose Solid form is a ring so the
// detail inside survives. Designs, reviews and corrections: evidence/design-review.md.
import {make,timePair,F,P,fill,stroke,tube,framed,bothFill,posed,shown,rigid,rect,circle,polygon,capsule,arc} from '../library/ai-authoring.mjs';

// ---------- shared assembly ----------
// A panel keeps its interior open in Solid (ring) and tints it in Duotone.
const panel=(name,d)=>[fill(name+'-interior',d,'tint'),...tube(name+'-frame',d)];
// An outline ring that becomes a filled surface in Solid.
const shape=(name,d,solid=d)=>[stroke(name,d),fill(name+'-solid',solid,'solid')];
// Toggle drivers: a forward route and a separately directed return route.
const R=(out,back)=>(t,s,r)=>r?back(t,s):out(t,s);
function toggle(name,label,intent,parts,light,strong,initialState,finalState,reverseLight=light,reverseStrong=strong){
 return {...make(name,label,intent,parts,light,strong),stateful:true,initialState,finalState,reverseSuffix:'-reverse',forwardLabel:finalState,reverseLabel:initialState,reverseTimings:timePair(reverseLight,reverseStrong)};
}
const seg=(a,b)=>`M${a[0]} ${a[1]} L${b[0]} ${b[1]}`;

// ---------- CUT CLIP (result, preview reset) ----------
// Clip x4.5–27.5, y13–22. A razor (T handle) starts 3.8 above the clip, drops through it and leaves a
// full-height cut mark joined to both edges. Strong lifts first, slams through and the clip dips on impact.
const bladeY=F([[0,0],[.05,0],[.5,12.8,'smooth']],[[0,0],[.1,-.6],[.24,12.8,'hit'],[.3,12.8]]);
const bladeShow=P([[0,1],[.5,1],[.66,0]],[[0,1],[.26,1],[.4,0]]);
const markShow=P([[0,0],[.47,0],[.52,1]],[[0,0],[.22,0],[.26,1]]);
const cutDip=F([[0,0],[.5,0]],[[0,0],[.24,0],[.3,.7,'hit'],[.42,-.15],[.52,0]]);
const cutClip=make('cut-clip','Cut clip',
 'Light: the razor glides down through the clip and leaves a cut mark joined to both edges. Strong: it lifts, slams through with the clip dipping on impact, and the mark stays.',[
 ...posed(panel('clip',rect(4.5,13,23,9,2.3)),rigid((t,s)=>({dy:cutDip(t,s)}))),
 ...shown(posed([...tube('blade-handle','M13.8 4.5 L18.2 4.5'),...tube('blade-stem','M16 4.5 L16 9.2')],rigid((t,s)=>({dy:bladeY(t,s)}))),bladeShow),
 ...shown(tube('mark','M16 13 L16 22'),markShow)
],.8,.7,['blade','mark']);
cutClip.duotoneFill=true;

// ---------- TRIM CLIP (toggle full ↔ trimmed) ----------
// The clip's right end is square and carries the trim handle: a vertical bar on the edge that
// overhangs 3 above and below. Dragging the handle left shortens the clip by 7.
const trimOut=F([[0,24.5],[.05,24.5],[.6,17.5,'smooth']],[[0,24.5],[.1,25.1],[.34,17,'hit'],[.5,17.5]]);
const trimBack=F([[0,17.5],[.05,17.5],[.6,24.5,'smooth']],[[0,17.5],[.1,16.9],[.34,25,'hit'],[.5,24.5]]);
const trimEdge=R(trimOut,trimBack);
const trimBody=(t,s,r)=>{const e=trimEdge(t,s,r);return polygon([[4.5,11.5],[e,11.5],[e,20.5],[4.5,20.5]],[2.3,0,0,2.3]);};
const trimClip=toggle('trim-clip','Trim clip',
 'Light: the trim handle drags the clip end 7 units to the left. Strong: a short backswing, then a fast pull with one bounded settle. The reverse drags it back out.',[
 ...framed('clip',trimBody),
 ...tube('handle',(t,s,r)=>{const e=trimEdge(t,s,r);return `M${e} 8.5 L${e} 23.5`;})
],.7,.62,'full','trimmed');
trimClip.duotoneFill=true;

// ---------- TRANSITION (toggle cut ↔ dissolve) ----------
// Two 11-unit frames on a diagonal. In the cut state their corners clear by 1.5 painted at Bold;
// the second frame slides 5.5 up-left to overlap the first for a dissolve (tints stack in Duotone).
const transOut=F([[0,0],[.05,0],[.6,5.5,'smooth']],[[0,0],[.12,-.6],[.36,5.9,'hit'],[.5,5.5]]);
const transBack=F([[0,5.5],[.05,5.5],[.6,0,'smooth']],[[0,5.5],[.12,6.1],[.36,-.4,'hit'],[.5,0]]);
const transU=R(transOut,transBack);
const transition=toggle('transition','Transition',
 'Light: the second frame slides up over the first into a dissolve overlap. Strong: it draws back, drives in with a hard stop and settles. Reverse separates the frames into a hard cut.',[
 ...framed('clip-a',rect(4,4,11,11,2)),
 ...posed(framed('clip-b',rect(17,17,11,11,2)),rigid((t,s,r)=>({dx:-transU(t,s,r),dy:-transU(t,s,r)})))
],.7,.66,'cut','dissolve');
transition.duotoneFill=true;

// ---------- SPEED RAMP (result, preview reset) ----------
// A speed curve rises from flat (y17) to fast (y6.5) above a clip; a bead rides it.
const RP=[[5.5,17],[13,17],[17.5,6.5],[26.5,6.5]];
const bez=u=>{const v=1-u;return [0,1].map(i=>v*v*v*RP[0][i]+3*v*v*u*RP[1][i]+3*v*u*u*RP[2][i]+u*u*u*RP[3][i]);};
const rampU=F([[0,0],[.05,0],[.85,1,'smooth']],[[0,0],[.14,-.03],[.3,.12],[.42,.5],[.52,1]]);
const speedRamp=make('speed-ramp','Speed ramp',
 'Light: the bead rides the speed curve steadily to the top. Strong: it hesitates, then accelerates up the ramp and arrives in 0.4 s. The ramped position is retained; preview reset rearms.',[
 ...framed('clip',rect(4.5,21.5,23,6,2.3)),
 ...tube('curve',`M${RP[0]} C${RP[1]} ${RP[2]} ${RP[3]}`),
 ...bothFill('marker',(t,s)=>circle(...bez(rampU(t,s)),2))
],.9,.8,['marker']);
speedRamp.duotoneFill=true;

// ---------- REVERSE CLIP (toggle forward ↔ reversed) ----------
// A play triangle inside a clip panel flips through its vertical axis to point left.
const flipOut=F([[0,1],[.05,1],[.5,-1,'smooth']],[[0,1],[.1,1.15],[.32,-1.2,'hit'],[.46,-1]]);
const flipBack=F([[0,-1],[.05,-1],[.5,1,'smooth']],[[0,-1],[.1,-1.15],[.32,1.2,'hit'],[.46,1]]);
const flipK=R(flipOut,flipBack),XM=16.7;
const playD=(t,s,r)=>{const k=flipK(t,s,r),px=x=>XM+(x-XM)*k;return polygon([[px(13.2),12],[px(20.2),16],[px(13.2),20]],.9);};
const reverseClip=toggle('reverse-clip','Reverse clip',
 'Light: the play triangle flips through its axis to point left. Strong: it stretches forward first, snaps through with a hard stop past the mirror and settles. Reverse flips it forward again.',[
 ...panel('clip',rect(4.5,8,23,16,2.5)),
 ...bothFill('play',playD)
],.6,.6,'forward','reversed');
reverseClip.duotoneFill=true;

// ---------- ROTATE CLIP (toggle landscape ↔ portrait) ----------
const rotOut=F([[0,0],[.05,0],[.6,90,'smooth']],[[0,0],[.1,-8],[.36,98,'hit'],[.52,90]]);
const rotBack=F([[0,90],[.05,90],[.6,0,'smooth']],[[0,90],[.1,98],[.36,-8,'hit'],[.52,0]]);
const rotA=R(rotOut,rotBack),rotPose=rigid((t,s,r)=>({angle:rotA(t,s,r)}));
const rotateClip=toggle('rotate-clip','Rotate clip',
 'Light: the 20×12 frame and its play mark turn a quarter turn clockwise into portrait. Strong: a small counter-wind, a fast turn past 90° and a settle back. Reverse turns it back.',[
 ...posed(panel('frame',rect(6,10,20,12,2.2)),rotPose),
 ...posed(bothFill('play',polygon([[13.8,13.5],[19,16],[13.8,18.5]],.8)),rotPose)
],.7,.62,'landscape','portrait');
rotateClip.duotoneFill=true;

// ---------- GREEN SCREEN (toggle backdrop ↔ keyed) ----------
// The 23-unit backdrop is drawn as four corner brackets whose arms meet at the midpoints (0.3
// overlap). Keying retracts the arms to 4.5 and the tinted backdrop disappears; the subject stays.
const keyOut=F([[0,11.8],[.05,11.8],[.6,4.5,'smooth']],[[0,11.8],[.1,12.4],[.32,3.9,'hit'],[.48,4.5]]);
const keyBack=F([[0,4.5],[.05,4.5],[.6,11.8,'smooth']],[[0,4.5],[.1,3.9],[.32,12.4,'hit'],[.48,11.8]]);
const keyL=R(keyOut,keyBack);
const keyTint=R(P([[0,1],[.1,1],[.5,0]],[[0,1],[.06,1],[.3,0]]),P([[0,0],[.1,0],[.5,1]],[[0,0],[.06,0],[.3,1]]));
const bracket=(cx,cy,sx,sy)=>(t,s,r)=>{const l=keyL(t,s,r);return `M${cx} ${cy+sy*l} L${cx} ${cy+sy*2} C${cx} ${cy+sy*.9} ${cx+sx*.9} ${cy} ${cx+sx*2} ${cy} L${cx+sx*l} ${cy}`;};
const greenScreen=toggle('green-screen','Green screen',
 'Light: the backdrop frame retracts into corner brackets and its tint fades, leaving the keyed subject. Strong: the arms stretch out, snap in past the mark and settle. Reverse restores the backdrop.',[
 ...shown([fill('backdrop-interior',rect(4.5,4.5,23,23,2),'tint')],keyTint),
 ...tube('corner-tl',bracket(4.5,4.5,1,1)),...tube('corner-tr',bracket(27.5,4.5,-1,1)),
 ...tube('corner-bl',bracket(4.5,27.5,1,-1)),...tube('corner-br',bracket(27.5,27.5,-1,-1)),
 ...shape('head',circle(16,11,2.6)),
 ...shape('shoulders','M8 23.5 C8 18.5 11.5 17 16 17 C20.5 17 24 18.5 24 23.5','M8 23.5 C8 18.5 11.5 17 16 17 C20.5 17 24 18.5 24 23.5 Z')
],.7,.62,'backdrop','keyed');
greenScreen.duotoneFill=true;

// ---------- STABILIZE (gesture) ----------
// The frame shakes about its centre while the lock ring stays put; the shake damps and locks.
const shX=F([[0,0],[.06,-.5],[.18,.45],[.3,-.3],[.42,.15],[.56,0]],[[0,0],[.05,-1.6],[.14,1.4],[.24,-1],[.34,.7,'hit'],[.44,-.2],[.56,0]]);
const shY=F([[0,0],[.1,.3],[.24,-.2],[.4,0]],[[0,0],[.08,.8],[.2,-.8],[.3,.5,'hit'],[.42,-.15],[.56,0]]);
const shA=F([[0,0],[.08,1],[.22,-.8],[.36,.4],[.5,0]],[[0,0],[.06,3.5],[.16,-3.5],[.26,2.2],[.34,-.9,'hit'],[.46,.3],[.56,0]]);
const stabilize=make('stabilize','Stabilize',
 'Light: the frame jitters half a unit and settles around the fixed lock ring. Strong: it shakes 1.6 units with a 3.5° wobble, snaps level with a hit and locks still.',[
 ...posed(panel('frame',rect(6,7.5,20,17,2.5)),rigid((t,s)=>({dx:shX(t,s),dy:shY(t,s),angle:shA(t,s)}))),
 ...tube('ring',circle(16,16,3.4))
],.6,.6);
stabilize.duotoneFill=true;

// ---------- THUMBNAIL (gesture) ----------
// A 24×19 card with a mountain scene and sun. It lifts like a picked card and settles.
const liftY=F([[0,0],[.05,0],[.3,-.7,'out'],[.66,0,'smooth']],[[0,0],[.08,.3],[.26,-1.5,'launch'],[.5,0,'hit'],[.58,-.25],[.68,0]]);
const liftA=F([[0,0],[.5,0]],[[0,0],[.08,.6],[.26,-3],[.5,0,'hit'],[.58,-.5],[.68,0]]);
const thumbPose=rigid((t,s)=>({dy:liftY(t,s),angle:liftA(t,s),pivot:[4,25.5]}));
const thumbnail=make('thumbnail','Thumbnail',
 'Light: the thumbnail card lifts 0.7 and settles. Strong: it dips, lifts 1.5 with a 3° tilt about its bottom-left corner, lands with a hit and settles.',[
 ...posed(panel('card',rect(4,6.5,24,19,2.5)),thumbPose),
 ...posed(framed('mountain',polygon([[7.5,21.5],[12,14.5],[15.5,18],[18.5,15.5],[24.5,21.5]],.8)),thumbPose),
 ...posed(bothFill('sun',circle(21.5,11,1.8)),thumbPose)
],.7,.72);
thumbnail.duotoneFill=true;

// ---------- EXPORT VIDEO (gesture) ----------
// An up arrow above a clip; the arrow hops and the clip squashes under the launch (Strong).
const hop=F([[0,0],[.05,0],[.3,-1.5,'out'],[.7,0,'smooth']],[[0,0],[.1,.5],[.28,-3.5,'launch'],[.56,0,'hit'],[.66,-.5],[.78,0]]);
const squash=F([[0,0],[.5,0]],[[0,0],[.1,.6],[.2,0],[.56,.9,'hit'],[.7,0]]);
const exportVideo=make('export-video','Export video',
 'Light: the export arrow hops 1.5 and eases back onto the clip. Strong: it presses down, launches 3.5, lands with the clip squashing 0.9 and settles.',[
 ...framed('clip',(t,s)=>rect(4.5,21+squash(t,s),23,6.5-squash(t,s),2.2)),
 ...posed([...tube('arrow-stem','M16 17 L16 7.5'),...tube('arrow-head','M11.5 12 L16 7.5 L20.5 12')],rigid((t,s)=>({dy:hop(t,s)})))
],.75,.8);
exportVideo.duotoneFill=true;

// ---------- RESOLUTION (toggle sd ↔ hd) ----------
// A 2×2 frame grid subdivides to 4×4: the extra lines grow outward from the centre lines.
const gridV=R(F([[0,0],[.05,0],[.6,1,'smooth']],[[0,0],[.05,0],[.26,1,'hit']]),F([[0,1],[.05,1],[.6,0,'smooth']],[[0,1],[.2,1],[.4,0,'hit']]));
const gridH=R(F([[0,0],[.05,0],[.6,1,'smooth']],[[0,0],[.2,0],[.4,1,'hit']]),F([[0,1],[.05,1],[.6,0,'smooth']],[[0,1],[.05,1],[.26,0,'hit']]));
const gridShow=fn=>(t,s,r)=>fn(t,s,r)<.03?0:1;
const vLine=x=>tube('v-'+x,(t,s,r)=>{const l=9.5*gridV(t,s,r);return `M${x} ${16-l} L${x} ${16+l}`;});
const hLine=y=>tube('h-'+y,(t,s,r)=>{const l=11.5*gridH(t,s,r);return `M${16-l} ${y} L${16+l} ${y}`;});
const resolution=toggle('resolution','Resolution',
 'Light: the extra grid lines grow from the centre lines until the frame holds a 4×4 grid. Strong: verticals snap out first, then horizontals. Reverse retracts them to the 2×2 grid.',[
 ...panel('frame',rect(4.5,6.5,23,19,2.5)),
 ...tube('axis-v','M16 6.5 L16 25.5'),...tube('axis-h','M4.5 16 L27.5 16'),
 ...shown([...vLine(10.25),...vLine(21.75)],gridShow(gridV)),
 ...shown([...hLine(11.25),...hLine(20.75)],gridShow(gridH))
],.66,.46,'sd','hd');
resolution.duotoneFill=true;

// ---------- ASPECT RATIO (toggle wide ↔ standard) ----------
// A 13-tall frame with a corner-to-corner diagonal narrows from 23 wide (16:9) to 17 wide (4:3).
const arOut=F([[0,23],[.05,23],[.6,17,'smooth']],[[0,23],[.1,23.6],[.32,16.6,'hit'],[.46,17]]);
const arBack=F([[0,17],[.05,17],[.6,23,'smooth']],[[0,17],[.1,16.4],[.32,23.4,'hit'],[.46,23]]);
const arW=R(arOut,arBack);
const aspectRatio=toggle('aspect-ratio','Aspect ratio',
 'Light: the widescreen frame narrows to 4:3 while its diagonal follows the corners. Strong: it stretches slightly, snaps narrow with a hit and settles. Reverse widens it again.',[
 ...panel('frame',(t,s,r)=>{const w=arW(t,s,r);return rect(16-w/2,9.5,w,13,2.2);}),
 ...tube('diagonal',(t,s,r)=>{const w=arW(t,s,r);return `M${16-w/2+1.8} 11.3 L${16+w/2-1.8} 20.7`;})
],.66,.5,'wide','standard');
aspectRatio.duotoneFill=true;

// ---------- MUTE TRACK (toggle live ↔ muted) ----------
// Five waveform bars in a track panel collapse to dots on the centre line.
const BARS=[[8.75,4],[12.375,7],[16,5],[19.625,8],[23.25,3.5]];
const muteK=R(F([[0,1],[.05,1],[.5,0,'smooth']],[[0,1],[.1,1.12],[.3,0,'hit']]),F([[0,0],[.05,0],[.5,1,'smooth']],[[0,0],[.08,0],[.3,1.15,'hit'],[.46,1]]));
const muteTrack=toggle('mute-track','Mute track',
 'Light: the waveform bars sink to dots on the centre line. Strong: they swell 12% then slam flat. Reverse raises them, overshooting 15% before settling.',[
 ...panel('track',rect(4.5,7,23,18,2.5)),
 ...BARS.flatMap(([x,h],i)=>tube('bar-'+i,(t,s,r)=>{const k=muteK(t,s,r);return `M${x} ${16-h*k/2} L${x} ${16+h*k/2}`;}))
],.56,.5,'live','muted');
muteTrack.duotoneFill=true;

// ---------- VOICEOVER (gesture) ----------
// A microphone on a stand at the left; three level bars at the right rise as it speaks.
const vo=(d,l,h)=>F([[0,0],[.05+d,0],[.2+d,l,'out'],[.55+d,0,'smooth']],[[0,0],[.04+d,0],[.14+d,h,'hit'],[.26+d,h*.3],[.36+d,h*.6],[.5+d,0]]);
const LEVELS=[[19.5,3.5,2,4],[23.5,7,3,5],[27.5,4.5,2,4]];
const micBob=F([[0,0],[.5,0]],[[0,0],[.06,.4],[.16,-1.2,'hit'],[.3,.3],[.42,0]]);
const voiceover=make('voiceover','Voiceover',
 'Light: the three level bars rise in a rolling wave and settle. Strong: the microphone bobs and the bars surge 4–5 units in a staggered snap before settling.',[
 ...posed([...framed('capsule',rect(7,5,5,9.5,2.5)),...tube('cradle',arc(9.5,12.5,6,0,180)),...tube('stand','M9.5 18.5 L9.5 22.5'),...tube('base','M6.5 22.5 L12.5 22.5')],rigid((t,s)=>({dy:micBob(t,s)}))),
 ...LEVELS.flatMap(([x,b,l,h],i)=>{const g=vo(i*.06,l,h);return tube('level-'+i,(t,s)=>{const v=b+g(t,s);return `M${x} ${13.5-v/2} L${x} ${13.5+v/2}`;});})
],.75,.7);
voiceover.duotoneFill=true;

// ---------- SCREEN RECORD (toggle idle ↔ recording) ----------
// A monitor with a record ring; the record dot grows inside the ring.
const recR=R(F([[0,0],[.05,0],[.4,1.7,'smooth']],[[0,0],[.08,0],[.24,2.15,'hit'],[.4,1.7]]),F([[0,1.7],[.05,1.7],[.4,0,'smooth']],[[0,1.7],[.1,2.1],[.3,0,'hit']]));
const ringR=F([[0,3.8],[.4,3.8]],[[0,3.8],[.08,3.8],[.24,4.3,'hit'],[.4,3.8]]);
const screenRecord=toggle('screen-record','Screen record',
 'Light: the record dot grows to fill the ring. Strong: it pops past size with the ring flashing wider, then both settle. Reverse shrinks the dot away.',[
 ...panel('screen',rect(4.5,5.5,23,16,2.2)),
 ...tube('neck','M16 21.5 L16 25.5'),...tube('base','M11 25.5 L21 25.5'),
 ...tube('ring',(t,s,r)=>circle(16,13.5,r?3.8:ringR(t,s))),
 ...shown(bothFill('dot',(t,s,r)=>circle(16,13.5,Math.max(0,recR(t,s,r)))),(t,s,r)=>recR(t,s,r)<.05?0:1)
],.46,.46,'idle','recording');
screenRecord.duotoneFill=true;

// ---------- CLAPPERBOARD (gesture) ----------
// Slate body y17–28; the striped clapper (y10.5–17) hinges at its left end (5.5,17), opens and claps.
const clap=F([[0,0],[.05,0],[.3,-8,'smooth'],[.58,0,'in']],[[0,0],[.1,0],[.24,-18],[.32,-18,'hold'],[.42,0,'hit'],[.5,-2],[.58,0]]);
const slateDip=F([[0,0],[.5,0]],[[0,0],[.4,0],[.47,.6],[.6,0]]);
const clapPose=rigid((t,s)=>({angle:clap(t,s),pivot:[5.5,17]}));
const clapperboard=make('clapperboard','Clapperboard',
 'Light: the clapper opens 8° on its hinge and drops shut. Strong: it opens 18°, holds, slams shut with the slate dipping on contact and rebounds once.',[
 ...posed(framed('slate',rect(4.5,17,23,11,2),rect(4.5,16.8,23,11.2,2)),rigid((t,s)=>({dy:slateDip(t,s)}))),
 ...posed([...panel('clapper',rect(4.5,10.5,23,6.5,1.8)),...tube('stripe-0','M9 17 L12.5 10.5'),...tube('stripe-1','M15.5 17 L19 10.5'),...tube('stripe-2','M22 17 L25.5 10.5')],clapPose)
],.6,.62);
clapperboard.duotoneFill=true;

// ---------- MERGE CLIPS (toggle apart ↔ merged) ----------
// Two clips of different lengths on one track slide together and meet with a 0.1 overlap.
const mergeA=R(F([[0,0],[.05,0],[.6,2.5,'smooth']],[[0,0],[.1,-.5],[.32,2.5,'hit'],[.4,2.2],[.48,2.5]]),F([[0,2.5],[.05,2.5],[.6,0,'smooth']],[[0,2.5],[.1,3],[.32,0,'hit']]));
const mergeB=R(F([[0,0],[.05,0],[.6,-2.6,'smooth']],[[0,0],[.1,.5],[.32,-2.6,'hit'],[.4,-2.3],[.48,-2.6]]),F([[0,-2.6],[.05,-2.6],[.6,0,'smooth']],[[0,-2.6],[.1,-3.1],[.32,0,'hit']]));
const mergeClips=toggle('merge-clips','Merge clips',
 'Light: the two clips slide together and join. Strong: each draws back, drives in, bumps on contact and settles joined. Reverse pulls them apart.',[
 ...posed(framed('clip-a',rect(4.5,11.5,8.5,9,2.3)),rigid((t,s,r)=>({dx:mergeA(t,s,r)}))),
 ...posed(framed('clip-b',rect(18,11.5,9.5,9,2.3)),rigid((t,s,r)=>({dx:mergeB(t,s,r)})))
],.66,.54,'apart','merged');
mergeClips.duotoneFill=true;

// ---------- ADD CAPTION (result, preview reset) ----------
// Two caption lines type into the bottom of a frame: 14 then 8 units long.
const cap1=F([[0,0],[.05,0],[.45,14,'smooth']],[[0,0],[.04,0],[.12,5],[.18,5,'hold'],[.26,9.5],[.32,9.5,'hold'],[.4,14]]);
const cap2=F([[0,0],[.45,0],[.8,8,'smooth']],[[0,0],[.44,0],[.52,4],[.58,4,'hold'],[.66,8]]);
const capLine=(name,y,fn)=>shown(tube(name,(t,s)=>`M8 ${y} L${8+fn(t,s)} ${y}`),(t,s)=>fn(t,s)<.05?0:1);
const addCaption=make('add-caption','Add caption',
 'Light: two caption lines extend smoothly along the bottom of the frame. Strong: they type in word by word with short holds. The caption is retained; preview reset clears it.',[
 ...panel('frame',rect(4.5,5,23,22,2.5)),
 ...capLine('line-1',19.5,cap1),...capLine('line-2',23,cap2)
],.85,.72,['line-1','line-2']);
addCaption.duotoneFill=true;

// ---------- VIDEO FILTER (toggle original ↔ filtered) ----------
// A slanted wipe sweeps across the frame; the filtered region behind it is tinted (Duotone) or filled (Solid).
const wipeX=R(F([[0,10.5],[.05,10.5],[.6,24.5,'smooth']],[[0,10.5],[.1,9.7],[.34,25,'hit'],[.5,24.5]]),F([[0,24.5],[.05,24.5],[.6,10.5,'smooth']],[[0,24.5],[.1,25.3],[.34,10,'hit'],[.5,10.5]]));
const regionD=(t,s,r)=>{const w=wipeX(t,s,r);return polygon([[4.5,6.5],[w,6.5],[w-3,25.5],[4.5,25.5]],[2.5,0,0,2.5]);};
const videoFilter=toggle('video-filter','Video filter',
 'Light: the wipe line sweeps 14 units across the frame and the filtered region follows it. Strong: a short pull back, a fast sweep with a hard stop and a settle. Reverse sweeps it back.',[
 ...panel('frame',rect(4.5,6.5,23,19,2.5)),
 fill('region-interior',regionD,'tint'),fill('region-solid',regionD,'solid'),
 ...tube('wipe',(t,s,r)=>{const w=wipeX(t,s,r);return `M${w} 6.5 L${w-3} 25.5`;})
],.66,.54,'original','filtered');
videoFilter.duotoneFill=true;

// ---------- RENDER PROGRESS (result, preview reset) ----------
// A progress capsule fills a track inside the render frame, monotonically.
const prog=F([[0,9.8],[.05,9.8],[.85,22.2,'smooth']],[[0,9.8],[.06,9.8],[.2,14.1],[.26,14.1,'hold'],[.38,18.5],[.44,18.5,'hold'],[.56,22.2]]);
const renderProgress=make('render-progress','Render progress',
 'Light: the render bar fills its track steadily. Strong: it fills in three decisive chunks with short holds. The finished bar is retained; preview reset empties it.',[
 ...panel('frame',rect(4.5,4,23,24,2.5)),
 ...tube('track',rect(7.5,17.5,17,7,3.5)),
 ...shown(bothFill('bar',(t,s)=>capsule([9.8,21],[prog(t,s),21],2.8)),(t,s)=>prog(t,s)<9.9?0:1)
],.9,.6,['bar']);
renderProgress.duotoneFill=true;

const ORDER=[cutClip,trimClip,transition,speedRamp,reverseClip,rotateClip,greenScreen,stabilize,thumbnail,exportVideo,resolution,aspectRatio,muteTrack,voiceover,screenRecord,clapperboard,mergeClips,addCaption,videoFilter,renderProgress];
const fixed={'cut-clip':[],'trim-clip':[],'transition':['clip-a-frame'],'speed-ramp':['clip-frame','curve'],'reverse-clip':['clip-frame'],'green-screen':['head','shoulders'],'stabilize':['ring'],'export-video':[],'resolution':['frame-frame','axis-v','axis-h'],'mute-track':['track-frame'],'voiceover':[],'screen-record':['screen-frame','neck','base'],'add-caption':['frame-frame'],'video-filter':['frame-frame'],'render-progress':['frame-frame','track']};
const rigidParts={'cut-clip':['clip-frame','blade-handle','blade-stem'],'transition':['clip-b-frame'],'rotate-clip':['frame-frame','play'],'stabilize':['frame-frame'],'thumbnail':['card-frame','mountain-frame'],'export-video':['arrow-stem','arrow-head'],'voiceover':['capsule-frame','cradle'],'clapperboard':['clapper-frame','stripe-0','slate-frame'],'merge-clips':['clip-a-frame','clip-b-frame']};
export const icons=ORDER.map((icon,index)=>({...icon,number:886+index,constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
