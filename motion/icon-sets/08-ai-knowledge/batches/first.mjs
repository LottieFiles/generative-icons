// Accepted subset pending native renderer review.
import {circle,rect,capsule,polygon,holes,arc,annulus,stroke,fill,framed,bothFill,tube,line,rigid} from '../../library/drawing.mjs';
import {track,drive as motion,flow,timePair,resultAction,has,clamp} from '../../library/direction.mjs';
const seg=(a,b,p=1)=>`M${a[0]} ${a[1]} L${a[0]+(b[0]-a[0])*p} ${a[1]+(b[1]-a[1])*p}`;
const T=(t,s,l,h)=>track(s?h:l,t);
const at=(t,r)=>r?0:t;
const posed=(parts,pose)=>parts.map(p=>Object.assign(p,{pose}));
const shown=(parts,visibility)=>parts.map(p=>Object.assign(p,{visibility}));
const icon=o=>({description:o.intent,...o});
const tilde=(a,b)=>`M8.5 13.5 C10.3 ${13.5-2.9*a} 11.7 ${13.5-2.9*a} 13.5 13.5 C15.3 ${13.5+2.9*b} 16.7 ${13.5+2.9*b} 18.5 13.5`;
const ssHump=(t,s,i)=>{const d=i*.1;return s?flow([[.18+d,0],[.24+d,-.2],[.5+d,1.3],[.64+d,.88],[.76+d,1]],t):track([[d,0],[.38+d,1,'out']],t);};
const ssLens=rigid((t,s,r)=>({angle:s?flow([[0,0],[.06,1.5,'launch'],[.24,-6],[.36,1.2],[.46,0]],at(t,r)):0,pivot:[25.5,25.5]}));
export const semanticSearch=resultAction(icon({
 name:'semantic-search',label:'Semantic search',heading:'Read the meaning',
 intent:'Under the lens a flat text line turns into a meaning wave, hump by hump. Strong presses the lens onto the page first, then the wave forms with a small flexible overshoot.',
 parts:[
  ...posed(framed('semantic-search-lens',circle(13.5,13.5,8),annulus(13.5,13.5,8,3)),ssLens),
  ...posed([stroke('semantic-search-handle','M19.4 19.4 L25.5 25.5'),fill('semantic-search-handle-solid',capsule([19.4,19.4],[25.5,25.5],3),'solid')],ssLens),
  ...tube('semantic-search-meaning',(t,s)=>tilde(ssHump(t,s,0),ssHump(t,s,1))),
 ],
 timings:timePair(.55,.95,[0,.12,.24,.38,.48,.55],[0,.06,.24,.36,.5,.64,.86,.95]),
 brief:'Light: hump 1 (0-.38) then hump 2 (.1-.48) rise to amplitude 1. Strong: lens+handle rotate -4 deg about handle tip (0-.34), humps .3-.78 and .4-.88 with 1.25 overshoot. Fixed: the page line position. Test: handle cap buried in ring band (outline) and annulus (solid); line ends stay inside the tilted lens; Duotone tint follows lens pose; wave flattens only in reset.'
}),p=>has(p,'meaning'));

/* 3. citations: two opening quotation marks; a footnote rule with anchor tick draws beneath. */
const quote=(name,cx,cy,delay)=>posed([
 ...framed(`${name}-ball`,circle(cx,cy,3.8),circle(cx,cy,4.6)),
 ...tube(`${name}-tail`,`M${cx+2.4} ${cy-3} C${cx+3.4} ${cy-4.4} ${cx+3.6} ${cy-6} ${cx+3.2} ${cy-7.6}`),
],rigid((t,s,r)=>({dy:s?flow([[delay,0],[.08+delay,-.9,'launch'],[.2+delay,2.2,'hit'],[.3+delay,-.5],[.4+delay,0]],at(t,r)):0})));
const cite=p=>{const L=19*p,a=Math.min(3.5,L),b=Math.max(0,L-3.5);return `M9.5 20.5 L9.5 ${20.5+a} L${9.5+b} ${20.5+a}`;};
const citeP=(t,s)=>s?flow([[.3,0],[.54,1]],t):track([[0,0],[.56,1,'out']],t);
export const citations=resultAction(icon({
 name:'citations',label:'Citations',heading:'Attach the source',
 intent:'Two quotation marks state a claim; a footnote rule draws beneath them, anchor tick first, then the source line. Strong stamps the marks down before the rule draws.',
 parts:[
  ...quote('citations-mark-1',9.5,13,0),
  ...quote('citations-mark-2',21.5,13,.06),
  ...shown(tube('citations-source',(t,s)=>cite(citeP(t,s))),(t,s)=>citeP(t,s)>0?1:0),
 ],
 timings:timePair(.65,.62,[0,.14,.28,.42,.56,.65],[0,.08,.2,.3,.42,.54,.62]),
 brief:'Light: source rule draws 19 units, 3.5 vertical then 15.5 horizontal (0-.56). Strong: marks stamp +1.4 staggered .06 (0-.36), rule draws .3-.54. Fixed: marks in Light. Test: tail cap sits on ring centerline (no visible cap), 2.4-unit gap between balls at 2.5 weight, degenerate L segment renders cleanly at p<.18, anchor clears ball bottom by 1.7 units.'
}),p=>has(p,'source'));




