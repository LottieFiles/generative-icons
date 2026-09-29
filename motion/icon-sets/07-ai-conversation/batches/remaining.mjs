// Claude revised draft accepted for native integration; render review remains pending.
import {make,F as P,fill,stroke,line,tube,framed,bothFill,circle,rect,capsule,hole,holes,annulus,arc,polygon,rigid,posed,shown,track} from '../../library/ai-authoring.mjs';
import {flow} from '../../library/direction.mjs';
const noTint=i=>Object.assign(i,{duotoneFill:false});
const shift=(fx,fy=()=>0)=>rigid((t,s)=>({dx:fx(t,s),dy:fy(t,s)}));
const turn=(fa,pivot)=>rigid((t,s)=>({angle:fa(t,s),pivot}));
const lerp=(a,b,u)=>[a[0]+(b[0]-a[0])*u,a[1]+(b[1]-a[1])*u];
// First u of one cubic, topology preserved (still M + one C).
const cut=(p,u)=>{const m=(A,B)=>lerp(A,B,u),q=[m(p[0],p[1]),m(p[1],p[2]),m(p[2],p[3])],r=[m(q[0],q[1]),m(q[1],q[2])];return `M${p[0]} C${q[0]} ${r[0]} ${lerp(r[0],r[1],u)}`;};

// 1 Autocomplete: typed right 13 / caret 16.5 / ghost 20 give constant 1.0 Bold gaps; ghost clears before travel.
const ac=(()=>{
 const adv=P([[0,0],[.22,0],[.58,10,'out']],[[0,0],[.44,0],[.64,9.5],[.76,10]]);
 const car=(t,s)=>s?flow([[0,0],[.44,0],[.62,10.8],[.72,9.9],[.8,10.1],[.88,10]],t):adv(t,s);
 const ghost=P([[0,.35],[.10,.35],[.22,0,'out']],[[0,.35],[.14,.35],[.28,.7],[.42,0]]);
 const blink=(t,s)=>s?flow([[0,1],[.08,0],[.18,1],[.86,1],[.9,0],[.98,1]],t):1;
 const typed=(t,s)=>rect(4,12,9+adv(t,s),8,4);
 return make('autocomplete','Autocomplete','The ghost suggestion clears, then the typed text and caret advance together to accept it',[
  ...framed('typed',typed,typed),
  ...posed(line('caret',[16.5,8],[16.5,24],{visibility:blink}),shift(car)),
  ...shown(framed('ghost',rect(20,12,8,8,4)),ghost)],.70,1.00,['typed','caret','ghost']);
})();

// 2 Rewrite: detached arrow, tip (13,7.75); bar top 20 leaves 1.0 to the loop start at Bold.
const rw=(()=>{
 const loop=[[22,16.5],[22,10],[18.5,7.75],[13,7.75]],arms=[[15.25,5.5],[15.25,10]];
 const right=P([[0,21],[.10,21],[.38,9.5,'in'],[.42,9.5],[.66,26,'out']],[[0,21],[.12,21],[.34,9.5],[.5,9.5],[.62,17],[.66,17],[.82,26]]);
 const c=P([[0,1]],[[0,1],[.14,1],[.30,0],[.34,0],[.48,1]]);        // curve fraction
 const h=P([[0,1]],[[0,1],[.04,1],[.14,0],[.46,0],[.56,1.15],[.64,.95],[.72,1]]);        // head unfold
 const vis=P([[0,1]],[[0,1],[.26,1],[.30,0,'in'],[.34,0],[.38,1,'out']]);      // lift-off, no held dot
 const arrow=(t,s)=>{const u=c(t,s),m=(A,B)=>lerp(A,B,u),q=[m(loop[0],loop[1]),m(loop[1],loop[2]),m(loop[2],loop[3])],r=[m(q[0],q[1]),m(q[1],q[2])],E=lerp(r[0],r[1],u),k=u<1?0:h(t,s),A=arms.map(a=>lerp(E,a,k));
  return `M${loop[0]} C${q[0]} ${r[0]} ${E} L${A[0]} L${E} L${A[1]}`;};
 return make('rewrite','Rewrite','Text erases and retypes longer; Strong first withdraws and redraws the rewrite arrow before typing in two passes',[
  ...framed('bar',(t,s)=>rect(4,20,right(t,s)-4,5.5,2.75)),
  ...tube('arrow',arrow,{visibility:vis})],.75,.90,['bar']);
})();

