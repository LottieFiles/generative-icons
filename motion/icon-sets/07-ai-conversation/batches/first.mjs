// Claude revision accepted for native integration; render gate remains pending.
import {circle,rect,capsule,polygon,holes,arc,stroke,fill,bothFill,tube,line,rigid} from '../../library/drawing.mjs';
import {track,flow,drive,timePair,resultAction,has,clamp} from '../../library/direction.mjs';

const seg=(a,b)=>`M${a[0]} ${a[1]} L${b[0]} ${b[1]}`;
const T=(t,s,light,strong)=>drive(t,s,light,strong);

/* ---------- ai-assistant: bubble + orbit core, full rotational cycle ---------- */
const aaBody='M11 5 L21 5 C24.314 5 27 7.686 27 11 L27 17 C27 20.314 24.314 23 21 23 L13.2 23 L8.9 26.7 C8 27.5 6.6 26.9 6.6 25.7 L6.6 22.4 C5.6 21.6 5 20.4 5 19 L5 11 C5 7.686 7.686 5 11 5 Z';
const aaSpin=(t,s)=>T(t,s,[[0,0],[.04,0],[.6,360,'io']],[[0,0],[.1,-18],[.5,374],[.62,354],[.72,362],[.84,360]]);
const aaCore=(t,s)=>s?flow([[0,1.3],[.46,1.3],[.54,1.75],[.64,1.18],[.74,1.38],[.84,1.3]],t):1.3;
const aaOrbit=rigid((t,s)=>({angle:aaSpin(t,s),pivot:[16,14]}));
export const aiAssistant={
 name:'ai-assistant',label:'AI assistant',heading:'Assistant thinks, then replies',
 intent:'The orbit spins one full turn around the core; in Strong the core swells once as the answer forms.',
 description:'Speech bubble with an orbit arc around a core.',
 duotoneFill:true,cycleDegrees:360,
 parts:[
  fill('ai-assistant-body-tint',aaBody,'tint'),
  stroke('ai-assistant-body',aaBody),
  fill('ai-assistant-body-solid',holes(aaBody,circle(16,14,6.3)),'solid'),
  stroke('ai-assistant-orbit',arc(16,14,4.2,-60,240),{pose:aaOrbit}),
  {name:'ai-assistant-orbit-solid',d:arc(16,14,4,-60,240),style:'solid',type:'stroke',width:2.4,pose:aaOrbit},
  ...bothFill('ai-assistant-core',(t,s)=>circle(16,14,aaCore(t,s)))
 ],
 timings:timePair(.6,.9,[0,.15,.3,.45,.6],[0,.1,.5,.62,.74,.9]),
 brief:{light:'Orbit arc rotates 360° about (16,14) in .56s ease in-out; end pose equals start.',
  strong:'Arc winds back 14° (0–.12s), spins to 360° by .66s, holds; core swells r 1.3→1.7 (.68–.8s) and settles by 1s.',
  fixed:'Bubble body in both profiles. Moving: orbit arc (rotation), core (radius, Strong only).',
  test:'Core clearance inside orbit at peak swell: 1.25 at Bold outline, 1.1 in Solid; arc gap at top at t=0 and t=end; cycleDegrees boundary compare.'}
};

/* ---------- prompt: fixed ring, separately drawn content, stays typed ---------- */
const prPill=rect(3,8,26,16,8);
const prRing=holes(prPill,rect(5.6,10.6,20.8,10.8,5.4));
const prChev='M8.8 13.6 L11.2 16 L8.8 18.4';
const prChevSolid=polygon([[9.72,12.68],[13.04,16],[9.72,19.32],[7.88,17.48],[9.36,16],[7.88,14.52]],[1,1.2,1,1,.5,1]);
const prType=(t,s)=>T(t,s,
 [[0,0],[.13,.32,'out'],[.2,.32],[.33,.66,'out'],[.4,.66],[.56,1,'out']],
 [[0,0],[.12,0],[.20,.32,'pass'],[.28,.66,'pass'],[.38,1]]);