const edge=(a,b,ra,rb)=>{const l=Math.hypot(b[0]-a[0],b[1]-a[1]),ux=(b[0]-a[0])/l,uy=(b[1]-a[1])/l;return seg([a[0]+ux*ra,a[1]+uy*ra],[b[0]-ux*rb,b[1]-uy*rb]);};
/* 1. knowledge-base (rev-01): three spines 5.3 wide with a title band; middle spine lifts out.
   Paint stays inside 3..29 at Bold in every frame. */
const KB_X=[4.25,13.35,22.45],KB_W=5.3,KB_TOP=9,KB_BOT=24.3,KB_BAND=13;
const kbVolume=i=>{const x=KB_X[i],n=`knowledge-base-volume-${i}`,w=KB_W;return[
 fill(n+'-interior',rect(x,KB_TOP,w,KB_BOT-KB_TOP,1.4),'tint'),
 stroke(n+'-frame',rect(x,KB_TOP,w,KB_BOT-KB_TOP,1.4)),
 stroke(n+'-band',`M${x} ${KB_BAND} L${x+w} ${KB_BAND}`),                       // cap discs sit inside the frame sides
 fill(n+'-solid-head',polygon([[x,KB_TOP],[x+w,KB_TOP],[x+w,12],[x,12]],[1.4,1.4,.6,.6]),'solid'),
 fill(n+'-solid-body',polygon([[x,13.6],[x+w,13.6],[x+w,26.5],[x,26.5]],[.6,.6,1.4,1.4]),'solid'), // reaches into rail capsule
];};
const kbLift=(t,s)=>motion(t,s,[[0,0],[.45,-3.5,'io']],[[0,0],[.08,.2,'launch'],[.36,-4.6],[.52,-3.1],[.64,-3.7],[.76,-3.5]]);
const kbLean=(t,s,r)=>s?flow([[.26,0],[.44,3],[.62,-.3],[.78,0]],at(t,r)):0;
export const knowledgeBase=resultAction(icon({
 name:'knowledge-base',label:'Knowledge base',heading:'Pull the volume',
 intent:'A query pulls the middle volume up out of a three-volume shelf. Strong dips first, over-pulls, settles, and the neighbours lean into the gap and recover.',
 parts:[
  ...line('knowledge-base-shelf',[5.5,27],[26.5,27]),
  ...posed(kbVolume(0),rigid((t,s,r)=>({angle:kbLean(t,s,r),pivot:[KB_X[0]+KB_W,KB_BOT]}))),
  ...posed(kbVolume(1),rigid((t,s)=>({dy:kbLift(t,s)}))),
  ...posed(kbVolume(2),rigid((t,s,r)=>({angle:-kbLean(t,s,r),pivot:[KB_X[2],KB_BOT]}))),
 ],
 timings:timePair(.6,.95,[0,.15,.3,.45,.6],[0,.08,.36,.5,.64,.78,.95]),
 brief:'Light: volume-1 rises 3.5 units (0-.45s), top edge 9 to 5.5. Strong: dip +.5 (0-.12), rise to -4.5 (.12-.5, top 4.5, Bold paint 3.25), settle to -3.5 (.76); volume-0/2 lean 3 deg about bottom inner corners (.32-.92). Fixed: shelf. Test: band stroke caps contained in frame sides; Solid head/body gap 1.6 reads at 24px; 3.8-unit gaps (1.3 clear at Bold) and 2.8-unit interiors at Bold; leaning top corner clears volume-1 by 0.5 at Bold; frame bottom 24.3 clears rail 27 by 0.2 at Bold; final dy=-3.5 in both profiles; reset fades only volume-1.'
}),p=>has(p,'volume-1'));