// 3 Summarize: rows 5/10/15/20, chevron 23.5..26.5; gather ±0.75, dip 0.75 (max paint 28.5).
const sm=(()=>{
 const ys=[5,10,15,20],ends=[18,22,14,19];
 const end=i=>P([[0,27],[.10+i*.06,27],[.42+i*.06,ends[i],'out']],[[0,27],[.3+i*.03,27],[.45+i*.03,ends[i]]]);
 const gy=P([[0,0]],[[0,0],[.16,1.2],[.32,-.3],[.42,0]]),dip=P([[0,0]],[[0,0],[.5,0],[.58,1.5],[.68,-.4],[.76,.15],[.84,0]]);
 return noTint(make('summarize','Summarize','Lines contract to an uneven summary above a fixed chevron; Strong gathers the outer lines first and the chevron dips once',[
  ...ys.flatMap((y,i)=>{const e=end(i),g=i===0?1:i===3?-1:0;return tube('l'+(i+1),(t,s)=>{const yy=y+g*gy(t,s);return `M5,${yy} L${e(t,s)},${yy}`;});}),
  ...posed(tube('chev','M12,23.5 L16,26.5 L20,23.5'),shift(()=>0,dip))],.65,.86,['l1','l2','l3','l4']));
})();

// 4 Translate: glyphs stable; underline arrow 5.5..25.5 at y24, head arms unfold from the shaft.
const tr=(()=>{
 const e=P([[0,11.5],[.10,11.5],[.48,25.5,'out']],[[0,11.5],[.12,8.5],[.46,26.6],[.58,25.1],[.68,25.6],[.78,25.5]]);
 const h=P([[0,0],[.40,0],[.62,1,'out']],[[0,0],[.4,0],[.54,1.2],[.64,.9],[.74,1]]);
 const arrow=(t,s)=>{const x=e(t,s),k=2.5*h(t,s);return `M5.5,24 L${x},24 L${x-2.5},${24-k} L${x},24 L${x-2.5},${24+k}`;};
 return noTint(make('translate','Translate','A and 文 stay put; an underline arrow draws from the source glyph to the target and opens its head',[
  ...tube('a','M5.5,18 L9.5,7 L13.5,18'),...line('abar',[7.2,14],[11.8,14]),
  ...line('wbar',[18.5,7],[26.5,7]),...line('w1',[19.5,10.5],[25.5,18]),...line('w2',[25.5,10.5],[19.5,18]),
  ...tube('arrow',arrow)],.70,.90,['arrow']));
})();

// 5 Brainstorm: dome r5.5 at (16,13.5); ray inner ends 1.0 clear of the wall; max length 3.5 keeps 3.5..28.5.
const bs=(()=>{
 const bulb='M16,8 C12.96,8 10.5,10.46 10.5,13.5 C10.5,16 12.5,17.5 13,20 L13,23 C13,24.1 13.9,25 15,25 L17,25 C18.1,25 19,24.1 19,23 L19,20 C19.5,17.5 21.5,16 21.5,13.5 C21.5,10.46 19.04,8 16,8 Z';
 const rays=[['rw',[7,13.5],[-1,0]],['rnw',[9.64,7.14],[-.7071,-.7071]],['rne',[22.36,7.14],[.7071,-.7071]],['re',[25,13.5],[1,0]]];
 const len=i=>P([[0,1.5],[.10,1.5],[.50,3,'out']],[[0,1.5],[.14,.5],[.16+i*.04,.5],[.36+i*.04,3.5],[.48+i*.04,2.7],[.6+i*.04,3.1],[.72+i*.04,3]]);
 return make('brainstorm','Brainstorm','Rays stretch from the bulb; Strong draws them in, bursts them in sequence, then settles',[
  ...framed('bulb',bulb,hole(bulb,rect(14.25,21,3.5,2,1))),stroke('thread','M13,22 L19,22'),
  ...rays.flatMap(([n,p,d],i)=>{const L=len(i);return tube(n,(t,s)=>{const l=L(t,s);return `M${p} L${p[0]+d[0]*l},${p[1]+d[1]*l}`;});})],.60,.90,['rw','rnw','rne','re']);
})();

