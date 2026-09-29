// Claude revised proposal integrated for native review.
// remaining15.mjs — Claude draft for Codex review. Unrendered.
import {make,F as P,posed,shown,fill,stroke,line,tube,framed,bothFill,rigid,rect,circle,capsule,polygon,holes} from '../../library/ai-authoring.mjs';
const L=(a,b)=>`M${a[0]} ${a[1]} L${b[0]} ${b[1]}`;
const up=v=>rigid((t,s)=>({dy:v(t,s)}));

// 1 document-chat — shell fixed; Solid slot revealed by a cover whose concave end forms the pen tip
function documentChat(){
 const wr=P([[.08,11],[.5,18.5,'out']],[[.06,11],[.24,18.5]]);
 const nod=P([[0,0]],[[.22,0],[.3,-1],[.4,.55],[.5,-.35],[.58,.12],[.66,0]]);
 const shell=n=>`M10 5 L22 5 C23.66 5 25 6.34 25 8 L25 21 C25 22.66 23.66 24 22 24 L15.2 24 C14.6 24 14.2 24.2 13.9 24.6 L${11.5-.3*n} ${27.2+n} C${11-.3*n} ${27.7+n} 10.5 ${27.5+.6*n} 10.5 ${26.8+.4*n} L10.5 24 L10 24 C8.34 24 7 22.66 7 21 L7 8 C7 6.34 8.34 5 10 5 Z`;
 const slots=[capsule([11,11],[21,11],2.6),capsule([11,15],[18,15],2.6),capsule([11,19],[19.5,19],2.6)];
 const cover=x=>`M${x} 17.6 L19.5 17.6 C20.27 17.6 20.9 18.23 20.9 19 C20.9 19.77 20.27 20.4 19.5 20.4 L${x} 20.4 C${x+.77} 20.4 ${x+1.4} 19.77 ${x+1.4} 19 C${x+1.4} 18.23 ${x+.77} 17.6 ${x} 17.6 Z`;
 return make('document-chat','Document chat','A reply line writes into a document-shaped bubble; Strong adds a tail nod.',[
  ...framed('bubble',(t,s)=>shell(nod(t,s)),(t,s)=>holes(shell(nod(t,s)),...slots)),
  stroke('text1',L([11,11],[21,11])),stroke('text2',L([11,15],[18,15])),
  stroke('ink',(t,s)=>L([11,19],[wr(t,s),19])),
  fill('ink-solid',(t,s)=>cover(wr(t,s)),'solid')
 ],.55,.72,['ink']);
}

// 2 source-grounding — block lifts, stem lengthens from fixed ground, block drops and settles
function sourceGrounding(){
 const dy=P([[0,0],[.15,-1.5,'out'],[.2,-1.5],[.4,0,'in'],[.45,.3,'out'],[.5,0,'out']],[[0,0],[.08,.4,'launch'],[.3,-2.5],[.5,0,'hit'],[.58,-.8],[.66,0,'hit'],[.72,-.25],[.78,0]]);
 const block=rect(11,6,10,9,2.5),q=[[13.5,10.5],[18.5,10.5]];
 return make('source-grounding','Source grounding','A statement block lifts off its ground symbol and settles back onto it.',[
  ...posed([...framed('block',block,holes(block,capsule(...q,2.6))),stroke('block-quote',L(...q))],up(dy)),
  ...tube('stem',(t,s)=>L([16,15+dy(t,s)],[16,21])),
  ...line('ground1',[7,21],[25,21]),...line('ground2',[10.5,24.5],[21.5,24.5]),...line('ground3',[14,28],[18,28])
 ],.5,.85);
}