const prBlink=(t,s)=>s?flow([[0,1],[.04,1],[.08,0],[.14,0],[.18,1]],t):1;
const prShow=(t,s)=>clamp(prType(t,s)*12);
const prNudge=(t,s)=>s?flow([[0,0],[.36,0],[.44,.6],[.53,-.25],[.60,.1],[.66,0]],t):0;
const prMarkPose=rigid((t,s,r)=>({dx:prNudge(r?0:t,s)}));
const prCursorPose=rigid((t,s)=>({dx:4*prType(t,s)}));
const prTypedEnd=(t,s)=>15.4+3.8*prType(t,s);
export const prompt={
 name:'prompt',label:'Prompt',heading:'User types a prompt',
 intent:'The cursor blinks, text is typed in three chunks, the prompt mark nudges as it submits.',
 description:'Pill input with a chevron prompt mark and a bar cursor.',
 duotoneFill:true,
 parts:[
  fill('prompt-field-tint',prPill,'tint'),
  stroke('prompt-field',prPill),
  fill('prompt-field-solid',prRing,'solid'),
  stroke('prompt-mark',prChev,{pose:prMarkPose}),
  fill('prompt-mark-solid',prChevSolid,'solid',{pose:prMarkPose}),
  stroke('prompt-cursor','M19.2 13.8 L19.2 18.2',{pose:prCursorPose,visibility:prBlink}),
  fill('prompt-cursor-solid',capsule([19.2,13.8],[19.2,18.2],2.6),'solid',{pose:prCursorPose,visibility:prBlink}),
  stroke('prompt-typed',(t,s)=>seg([15.2,16],[prTypedEnd(t,s),16]),{visibility:prShow}),
  fill('prompt-typed-solid',(t,s)=>capsule([15.2,16],[prTypedEnd(t,s),16],2.6),'solid',{visibility:prShow})
 ],
 timings:timePair(.6,1,[0,.13,.33,.56,.6],[0,.08,.18,.28,.42,.58,.74,1]),
 brief:{light:'Typed line grows 15.2→19.2 in three eased chunks; cursor advances 4 units ahead of it, .56s.',
  strong:'Cursor blinks off/on by visibility (.07–.24s), same three chunks (.28–.86s), chevron nudges right .6 and returns (.84–1s).',
  fixed:'Pill outline and Solid ring. Moving: cursor (dx, visibility), typed line (length, visibility), chevron (dx, Strong only).',
  test:'Solid clearances: chevron end to window .84, cursor top to window .99, chevron tip to text ~1.1, text end to cursor 1.4; blink keeps cursor size; reset fades only cursor and typed line, ring stays.'}
};
resultAction(prompt,p=>has(p,'prompt-cursor','prompt-typed'));

/* ---------- system-instructions: directive tree, spine descends, rule writes ---------- */
const siNode=rect(5,4,8,8,2.5);
const siExt=(t,s)=>T(t,s,[[0,0],[.2,1,'out']],[[0,0],[.04,0],[.2,1]]);
const siWrite=(t,s)=>T(t,s,[[0,0],[.2,0],[.52,1,'out']],[[0,0],[.16,0],[.24,.55,'pass'],[.38,1]]);
// One elbow path: vertical leg + 1.5 corner grow with e, horizontal leg grows with w.
const siRule3=(t,s)=>{const e=siExt(t,s),w=siWrite(t,s),y=21.5+4.5*e;
 return `M9 21.5 L9 ${21.5+3*e} C9 ${21.5+3.83*e} ${9+.67*e} ${y} ${9+1.5*e} ${y} L${9+1.5*e+16.5*w} ${y}`;};
export const systemInstructions={
 name:'system-instructions',label:'System instructions',heading:'A rule is added under the root',
 intent:'The spine extends one level below the last rule, then a third rule writes out and stays.',
 description:'Root node with a spine and rules branching right, a directive tree.',
 duotoneFill:true,
 parts:[
  fill('system-instructions-node-tint',siNode,'tint'),
  stroke('system-instructions-node',siNode),
  fill('system-instructions-node-solid',siNode,'solid'),
  ...line('system-instructions-spine',[9,12],[9,21.5]),
  ...line('system-instructions-rule-1',[9,17],[27,17]),
  ...line('system-instructions-rule-2',[9,21.5],[27,21.5]),
  ...tube('system-instructions-rule-3',siRule3)
 ],
 timings:timePair(.55,.7,[0,.1,.2,.36,.52,.55],[0,.04,.2,.34,.38,.56,.7]),
 brief:{light:'Spine descends 4.5 into a rounded elbow (0–.2s), rule 3 writes to x 27 (.2–.52s), hold.',
  strong:'Spine descends with ease in-out (.06–.32s), holds; rule writes in two chunks (.38–.58s, .66–.9s), hold to .98s.',
  fixed:'Node, spine, rules 1–2 in every style. Moving: rule 3 elbow path (point growth only).',
  test:'Degenerate elbow at t=0 is a round dot at (9,21.5) fully inside the spine/rule-2 junction in all weights; rule spacing 4.5 at Bold 24px; silhouette reads as hierarchy, not Document/Clipboard; reset fades only rule 3.'}
};
resultAction(systemInstructions,p=>has(p,'rule-3'));

/* ---------- context-window: fixed frame, retained advance inside a content region ---------- */
const cwFrame=rect(4,7,24,18,4);
const cwSolid=holes(cwFrame,rect(6.6,9.6,18.8,12.8,2));
const cwPitch=5.5;
// Content region x 8..24, y 11.7..20.3 (Solid window 6.6..25.4 / 9.6..22.4). Slots at x 10.5, 16, 21.5.
const cwMove=(t,s,row)=>-cwPitch*T(t,s,[[0,0],[.12,0],[.45,1,'io']],row?[[0,0],[.22,0],[.4,1.045],[.48,.982],[.56,1]]:[[0,0],[.14,0],[.32,1.045],[.4,.982],[.48,1]]);
const cwLeave=(t,s)=>T(t,s,[[0,1],[.12,0,'out']],[[0,1],[.1,0]]);
const cwArrive=(t,s,row)=>T(t,s,[[0,0],[.45,0],[.55,1,'out']],row?[[0,0],[.4,0],[.5,1]]:[[0,0],[.32,0],[.42,1]]);
const cwToken=(name,cx,y,len,extra)=>[
 stroke(name,seg([cx-len/2,y],[cx+len/2,y]),extra),
 fill(name+'-solid',capsule([cx-len/2,y],[cx+len/2,y],2.6),'solid',extra)];
