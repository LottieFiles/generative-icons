// motion/icon-sets/28-text-editing/formatting.mjs (draft v2 for Codex integration)
import {make,F,tube,line,fill,stroke,bothFill,framed,rect,capsule,polygon,rigid,posed,geometricRearm,clamp,mix} from './helpers.mjs';

const drawn=(name,d,visibility)=>[
 {name,d,style:'outline',type:'stroke',width:2,visibility},
 {name:name+'-solid',d,style:'solid',type:'stroke',width:2.6,visibility}];
function drawPoly(pts,p){const seg=pts.slice(1).map((q,i)=>Math.hypot(q[0]-pts[i][0],q[1]-pts[i][1]));let rem=p*seg.reduce((a,b)=>a+b,0),cur=pts[0];const out=[cur];for(let i=0;i<seg.length;i++){const f=clamp(rem/seg[i]);if(f>0)cur=[mix(pts[i][0],pts[i+1][0],f),mix(pts[i][1],pts[i+1][1],f)];rem-=seg[i];out.push(cur);}return 'M'+out[0].join(' ')+out.slice(1).map(q=>' L'+q.join(' ')).join('');}
const open=false;

// ---------------------------------------------------------------- 1 horizontal-rule (unchanged)
const ruleP=F([[0,0],[.1,0],[.6,1,'smooth']],[[0,0],[.14,0,'hold'],[.36,1]]);
const squeeze=F([[0,0]],[[0,0],[.14,1],[.32,-.4],[.55,0]]);
const topPose=rigid((t,s)=>({dy:squeeze(t,s)})),botPose=rigid((t,s)=>({dy:-squeeze(t,s)}));
const horizontalRule=geometricRearm({...make('horizontal-rule','Horizontal rule',
 'A divider is drawn between two content blocks. Light: the rule draws quietly. Strong: the blocks press toward the gap, the rule sweeps across in one stroke, and the blocks spring back to rest.',
 [...posed([...line('top-1',[6,7],[26,7]),...line('top-2',[6,11],[20,11])],topPose),
  ...posed([...line('bottom-1',[6,21],[26,21]),...line('bottom-2',[6,25],[18,25])],botPose),
  ...drawn('rule',(t,s)=>`M6 16 L${6+20*ruleP(t,s)} 16`,(t,s)=>clamp(20*ruleP(t,s)/.8))],
 .6,.55),duotoneFill:open});

// ---------------------------------------------------------------- 2 text-color (gesture; Duotone tints the real swatch surface)
const lift=F([[0,0],[.1,0],[.34,-.8,'smooth'],[.64,0,'smooth']],[[0,0],[.12,.3],[.3,-1.8],[.48,.35],[.64,0]]);
const swatchPose=rigid((t,s)=>({dy:lift(t,s)}));
const textColor={...make('text-color','Text color',
 'Choose the foreground color of selected text. The swatch bar under the A rises to apply its color and settles back. Light: a small quiet lift. Strong: a short dip, a fast rise almost to the baseline, one bounded rebound.',
 [...tube('legs','M9.5 19.5 L15.4 6.4 C15.7 5.7 16.3 5.7 16.6 6.4 L22.5 19.5'),
  ...tube('crossbar','M12.2 14.5 L19.8 14.5'),
  ...posed(framed('swatch',rect(6,25,20,3,1.5),rect(4.7,23.7,22.6,5.6,2.8)),swatchPose)],
 .64,.64),duotoneFill:true};