// 3 retrieve-context — card rises 3 in both profiles; second line writes; Strong prepares briefly, then lifts continuously
function retrieveContext(){
 const dy=P([[.05,0],[.4,-3,'out']],[[0,0],[.08,.25,'launch'],[.29,-3.4],[.40,-2.9],[.52,-3]]);
 const wr=P([[.3,12.5],[.55,17.5,'out']],[[.32,12.5],[.46,17.5]]);
 const card=rect(9,7,14,11,2.5),l1=[[12.5,10.75],[19.5,10.75]];
 return make('retrieve-context','Retrieve context','A card lifts from its reader slot and its retrieved line writes.',[
  ...framed('slot',rect(6,21.5,20,5.5,2.75)),
  ...posed([...framed('card',card,(t,s)=>holes(card,capsule(...l1,2.6),capsule([12.5,14.25],[wr(t,s),14.25],2.6))),
   stroke('card-l1',L(...l1)),stroke('card-l2',(t,s)=>L([12.5,14.25],[wr(t,s),14.25]))],up(dy))
 ],.6,.62,['card']);
}

// 4 vector-store — dot-headed vectors sink toward a fixed crate lid, stalks shorten
function vectorStore(){
 const node=(i,base,top,down)=>{const j=i-1;
  const p=P([[.05+j*.1,0],[.4+j*.1,1,'io']],[[0,0],[.1+j*.06,-.1,'launch'],[.42+j*.08,1.1],[.56+j*.08,.94],[.68+j*.08,1]]);
  const at=(t,s)=>{const u=p(t,s);return[top[0]+(down[0]-top[0])*u,top[1]+(down[1]-top[1])*u];};
  return[...tube(`p${i}-stalk`,(t,s)=>L(base,at(t,s))),...bothFill(`p${i}-dot`,(t,s)=>circle(...at(t,s),1.6))];};
 return make('vector-store','Vector store','Three vectors are pulled down onto a crate lid and stored.',[
  ...framed('crate',rect(6,18,20,8,2.5)),
  ...node(1,[10,18],[8.5,7],[10,14]),...node(2,[16,18],[16,5],[16,13]),...node(3,[22,18],[23.5,7],[22,14])
 ],.65,.9,['p1','p2','p3']);
}

// 5 dataset — table grows a row, then a value is written; Solid is rim + bars
function dataset(){
 const bot=P([[.05,20],[.4,27,'out']],[[0,20],[.22,27]]);
 const col=P([[.05,19.4],[.4,26.4,'out']],[[.04,19.4],[.24,26.4]]);
 const row=P([[.25,5.6],[.45,26.4,'out']],[[.24,5.6],[.34,26.4]]);
 const val=P([[.42,8.5],[.58,11.5,'out']],[[.38,8.5],[.48,11.5]]);
 return make('dataset','Dataset','A table gains a new row and a value is entered into it.',[
  ...framed('table-frame',(t,s)=>rect(5,6,22,bot(t,s)-6,3),(t,s)=>holes(rect(3.7,4.7,24.6,bot(t,s)-3.4,4.3),rect(6.3,7.3,19.4,bot(t,s)-8.6,1.7))),
  ...tube('table-col',(t,s)=>L([16,6.6],[16,col(t,s)])),
  ...line('table-row1',[5.6,13],[26.4,13]),
  ...shown(tube('table-row2',(t,s)=>L([5.6,20],[row(t,s),20])),P([[.2,0],[.26,1,'out']],[[.22,0],[.26,1]])),
  ...shown(tube('table-val',(t,s)=>L([8.5,23.5],[val(t,s),23.5])),P([[.4,0],[.45,1,'out']],[[.36,0],[.4,1]]))
 ],.6,.6,['table']);
}

// 6 classification — token drops through the fork into the left open bin; Strong passes continuously through the fork
function classification(){
 const dy=P([[0,0],[.2,7.5,'in'],[.45,13.5,'io'],[.55,17.25,'out']],[[0,0],[.08,-.4,'launch'],[.2,7.5,'hit'],[.34,13.5],[.42,17.25,'hit'],[.48,16.1],[.54,17.25]]);
 const dx=P([[.2,0],[.45,-6.5,'io']],[[.2,0],[.34,-6.5]]);
 const bin=x=>`M${x} 20 L${x} 24.5 C${x} 25.9 ${x+1.1} 27 ${x+2.5} 27 L${x+6.5} 27 C${x+7.9} 27 ${x+9} 25.9 ${x+9} 24.5 L${x+9} 20`;
 return Object.assign(make('classification','Classification','A token falls down a splitter into one of two open bins.',[
  ...tube('stem',L([16,7],[16,13])),...tube('fork','M9.5 19 L16 13 L22.5 19'),
  ...tube('bin-l',bin(5)),...tube('bin-r',bin(18)),
  ...posed(bothFill('token',circle(16,5.5,2)),rigid((t,s)=>({dx:dx(t,s),dy:dy(t,s)})))
 ],.55,.6,['token']),{duotoneFill:false});
}