// 6 Reasoning: rings r3.75, beads r1.6 hop on an outward bow (3.75 Light, 4.5 Strong) so transit is visible off the link.
const rs=(()=>{
 const N=[[9,21.5],[16,9],[23,21.5]];
 const hop=(a,b,u,k)=>{const d=[b[0]-a[0],b[1]-a[1]],l=Math.hypot(...d),w=k*Math.sin(Math.PI*u);return [a[0]+d[0]*u+d[1]/l*w,a[1]+d[1]*u-d[0]/l*w];};
 const u1=P([[0,0],[.10,0],[.36,1,'io']],[[0,0],[.06,0],[.29,1]]),u2=P([[0,0],[.40,0],[.66,1,'io']],[[0,0],[.31,0],[.54,1]]);
 const bead=legs=>rigid((t,s)=>{const k=s?5.5:3.75,a=u1(t,s),b=legs>1?u2(t,s):0,p=b>0?hop(N[1],N[2],b,k):hop(N[0],N[1],a,k);return {dx:p[0]-N[0][0],dy:p[1]-N[0][1]};});
 const link=(a,b)=>{const l=Math.hypot(b[0]-a[0],b[1]-a[1]);return `M${lerp(a,b,3.75/l)} L${lerp(a,b,1-3.75/l)}`;};
 return make('reasoning','Reasoning','Thought hops from the first node to the second and on to the third, leaving every visited node marked; Strong makes a broad, continuous hop through the visited nodes',[
  ...N.flatMap((c,i)=>framed('n'+(i+1),circle(c[0],c[1],3.75),annulus(c[0],c[1],3.75,2.6))),
  ...tube('k1',link(N[0],N[1])),...tube('k2',link(N[1],N[2])),
  ...bothFill('d1',circle(9,21.5,1.6)),
  ...posed(bothFill('t1',circle(9,21.5,1.6)),bead(1)),...posed(bothFill('t2',circle(9,21.5,1.6)),bead(2))],.70,.95,['t1','t2']);
})();

// 7 Branch: fork (16,15.5), leaves r3.5 at y23.5; -14° peak keeps right paint 29.5, +4° keeps bottom 28.7.
const cb=(()=>{
 const F=[16,15.5];
 const ra=P([[0,0],[.10,0],[.50,-12,'io']],[[0,0],[.06,0],[.2,5],[.48,-14],[.6,-10.5],[.7,-12.6],[.8,-12]]);
 const la=P([[0,0]],[[0,0],[.06,0],[.2,6],[.44,6],[.62,-1.5],[.74,.5],[.84,0]]);
 const dot=P([[0,0],[.38,0],[.55,1,'out']],[[0,0],[.46,0],[.56,1]]);
 const leaf=(n,x)=>framed(n,circle(x,23.5,3.5),annulus(x,23.5,3.5,2.6));
 return make('conversation-branch','Conversation branch','A reply branch swings outward from the fork and activates; Strong lets the other branch yield first',[
  ...framed('root',rect(10,5,12,6,3)),...tube('stem','M16,11 L16,15.5'),
  ...posed([...tube('la','M16,15.5 C16,18.5 9,17 9,20'),...leaf('ll',9)],turn(la,F)),
  ...posed([...tube('ra','M16,15.5 C16,18.5 23,17 23,20'),...leaf('rl',23),...shown(bothFill('rd',circle(23,23.5,1.25)),dot)],turn(ra,F))],.60,.90,['ra','rl','rd']);
})();

// 8 Template: brackets 5..8.5 / 23.5..27, slot 12.5..19.5 gives 1.5 clearance at Bold; spread ±1.2 stays 2.55..29.45.
const pt=(()=>{
 const end=P([[0,13],[.10,13],[.50,19.5,'out']],[[0,13],[.28,13],[.44,19.5]]);
 const spread=P([[0,0]],[[0,0],[.04,0],[.18,1.5],[.34,1.2],[.46,1.2],[.62,-.15],[.73,0]]);
 return noTint(make('prompt-template','Prompt template','A slot fills inside the brackets; Strong parts the brackets before the fill and closes them after',[
  ...line('top',[5,8],[27,8]),...line('bottom',[5,26],[19,26]),
  ...posed(tube('lb','M8.5,12.5 L7,12.5 C5.9,12.5 5,13.4 5,14.5 L5,19.5 C5,20.6 5.9,21.5 7,21.5 L8.5,21.5'),shift((t,s)=>-spread(t,s))),
  ...posed(tube('rb','M23.5,12.5 L25,12.5 C26.1,12.5 27,13.4 27,14.5 L27,19.5 C27,20.6 26.1,21.5 25,21.5 L23.5,21.5'),shift(spread)),
  ...tube('slot',(t,s)=>`M12.5,17 L${end(t,s)},17`)],.60,.80,['slot']));
})();