/* 4. embeddings (rev-01): L-shaped axes with a round join at the origin; existing data fixed;
   the token appears opaque-over-opaque at the origin, then travels while its vector draws behind it. */
const O=[6.5,25.5],P1=[23,13.5],P2=[16.5,10];
const toward=(a,b,d)=>{const l=Math.hypot(b[0]-a[0],b[1]-a[1]);return[b[0]-(b[0]-a[0])*d/l,b[1]-(b[1]-a[1])*d/l];};
const E2=toward(O,P2,1.4);                                                       // vector tip 1.4 inside the token disc
const emAppear=t=>track([[0,0],[.1,1,'out']],t);
const emTravel=(t,s)=>s?flow([[.1,0],[.17,-.05,'launch'],[.48,1.08],[.62,.97],[.74,1]],t):track([[.1,0],[.56,1,'out']],t);
export const embeddings=resultAction(icon({
 name:'embeddings',label:'Embeddings',heading:'Place the token',
 intent:'A token appears at the origin and travels out along its vector to sit near an existing point. Strong prepares at the origin, places the token decisively, then settles with bounded spatial follow-through. Existing data never moves.',
 duotoneFill:false,
 parts:[
  ...tube('embeddings-axes','M6.5 5.5 L6.5 25.5 L26.5 25.5'),
  ...tube('embeddings-vector-1',edge(O,P1,0,1.4)),
  ...bothFill('embeddings-point-1',circle(...P1,2.4)),
  ...shown(tube('embeddings-vector-2',(t,s)=>seg(O,E2,emTravel(t,s))),(t,s)=>emTravel(t,s)>.02?1:0),
  ...shown(posed(bothFill('embeddings-token',circle(...P2,2.4)),rigid((t,s)=>{const q=1-emTravel(t,s);return{dx:(O[0]-P2[0])*q,dy:(O[1]-P2[1])*q};})),emAppear),
 ],
 timings:timePair(.65,.9,[0,.06,.2,.38,.56,.65],[0,.1,.17,.32,.48,.62,.74,.9]),
 brief:'Light: token fades in at the origin over the opaque axis join (0-.1), then travels to P2 while vector-2 draws behind it (.1-.56). Strong: prepare, accelerate continuously through placement, then settle once at the shared endpoint. Current authored curves define timing. Fixed: axes, vector-1, point-1. Test: Duotone equals Outline; vector caps at O coincide with the axis round join at all weights; vector-2 is gated off until 2% drawn so no zero-length stub; token disc overlaps only opaque paint while translucent; vector-2 tip stays 1.4k inside the token disc for all k; reset fades vector-2 and token together (request common-opacity group).'
}),p=>has(p,'vector-2','token'));

/* 5. knowledge-graph (rev-01): hub r3.4 + three satellites r2.6. Every edge centerline is cut 0.4 beyond
   the ring centerline of both nodes, so a round cap never enters a hollow at 1.5/2/2.5. */