// 7 clustering — four stray dots gather around two outlined centroid rings; straight, non-crossing paths
function clustering(){
 const sat=(i,a,b)=>{const j=i-1;const k=.6/Math.hypot(b[0]-a[0],b[1]-a[1]);
  const p=P([[.05+j*.03,0],[.5+j*.03,1,'io']],[[0,0],[.1+j*.05,-k,'launch'],[.44+j*.06,1.1],[.56+j*.06,.96],[.66+j*.06,1]]);
  return posed(bothFill(`s${i}`,circle(...a,1.4)),rigid((t,s)=>{const u=p(t,s);return{dx:(b[0]-a[0])*u,dy:(b[1]-a[1])*u};}));};
 return make('clustering','Clustering','Scattered points gather around two cluster centroids.',[
  ...framed('ring-a',circle(10,10,3)),...framed('ring-b',circle(22,22,3)),
  ...sat(1,[24,5],[16.6,7.6]),...sat(2,[27,13],[24.4,15.4]),...sat(3,[5,26],[7.6,16.6]),...sat(4,[12,27],[15.4,24.4])
 ],.6,.9,['s1','s2','s3','s4']);
}

// 8 entity-extraction — pill lifts 6.5 out of line one; the line closes to a word gap
function entityExtraction(){
 const dy=P([[.05,0],[.45,-6.5,'out']],[[0,0],[.06,.6,'launch'],[.25,-6.5]]);
 const a=P([[.45,8],[.6,14,'out']],[[.29,8],[.47,14]]),b=P([[.45,24],[.6,18,'out']],[[.29,24],[.47,18]]);
 return make('entity-extraction','Entity extraction','An entity pill lifts out of the text and the line closes behind it.',[
  ...tube('l1a',(t,s)=>L([6,13],[a(t,s),13])),...tube('l1b',(t,s)=>L([b(t,s),13],[26,13])),
  ...line('l2',[6,20],[26,20]),...line('l3',[6,26],[18,26]),
  ...posed(framed('pill',rect(11.5,10,9,6,3)),up(dy))
 ],.6,.9,['pill','l1a','l1b']);
}

// 9 fact-check — stamp presses 5 (Strong 5.3 squash), check draws after contact, stamp returns
function factCheck(){
 const dy=P([[.05,0],[.25,5,'in'],[.35,5],[.58,0,'out']],[[0,0],[.07,-1.1,'launch'],[.26,5.5,'hit'],[.37,4.4],[.64,-.4],[.75,0]]);
 const pr=P([[.33,0],[.55,1,'out']],[[.34,0],[.62,1]]);
 const stamp='M15.5 5 L16.5 5 C17.9 5 19 6.1 19 7.5 L19 10 C19 10.55 19.45 11 20 11 L22 11 C23.1 11 24 11.9 24 13 L24 15 C24 16.1 23.1 17 22 17 L10 17 C8.9 17 8 16.1 8 15 L8 13 C8 11.9 8.9 11 10 11 L12 11 C12.55 11 13 10.55 13 10 L13 7.5 C13 6.1 14.1 5 15.5 5 Z';
 const check=u=>{const m=.324,a=[12,24],b=[15,27],c=[21,20.5];
  const p=u<m?[a[0]+3*u/m,a[1]+3*u/m]:[b[0]+6*(u-m)/(1-m),b[1]-6.5*(u-m)/(1-m)];const k=u<m?p:b;
  return`M12 24 L${k[0]} ${k[1]} L${p[0]} ${p[1]}`;};
 return make('fact-check','Fact check','A stamp presses down and leaves a checkmark imprint.',[
  ...shown(tube('check',(t,s)=>check(pr(t,s))),P([[.31,0],[.35,1,'out']],[[.32,0],[.36,1]])),
  ...posed(framed('stamp',stamp),up(dy))
 ],.6,.85,['check']);
}

