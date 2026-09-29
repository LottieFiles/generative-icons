// Text & document editing — batch: paragraph, text-alignment, bulleted-list, numbered-list, indent-outdent
import {make,toggle,geometricRearm,line,tube,bothFill,fill,circle,arc,rigid,F} from './helpers.mjs';

const D=(light,strong,delay=0)=>(t,s)=>F(light,strong)(t-delay,s);
const hline=(name,y,x0,x1,extra)=>line(name,[x0,y],[x1,y],extra);

/* 1. PARAGRAPH — pilcrow. All parts move together as one rigid glyph (carriage-return dip).
   Fixed: nothing. Pivot: pure translation (dy). The typographic counter stays unfilled. */
const bowlArc=arc(13.5,10.5,4.5,-90,-270).replace(/^M[^C]*C/,' C');   // top (13.5,6) → left → bottom (13.5,15)
const bowlOpen=`M23 6 L13.5 6${bowlArc} L18 15`;                        // bar + bowl, one M
const bowlClosed=`M18 6 L13.5 6${bowlArc} L18 15 Z`;                     // tint surface only
const pDip=rigid((t,s)=>({dy:F([[0,0],[.22,1.2,'smooth'],[.5,0,'smooth']],
                              [[0,0],[.12,-1.4],[.3,2.6,'hit'],[.42,-.5],[.55,0]])(t,s)}));
const paragraph={...make('paragraph','Paragraph',
 'The pilcrow lifts slightly, drops like a carriage return, and settles in place.',
 [...tube('bowl',bowlOpen,{pose:pDip}),
  ...line('stem-a',[18,6],[18,25],{pose:pDip}),
  ...line('stem-b',[23,6],[23,25],{pose:pDip})],.5,.6),duotoneFill:false};
// Sweep: y 3.3..28.9 (Solid caps), x 7.7..24.3.

/* 2. TEXT ALIGNMENT — toggle left ↔ right. Fixed: full lines y8/y18. Moving: short lines y13/y23 (dx 0→8).
   Supported source states: left dx0, center dx4, right dx8, justified = all full (not exported as rows). */
const alignKeys=[[[0,0],[.45,1,'smooth']],[[0,0],[.1,-.1],[.34,1.12,'hit'],[.5,1]]];
const slide=delay=>rigid((t,s,r)=>{const p=D(...alignKeys,delay)(t,s);return{dx:8*(r?1-p:p)};});
const textAlignment={...toggle('text-alignment','Text alignment',
 'Short lines slide from the left edge to the right edge; Strong pulls back first and lands with a checked overshoot.',
 [ ...hline('full-a',8,6,26), ...hline('full-b',18,6,26),
   ...hline('short-a',13,6,18,{pose:slide(0)}),
   ...hline('short-b',23,6,18,{pose:slide(.06)}) ],
 .55,.6,'left','right'),duotoneFill:false};

/* 3. BULLETED LIST — result output. Fixed: three text lines x12..26. Moving: three bullet disks slide in from the left. */
const dotKeys=[[[0,0],[.35,1,'smooth']],[[0,0],[.16,1.15,'hit'],[.28,.94],[.38,1]]];
const dotPose=delay=>rigid((t,s)=>({dx:-2.8*(1-D(...dotKeys,delay)(t,s))}));
const dotVis=delay=>(t,s,r)=>Math.max(0,Math.min(1,(t-delay)/.1));
const bullet=(name,y,delay)=>bothFill(name,circle(6.5,y,1.6),{pose:dotPose(delay),visibility:dotVis(delay)});
const bulletedList={...geometricRearm(make('bulleted-list','Bulleted list',
 'Bullets enter from the left in order and hold; Strong pops each dot with a single overshoot.',
 [ ...hline('line-a',9,12,26), ...hline('line-b',16,12,26), ...hline('line-c',23,12,26),
   ...bullet('dot-a',9,0), ...bullet('dot-b',16,.08), ...bullet('dot-c',23,.16) ],
 .55,.55)),duotoneFill:false};
// Rest gap: dot right edge 8.1 (overshoot 8.5) vs line cap 10.7 → ≥2.2. Left sweep ≥2.1.

/* 4. NUMBERED LIST — result output. Fixed: two lines x15..27. Moving: vector numerals 1 and 2 drop from above. */
const numKeys=[[[0,0],[.3,1,'smooth']],[[0,0],[.14,1.18,'hit'],[.26,.95],[.34,1]]];
const numPose=(delay,offset=0)=>rigid((t,s)=>({dy:offset-(s?1.8:.8)*(1-D(...numKeys,delay)(t,s))}));
const one='M5.5 8 L8 6 L8 12';
const two=`M5 9.5${arc(8,9.5,3,180,360).replace(/^M[^C]*C/,' C')} L5 14 L11 14`;  // open top arc, diagonal, foot
const numberedList={...geometricRearm(make('numbered-list','Numbered list',
 'Numerals 1 then 2 drop into their rows and hold; Strong lands each with a brief rebound.',
 [ ...hline('line-a',9,15,27), ...hline('line-b',22,15,27),
   ...tube('one',one,{pose:numPose(0),visibility:dotVis(0)}),
   ...tube('two',two,{pose:numPose(.14,11.5),visibility:dotVis(.14)}) ],
 .5,.5)),duotoneFill:false};
// Numeral 2 diagonal-to-arc-cap painted gap ≈1.0 at Solid; numeral cap 12.3 vs line cap 13.7.

/* 5. INDENT / OUTDENT — toggle indent ↔ outdent. Fixed: full lines y7/y25.
   Moving: arrow (shaft+head, rigid; pivot [8,16], turns 180° and moves dx −2.5), short lines y12/y20 (dx −4).
   Order: forward = arrow turns first, then lines move into the vacated channel; reverse = lines first, then arrow. */
const arrowFwd=[[[0,0],[.35,1,'smooth']],[[0,0],[.08,-.08],[.3,1.06,'hit'],[.4,1]]];
const linesFwd=[[[.2,0],[.55,1,'smooth']],[[.22,0],[.42,1.06,'hit'],[.55,1]]];
const linesRev=[[[0,1],[.35,0,'smooth']],[[0,1],[.06,1.06],[.26,-.06,'hit'],[.36,0]]];
const arrowRev=[[[.2,1],[.55,0,'smooth']],[[.2,1],[.42,-.05,'hit'],[.55,0]]];
const arrowPose=rigid((t,s,r)=>{const a=(r?D(...arrowRev):D(...arrowFwd))(t,s);return{angle:180*a,pivot:[8,16],dx:-2.5*a};});
const shortPose=rigid((t,s,r)=>{const p=(r?D(...linesRev):D(...linesFwd))(t,s);return{dx:-4*p};});
const indentOutdent={...toggle('indent-outdent','Indent / outdent',
 'The guide arrow turns back and steps aside, then the indented lines slide out to the margin; reverse restores the indent.',
 [ ...hline('full-a',7,6,26), ...hline('full-b',25,6,26),
   ...hline('short-a',12,15.5,26,{pose:shortPose}), ...hline('short-b',20,15.5,26,{pose:shortPose}),
   ...tube('arrow-shaft','M6 16 L10 16',{pose:arrowPose}),
   ...tube('arrow-head','M7.5 13.5 L10 16 L7.5 18.5',{pose:arrowPose}) ],
 .55,.55,'indented','outdented'),duotoneFill:false};

export const icons=[paragraph,textAlignment,bulletedList,numberedList,indentOutdent];