// 9 Tone: arc r9.5, ticks r13..14.5, needle length 6 (1.0 clear of arc), cap buried in the r2 pivot. No tint.
const tn=(()=>{
 const ang=P([[0,0],[.10,0],[.55,70,'io']],[[0,0],[.08,0],[.16,-10],[.44,76],[.56,66],[.66,71.5],[.76,70]]);
 return noTint(make('tone-control','Tone control','The needle swings to the chosen setting; Strong winds back slightly, then travels and stops exactly on it',[
  ...tube('arc',arc(16,21,9.5,180,360)),
  ...line('t1',[6.81,11.81],[5.75,10.75]),...line('t2',[16,8],[16,6.5]),...line('t3',[25.19,11.81],[26.25,10.75]),
  ...line('base',[8,26.5],[24,26.5]),
  ...posed(line('needle',[16,21],[11.76,16.76]),turn(ang,[16,21])),...bothFill('pivot',circle(16,21,2))],.60,.85,['needle']));
})();

// 10 Compare: Solid tiles are 2.5 rings with a 5.5x15 counter; text is separate capsules; no alpha, only lift/sink.
const cp=(()=>{
 const ry=P([[0,0],[.10,0],[.45,-2,'io']],[[0,0],[.06,0],[.16,.6],[.38,-2.7],[.5,-1.6],[.6,-2.15],[.7,-2]]);
 const ly=P([[0,0],[.10,0],[.45,1,'io']],[[0,0],[.06,0],[.16,.6],[.38,1.4],[.5,.8],[.6,1.05],[.7,1]]);
 const tile=(n,x,rows)=>[...framed(n,rect(x,6,10.5,20,3.5),hole(rect(x,6,10.5,20,3.5),rect(x+2.5,8.5,5.5,15,1.5))),
  ...rows.flatMap((y,i)=>[stroke(n+'-r'+i,`M${x+3.5},${y} L${x+7},${y}`),fill(n+'-r'+i+'-solid',capsule([x+4.25,y],[x+6.25,y],2.6))])];
 return make('answer-compare','Answer compare','Two candidate answers; the chosen one lifts while the other settles lower. Strong weighs both first',[
  ...posed(tile('lt',4,[12.5,19.5]),shift(()=>0,ly)),...posed(tile('rt',17.5,[11.5,16,20.5]),shift(()=>0,ry))],.55,.85,['lt','rt']);
})();

// 11 Follow-up: polygon fillets round the tail (tip 9.5,19); shaft x5.5 keeps 1.5 from the tail and 1.0 below the bubble.
const fu=(()=>{
 const bub=polygon([[5,4.5],[27,4.5],[27,15],[14,15],[9.5,19],[9.5,15],[5,15]],[3,3,3,1,.8,1,3]);
 const ext=P([[0,0],[.10,0],[.50,7,'out']],[[0,0],[.3,0],[.48,8.5],[.58,6.5],[.66,7.3],[.74,7]]);
 const dots=[12,16,20].map((x,i)=>[x,P([[0,0]],[[0,0],[.02+i*.07,0],[.09+i*.07,-1.8],[.17+i*.07,.3],[.23+i*.07,0]])]);
 const arrow=(t,s)=>{const x=17+ext(t,s);return `M5.5,18.5 L5.5,22.5 C5.5,24.16 6.84,25.5 8.5,25.5 L${x},25.5 L${x-2},23.5 L${x},25.5 L${x-2},27.5`;};
 return make('follow-up','Follow-up','The thread arrow under the bubble extends to ask again; Strong first bounces the typing dots in turn',[
  ...framed('bub',bub,(t,s)=>holes(bub,...dots.map(([x,b])=>circle(x,9.75+b(t,s),1.5)))),
  ...dots.flatMap(([x,b],i)=>posed([fill('d'+i,circle(x,9.75,1.2),'outline')],shift(()=>0,b))),
  ...tube('arrow',arrow)],.60,.85,['arrow']);
})();

// 12 Intent: bubble r5.5 with fillet cubics at both tail joins and tip; ticks 6.5..4.5 move in 0.75 (gap 0.75), out 0.5 (paint 2.75).
const it=(()=>{
 const bub='M10.66,18.87 C10.73,18.48 10.97,18.26 10.83,17.88 '+arc(16,16,5.5,160,473).replace(/^M[^C]*/,'')+' C13.39,20.87 13.34,21.34 13,21.71 L10.04,24.91 C9.73,25.24 9.56,25.15 9.64,24.71 Z';
 const inw=P([[0,0],[.10,0],[.42,.75,'out']],[[0,0],[.04,0],[.12,-.5],[.32,.75],[.43,.65],[.54,.75]]);
 const rad=P([[0,.01],[.30,.01],[.50,1.5,'out']],[[0,.01],[.26,.01],[.38,1.8],[.49,1.35],[.61,1.5]]);
 const ticks=[['tn',[16,6.5],[16,4.5],[0,1]],['te',[25.5,16],[27.5,16],[-1,0]],['tw',[6.5,16],[4.5,16],[1,0]],['ts',[16,25.5],[16,27.5],[0,-1]]];
 return make('intent-detection','Intent detection','Reticle ticks close on the message and a lock dot grows; Strong flares outward first and pops the dot',[
  ...framed('bub',bub,hole(bub,circle(16,16,2.75))),
  ...bothFill('dot',(t,s)=>circle(16,16,rad(t,s))),
  ...ticks.flatMap(([n,a,b,d])=>posed(line(n,a,b),shift((t,s)=>d[0]*inw(t,s),(t,s)=>d[1]*inw(t,s))))],.55,.85,['tn','te','tw','ts','dot']);
})();