// 10 source-rank — single stepped podium contour; right step rises, left step drops
function sourceRank(){
 const l=P([[.05,14],[.5,17,'io']],[[0,14],[.10,14],[.16,15,'pass'],[.28,17]]),r=P([[.05,19],[.5,12,'io']],[[0,19],[.025,19],[.095,17,'pass'],[.23,12]]);
 const pod=(t,s)=>polygon([[4,26],[4,l(t,s)],[13,l(t,s)],[13,8],[19,8],[19,r(t,s)],[28,r(t,s)],[28,26]],[2.5,2.5,1.5,2.5,2.5,1.5,2.5,2.5]);
 return make('source-rank','Source rank','A podium re-ranks: one source rises while another drops.',framed('podium',pod),.6,.8,['podium']);
}

// 11 index-data — attached back tab lifts (Strong), two entries write into a divider window
function indexData(){
 const lift=P([[0,0]],[[0,0],[.05,-.3,'launch'],[.18,1.5],[.32,.9],[.48,-.2],[.6,0]]);
 const e1=P([[.05,9],[.35,21,'out']],[[.18,9],[.32,21]]),e2=P([[.25,9],[.55,16,'out']],[[.26,9],[.4,16]]);
 const div='M8.5 9 L11.5 9 C12.9 9 14 10.1 14 11.5 L14 13.5 C14 14.05 14.45 14.5 15 14.5 L23.5 14.5 C24.9 14.5 26 15.6 26 17 L26 25 C26 26.4 24.9 27.5 23.5 27.5 L8.5 27.5 C7.1 27.5 6 26.4 6 25 L6 11.5 C6 10.1 7.1 9 8.5 9 Z';
 const tab=l=>`M17 13.9 L17 ${7.5-l} C17 ${6.1-l} 18.1 ${5-l} 19.5 ${5-l} L22.5 ${5-l} C23.9 ${5-l} 25 ${6.1-l} 25 ${7.5-l} L25 13.9`;
 return make('index-data','Index data','Entries are written onto a tabbed divider while its back tab is pulled and reseated.',[
  stroke('tab',(t,s)=>tab(lift(t,s))),fill('tab-solid',(t,s)=>rect(17,5-lift(t,s),8,11+lift(t,s),2.5),'solid'),
  ...framed('divider',div,holes(div,rect(7,16.5,18,9,1.5))),
  ...tube('e1',(t,s)=>L([9,19],[e1(t,s),19])),...tube('e2',(t,s)=>L([9,23.5],[e2(t,s),23.5]))
 ],.6,.66,['e1','e2']);
}

// 12 deduplicate — B fades fully before A slides to centre; Strong compares first
function deduplicate(){
 const ax=P([[.4,0],[.6,7,'out']],[[0,0],[.13,.6,'hit'],[.23,-.3],[.31,0],[.40,-.5,'launch'],[.64,7.4],[.75,6.85],[.84,7]]);
 const bx=P([[0,0]],[[0,0],[.16,-.6,'hit'],[.26,.3],[.34,0]]);
 const bo=P([[.1,1],[.4,0,'out']],[[.26,1],[.44,0]]);
 const card=(n,x)=>{const c=rect(x,9,9,14,2.5),l1=[[x+3.5,14],[x+5.5,14]],l2=[[x+3.5,18],[x+5.5,18]];
  return[fill(n+'-interior',rect(x+1,10,7,12,1.5),'tint'),stroke(n+'-frame',c),fill(n+'-solid',holes(c,capsule(...l1,2.6),capsule(...l2,2.6)),'solid'),stroke(n+'-t1',L(...l1)),stroke(n+'-t2',L(...l2))];};
 return make('deduplicate','Deduplicate','The duplicate record dissolves and the survivor recentres.',[
  ...shown(posed(card('b',18.5),rigid((t,s)=>({dx:bx(t,s)}))),bo),
  ...posed(card('a',4.5),rigid((t,s)=>({dx:ax(t,s)})))
 ],.6,.95,['a','b']);
}