const cwRow=(row,y,lens,incoming)=>{const slide={pose:rigid((t,s)=>({dx:cwMove(t,s,row)}))};return[
 ...cwToken(`context-window-token-${row}-out`,10.5,y,lens[0],{visibility:cwLeave}),
 ...cwToken(`context-window-token-${row}-a`,16,y,lens[1],slide),
 ...cwToken(`context-window-token-${row}-b`,21.5,y,lens[2],slide),
 ...cwToken(`context-window-token-${row}-in`,21.5,y,incoming,{visibility:(t,s)=>cwArrive(t,s,row)})];};
export const contextWindow={
 name:'context-window',label:'Context window',heading:'The window advances over the text',
 intent:'The oldest token drops out, the remaining tokens shift one slot left, a new token arrives on the right.',
 description:'Fixed frame over two rows of token dashes.',
 duotoneFill:true,
 parts:[
  fill('context-window-frame-tint',cwFrame,'tint'),
  stroke('context-window-frame',cwFrame),
  fill('context-window-frame-solid',cwSolid,'solid'),
  ...cwRow(0,13,[2,.8,1.6],1.2),
  ...cwRow(1,19,[1.2,2,.8],2)
 ],
 timings:timePair(.55,.7,[0,.12,.3,.45,.55],[0,.1,.22,.32,.42,.56,.7]),
 brief:{light:'Leftmost tokens fade in place (0–.12s), both rows slide 5.5 left ease in-out (.12–.45s), new tokens fade in at the right slot (.45–.55s).',
  strong:'Leftmost tokens fade (0–.14s); row 1 slides .16–.5s, row 2 follows .28–.62s; new tokens arrive .55–.67s and .68–.8s; hold to .9s.',
  fixed:'Frame in every style. Moving: two sliding tokens per row (dx), outgoing and incoming tokens (visibility only).',
  test:'No token touches the frame stroke at any frame; end pose visibly differs from start (row 1 lengths [.8,1.6,1.2] vs [2,.8,1.6]); reset fades tokens only, frame stays; Bold neighbour gap 1.0.'}
};
resultAction(contextWindow,p=>has(p,'token'));

/* ---------- response-stream: unchanged from the accepted proposal ---------- */
const rsPane='M14 6 L9 6 C6.239 6 4 8.239 4 11 L4 21 C4 23.761 6.239 26 9 26 L14 26';
const rsFlow=(t,s)=>T(t,s,
 [[0,0],[.14,.35,'out'],[.22,.35],[.36,.7,'out'],[.44,.7],[.58,1,'out']],
 [[0,0],[.1,0],[.20,.45,'pass'],[.29,.75,'pass'],[.42,1]]);
const rsBlink=(t,s)=>s?flow([[0,1],[.02,1],[.06,0],[.12,0],[.16,1],[.66,1],[.7,0],[.76,0],[.8,1]],t):1;
export const responseStream={
 name:'response-stream',label:'Response stream',heading:'Tokens stream out of the pane',
 intent:'The cursor blinks, the third line arrives in three chunks with the cursor leading, then idles.',
 description:'Open C pane with text lines flowing out and a block cursor.',
 duotoneFill:false,
 parts:[
  ...tube('response-stream-pane',rsPane),
  ...line('response-stream-line-1',[9,11.5],[24,11.5]),
  ...line('response-stream-line-2',[9,16],[24,16]),
  stroke('response-stream-line-3',(t,s)=>seg([9,20.5],[10.5+9.5*rsFlow(t,s),20.5])),
  fill('response-stream-line-3-solid',(t,s)=>capsule([9,20.5],[10.5+9.5*rsFlow(t,s),20.5],2.6),'solid'),
  ...bothFill('response-stream-cursor',rect(12.9,18.2,2.2,4.6,1.1),{pose:rigid((t,s)=>({dx:9.5*rsFlow(t,s)})),visibility:rsBlink})
 ],
 timings:timePair(.6,.85,[0,.14,.36,.58,.6],[0,.06,.16,.28,.44,.6,.7,.85]),
 brief:{light:'Line 3 grows 10.5→20 in three eased chunks; block cursor leads with a constant 3.5 gap, .58s.',
  strong:'Cursor prepares, then output advances in brisk connected chunks. The cursor ends visible; authored curves define current timing.',
  fixed:'Pane, lines 1–2. Moving: line 3 (length), cursor (dx + visibility).',
  test:'Duotone must equal Outline (no closed surface); the open pane must not read as a bracket glyph at 24px; optical mass versus folder/check since this is the sparsest of the five.'}
};
resultAction(responseStream,p=>has(p,'line-3','response-stream-cursor'));

export const icons=[aiAssistant,prompt,systemInstructions,contextWindow,responseStream];