// Length progresses continuously; the handle leads the live line ends with a clear gap.
const rl=(()=>{
 const ext=i=>P([[0,0],[.10,0],[.50,4.5,'out']],[[0,0],[.12+i*.03,0],[.44+i*.03,4.5]]);
 const hnd=P([[0,0],[.10,0],[.50,4.5,'out']],[[0,0],[.08,0],[.12,-.4],[.36,5.1],[.46,4.4],[.54,4.55],[.62,4.5]]);
 return noTint(make('response-length','Response length','Drag the handle right and every line extends with it; Strong extends in two deliberate steps with a held middle setting',[
  ...[7,13,19,25].flatMap((y,i)=>{const e=ext(i);return tube('l'+(i+1),(t,s)=>`M5,${y} L${18+e(t,s)},${y}`);}),
  ...posed(line('h',[22,5.5],[22,26.5]),shift(hnd))],.55,.80,['l1','l2','l3','l4','h']));
})();

// 14 Memory: pills h5 with 3.5 gaps (1.0 clear at Bold); markers r1.25 at x5.5 beside rows, 1.0 clear of the front pill.
const cm=(()=>{
 const rows=[[9,22,18,24.5],[10.5,13.5,16.5,16],[12,5,15,7.5]];
 const grow=i=>P([[0,.01],[.10+i*.12,.01],[.24+i*.12,1.25,'out']],[[0,.01],[.06+i*.18,.01],[.18+i*.18,1.25]]);
 const nudge=i=>P([[0,0]],[[0,0],[.045+i*.16,0],[.12+i*.16,1.1],[.20+i*.16,.7],[.33+i*.16,0]]);
 return make('conversation-memory','Conversation memory','Recall sweeps up the stack: a marker grows beside each remembered row; Strong nudges each row forward as it is recalled',[
  ...rows.flatMap(([x,y,w,cy],i)=>{const g=grow(i);return [...posed(framed('p'+(i+1),rect(x,y,w,5,2.5)),shift(nudge(i))),...bothFill('d'+(i+1),(t,s)=>circle(5.5,cy,g(t,s)))];})],.65,.95,['d1','d2','d3']);
})();

// 15 Structured: brace midpoint is a rounded cap (x≈4.9); indent to 17.5 leaves 1.15 between key dot and line cap.
const so=(()=>{
 const brace=m=>`M9,5.5 C7.5,5.5 7,6.5 7,8.5 L7,12.75 C7,14.6 6.3,15.3 ${m+.3},15.65 C${m-.1},15.8 ${m-.1},16.2 ${m+.3},16.35 C6.3,16.7 7,17.4 7,19.25 L7,23.5 C7,25.5 7.5,26.5 9,26.5`;
 const flex=P([[0,5]],[[0,5],[.24,5],[.34,3.6],[.44,5.5],[.52,4.8],[.6,5]]);
 const ind=i=>P([[0,12.5],[.10+i*.08,12.5],[.42+i*.08,17.5,'out']],[[0,12.5],[.06+i*.05,12.5],[.2+i*.05,17.5]]);
 const key=P([[0,.01],[.38,.01],[.56,1.1,'out']],[[0,.01],[.26,.01],[.36,1.35],[.44,.95],[.52,1.15],[.6,1.1]]);
 return noTint(make('structured-output','Structured output','Two lines indent under the first and gain key markers; Strong lets the brace flex once the structure nests',[
  ...tube('brace',(t,s)=>brace(flex(t,s))),...line('l1',[12.5,9],[26.5,9]),
  ...[[16,24.5],[23,26.5]].flatMap(([y,r],i)=>{const e=ind(i);return [...tube('l'+(i+2),(t,s)=>`M${e(t,s)},${y} L${r},${y}`),...bothFill('d'+(i+2),(t,s)=>circle(14,y,key(t,s)))];})],.60,.70,['l2','l3','d2','d3']));
})();

export const icons=[ac,rw,sm,tr,bs,rs,cb,pt,tn,cp,fu,it,rl,cm,so];