// ---------------------------------------------------------------- 3 text-highlight (revised)
// Fixed: text rows y=5/9 (paint bottom ≤10.3). Output: filled band y=25, x 6→24, capsule width 3.5 (paint top 23.25).
// Tool: one rounded polygon, chisel apex at (6,24) = ink start, axis 65° up-right, body 3.6u wide, 9.5u long.
// Apex is 0.75u inside the band's painted top; body top corner (8.39,14.63) is 4.3u below the text paint.
// Pose: dx = 18·bandP (+ Strong pull-back before ink), dy = Strong lift after completion. Rest = contact for both profiles.
const bandP=F([[0,0],[.1,0],[.65,1,'smooth']],[[0,0],[.2,0,'hold'],[.42,1]]);
const pull=F([[0,0]],[[0,0],[.1,-1],[.2,0,'launch']]);
const flick=F([[0,0]],[[0,0],[.42,0,'hold'],[.5,-1.8],[.62,0]]);
const markerPose=rigid((t,s)=>({dx:18*bandP(t,s)+pull(t,s),dy:flick(t,s)}));
const markerD=polygon([[6,24],[7.86,23.1],[8.73,22.41],[11.65,16.15],[8.39,14.63],[5.47,20.88],[5.5,22]],[.5,.4,.6,1.2,1.2,.6,.4]);
const textHighlight=geometricRearm({...make('text-highlight','Text highlight',
 'A highlighter draws a band under the passage that stays. Light: one smooth pass with the tip riding the ink. Strong: the marker draws back, strikes across in one decisive stroke, and flicks up before resting on the band end.',
 [...tube('line-1','M6 5 L26 5'),...tube('line-2','M6 9 L26 9'),
  ...bothFill('band',(t,s)=>capsule([6,25],[6+18*bandP(t,s),25],3.5),{visibility:(t,s)=>clamp(18*bandP(t,s)/.8)}),
  ...posed(bothFill('marker',markerD),markerPose)],
 .65,.62),duotoneFill:open});

// ---------------------------------------------------------------- 4 clear-formatting (unchanged)
const wipe=F([[0,0],[.1,0],[.55,1,'smooth']],[[0,0],[.14,0,'hold'],[.36,1]]);
const press=F([[0,0]],[[0,0],[.12,1],[.32,-.3],[.5,0]]);
const badgePose=rigid((t,s)=>({dy:press(t,s)}));
const clearFormatting=geometricRearm({...make('clear-formatting','Clear formatting',
 'Styling is removed while the text stays. The underline beneath the T retracts and does not return. Light: quiet retraction. Strong: the clear badge presses, the underline snaps away, the badge rebounds once.',
 [...tube('crossbar','M8 7 L24 7'),...tube('stem','M16 7 L16 21'),
  ...drawn('underline',(t,s)=>`M8 25 L${18-10*wipe(t,s)} 25`,(t,s)=>clamp(10*(1-wipe(t,s))/.6)),
  ...posed([...tube('badge-a','M23.5 22 L27.5 26'),...tube('badge-b','M23.5 26 L27.5 22')],badgePose)],
 .55,.5),duotoneFill:open});

// ---------------------------------------------------------------- 5 spell-check (unchanged)
const amp=F([[0,1.4],[.1,1.4],[.5,0,'smooth']],[[0,1.4],[.12,2.1],[.32,0]]);
const chk=F([[0,0],[.3,0],[.62,1,'smooth']],[[0,0],[.26,0,'hold'],[.46,1]]);
const squiggle=(t,s)=>{const h=amp(t,s)/.75;let d='M5 20';for(let i=0;i<4;i++){const x0=5+i*2.75,x1=x0+2.75,y=20+(i%2?-h:h);d+=` C${x0+.92} ${y} ${x1-.92} ${y} ${x1} 20`;}return d;};
const checkPts=[[20.5,21.5],[23.5,24.5],[28,17.5]];
const spellCheck=geometricRearm({...make('spell-check','Spell check',
 'The misspelling underline is inspected and the correction accepted: the squiggle flattens into a plain underline and a check draws beside it. Light: quiet flatten then check. Strong: the squiggle tenses higher, snaps flat, and the check is struck in one stroke.',
 [...tube('line-1','M5 9 L27 9'),...tube('line-2','M5 14 L16 14'),
  ...drawn('squiggle',squiggle),
  ...drawn('check',(t,s)=>drawPoly(checkPts,chk(t,s)),(t,s)=>clamp(12.5*chk(t,s)/.6))],
 .65,.5),duotoneFill:open});

export const icons=[horizontalRule,textColor,textHighlight,clearFormatting,spellCheck];