const H=[16,13.5],A=[7.4,7.2],B=[24.6,7.2],C=[16,24.4],NR=2.6,HR=3.4,CUT_N=NR+.4,CUT_H=HR+.4;
const unit=(a,b)=>{const l=Math.hypot(b[0]-a[0],b[1]-a[1]);return[(b[0]-a[0])/l,(b[1]-a[1])/l];};
const uA=unit(A,H),uB=unit(B,H);
const kgHub=(t,s,r)=>s?flow([[0,0],[.08,-.6,'launch'],[.24,1.8],[.36,-.4],[.46,0]],at(t,r)):0;
const kgNudge=(t,s,r)=>s?flow([[.56,0],[.72,1],[.84,-.3],[.94,0]],at(t,r)):0;
const kgDrop=(t,s)=>s?flow([[.38,.8],[.62,-.35],[.74,.12],[.84,0]],t):track([[.28,.8],[.52,0,'out']],t);
const kgDraw=(t,s)=>s?flow([[.22,0],[.58,1]],t):track([[.04,0],[.42,1,'out']],t);
const hubAt=(t,s,r)=>[H[0],H[1]+kgHub(t,s,r)];
const satAt=(P,u)=>(t,s,r)=>{const k=kgNudge(t,s,r);return[P[0]+u[0]*k,P[1]+u[1]*k];};
const aAt=satAt(A,uA),bAt=satAt(B,uB),cAt=(t,s)=>[C[0],C[1]+kgDrop(t,s)];
const node=(name,c,r,rs)=>framed(name,circle(...c,r),circle(...c,rs));
export const knowledgeGraph=resultAction(icon({
 name:'knowledge-graph',label:'Knowledge graph',heading:'Link the node',
 intent:'A new edge grows from the hub to an unlinked node, which snaps into place as it connects. Strong leans the hub toward it first and lets the linked neighbours rebalance afterwards.',
 parts:[
  ...tube('knowledge-graph-edge-a',(t,s,r)=>edge(aAt(t,s,r),hubAt(t,s,r),CUT_N,CUT_H)),
  ...tube('knowledge-graph-edge-b',(t,s,r)=>edge(bAt(t,s,r),hubAt(t,s,r),CUT_N,CUT_H)),
  ...shown(tube('knowledge-graph-edge-c',(t,s)=>{const h=hubAt(t,s,false),c=cAt(t,s);return seg([h[0],h[1]+CUT_H],[c[0],c[1]-CUT_N],kgDraw(t,s));}),(t,s)=>kgDraw(t,s)>.02?1:0),
  ...posed(node('knowledge-graph-hub',H,HR,4.4),rigid((t,s,r)=>({dy:kgHub(t,s,r)}))),
  ...posed(node('knowledge-graph-node-a',A,NR,3.6),rigid((t,s,r)=>{const k=kgNudge(t,s,r);return{dx:uA[0]*k,dy:uA[1]*k};})),
  ...posed(node('knowledge-graph-node-b',B,NR,3.6),rigid((t,s,r)=>{const k=kgNudge(t,s,r);return{dx:uB[0]*k,dy:uB[1]*k};})),
  ...posed(node('knowledge-graph-node-c',C,NR,3.6),rigid((t,s)=>({dy:kgDrop(t,s)}))),
 ],
 timings:timePair(.6,.95,[0,.15,.28,.42,.52,.6],[0,.08,.24,.38,.5,.62,.74,.84,.95]),
 brief:'Light: edge-c grows hub->node-c (.04-.42); node-c snaps up 0.8 units as the edge lands (.28-.52). Strong: hub leans +1 toward C and returns (0-.32), edges a/b follow the hub; edge-c grows .3-.68; snap .5-.78; node-a/b nudge .7 toward hub and back (.62-1). Every edge endpoint is recomputed from the current node centres, so the 3.0/3.8 cuts hold in every frame. Test: at Bold the nearest cap paint sits 0.4 outside every hollow; visible edge-c length 4.1 at rest, 4.9 at snap start; node-c start pose bottom paint 29.05 at Bold; satellite hollows 2.7 units across at Bold; Solid discs 3.6/4.4 cover tube caps ending 1.7/2.5 from centre; reset fades only node-c and edge-c (request common-opacity group for their shared overlap).'
}),p=>has(p,'node-c','edge-c'));

// Reset-only occlusion ordering: the vector fades while its terminal stays opaque.
for(const p of embeddings.parts.filter(p=>p.name.includes('embeddings-token'))){
 const visible=p.visibility;
 p.visibility=(t,s,r)=>r?track([[0,1],[.08,1],[.12,0,'out']],t):visible(t,s,false);
}
// Clear the relation first; reposition the still-opaque node afterwards.
for(const p of knowledgeGraph.parts.filter(p=>p.name.includes('node-c'))){
 const forward=p.pose,fn=forward.transform;
 p.pose=rigid((t,s,r)=>r?{dy:track([[0,0],[.09,0],[.24,.8,'io']],t)}:fn(t,s,false));
 p.visibility=()=>1;
}
export const icons=[knowledgeBase,semanticSearch,citations,embeddings,knowledgeGraph];