// 13 data-cleaning — broom pushes right 5 with a 6° tilt about the handle top, specks vanish under the head
function dataCleaning(){
 const ang=P([[.05,0],[.35,-6,'io'],[.38,-6],[.7,0,'out']],[[0,0],[.07,2.5,'launch'],[.32,-6.5],[.46,-5.2],[.75,0]]);
 const dx=P([[.05,0],[.35,5,'io'],[.38,5],[.7,0,'out']],[[0,0],[.07,-1,'launch'],[.32,6.5],[.46,5.2],[.75,0]]);
 const speck=(i,x,y,tl,ts)=>shown(bothFill(`speck${i}`,circle(x,y,1.3)),P([[tl,1],[tl+.1,0,'out']],[[ts,1],[ts+.1,0,'out']]));
 return make('data-cleaning','Data cleaning','A broom sweeps three specks off the canvas.',[
  ...speck(1,21,24,.22,.3),...speck(2,24,26.5,.28,.36),...speck(3,25,21,.34,.42),
  ...posed([...framed('head',rect(6,19,12,7,2.5)),...tube('handle',L([22,4],[13.7,18.65]))],rigid((t,s)=>({angle:ang(t,s),dx:dx(t,s),pivot:[22,4]})))
 ],.7,.9,['speck1','speck2','speck3']);
}

// 14 context-attach — pin drops 5 onto a fixed card; entry writes after contact
function contextAttach(){
 const dy=P([[.05,0],[.35,5,'in']],[[0,0],[.08,-1,'launch'],[.19,5.4,'hit'],[.27,4.85],[.33,5.1],[.39,5]]);
 const en=P([[.45,9.5],[.6,20,'out']],[[.21,9.5],[.35,20]]);
 const card=rect(6,20,20,8,2.5);
 return make('context-attach','Context attach','A pushpin drops onto a note card and the attached context writes.',[
  ...framed('card',card,holes(card,rect(7.6,21.6,16.8,4.8,1.6))),
  ...tube('entry',(t,s)=>L([9.5,24],[en(t,s),24])),
  ...posed([...framed('pin-head',rect(11.5,4.5,9,6,3)),...tube('pin-needle',L([16,10.5],[16,15]))],up(dy))
 ],.6,.6,['pin','entry']);
}

// 15 long-term-memory — vault dial turns 90 (Strong 180 with wind-up and click); fourfold symmetric
function longTermMemory(){
 const ang=P([[.05,0],[.5,90,'out']],[[0,0],[.1,-20,'launch'],[.5,188],[.62,176],[.72,182],[.8,180]]);
 const frame=rect(4,4,24,24,4.5),tick=(n,a,b)=>tube('dial-'+n,L(a,b));
 return Object.assign(make('long-term-memory','Long-term memory','A memory vault dial turns to its next locked position.',[
  fill('frame-interior',frame,'tint'),stroke('frame',frame),fill('frame-solid',holes(rect(3,3,26,26,5.5),circle(16,16,10)),'solid'),
  ...posed([stroke('dial',circle(16,16,4.5)),fill('dial-solid',circle(16,16,5.8),'solid'),
   tick('n',[16,11.5],[16,8.5]),tick('e',[20.5,16],[23.5,16]),tick('s',[16,20.5],[16,23.5]),tick('w',[11.5,16],[8.5,16])].flat(),rigid((t,s)=>({angle:ang(t,s)})))
 ],.55,.85),{cycleDegrees:90,symmetricCycle:true});
}

export const icons=[documentChat(),sourceGrounding(),retrieveContext(),vectorStore(),dataset(),classification(),clustering(),entityExtraction(),factCheck(),sourceRank(),indexData(),deduplicate(),dataCleaning(),contextAttach(),longTermMemory()];
