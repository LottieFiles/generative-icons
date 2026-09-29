// Claude Code design; Codex integration and verification.
// agents-authoring-04.mjs — draft, unrendered
import {make,F as P,posed,shown,framed,tube,line,fill,stroke,bothFill,rigid,rect,circle,arc,hole,capsule,mix} from '../../library/ai-authoring.mjs';
const dyn=(parts,fn)=>parts.map(p=>({...p,d:(t,s)=>fn(t,s,p.style==='solid')}));
const cap=(a,b,solid)=>solid?capsule(a,b,2.6):`M${a} L${b}`;

// 11 local-model — core: pivot (16,16); pins: rigid outward translation, caps end ≥3.1
const lmCore=P([[0,0],[.48,180,'io']],[[0,0],[.1,-22],[.46,188],[.58,177],[.66,180]]);
const lmTB=P([[.34,0],[.5,.6,'out']],[[.44,0],[.56,.75],[.66,.6]]);
const lmLR=P([[.34,0],[.5,.6,'out']],[[.5,0],[.62,.75],[.72,.6]]);
const pin=(n,a,b,ux,uy,ext)=>posed(line(n,a,b),rigid((t,s)=>({dx:ux*ext(t,s),dy:uy*ext(t,s)})));
const body=rect(8,8,16,16,3.5);
const localModel=make('local-model','Local model','On-device model: the chip core spins up to a locked angle and every pin seats outward.',[
 ...framed('body',body,hole(body,circle(16,16,6.25))),
 ...posed(tube('core',arc(16,16,3.75,75,375)),rigid((t,s)=>({angle:lmCore(t,s),pivot:[16,16]}))),
 ...pin('pin-t1',[12.5,8],[12.5,5],0,-1,lmTB),...pin('pin-t2',[19.5,8],[19.5,5],0,-1,lmTB),
 ...pin('pin-b1',[12.5,24],[12.5,27],0,1,lmTB),...pin('pin-b2',[19.5,24],[19.5,27],0,1,lmTB),
 ...pin('pin-l1',[8,12.5],[5,12.5],-1,0,lmLR),...pin('pin-l2',[8,19.5],[5,19.5],-1,0,lmLR),
 ...pin('pin-r1',[24,12.5],[27,12.5],1,0,lmLR),...pin('pin-r2',[24,19.5],[27,19.5],1,0,lmLR)],
 .6,.95,['core','pin']);

// 12 model-deploy — dock is a returned follower (dip); model + lamp are the retained result
const mdDrop=P([[0,0],[.46,10,'io']],[[0,0],[.12,-1.5],[.4,10,'hit']]);
const mdDip=P([[0,0]],[[.4,0],[.47,.6],[.56,-.15],[.64,0]]);
const mdLamp=P([[.46,0],[.6,1,'smooth']],[[.5,0],[.62,1]]);
const unit=rect(10,5,12,10,3),ride=rigid((t,s)=>({dy:mdDrop(t,s)+mdDip(t,s)}));
const modelDeploy=make('model-deploy','Model deploy','A model unit lowers into its serving dock, seats, and its status lamp comes on.',[
 ...posed(tube('dock','M6 16 L6 25 C6 26.66 7.34 28 9 28 L23 28 C24.66 28 26 26.66 26 25 L26 16'),rigid((t,s)=>({dy:mdDip(t,s)}))),
 ...posed(framed('model',unit,hole(unit,circle(16,10,2.75))),ride),
 ...shown(posed(bothFill('lamp',circle(16,10,1.75)),ride),(t,s)=>mdLamp(t,s))],
 .65,1,['model','lamp']);

// 13 model-version — pills 5 tall on a 9 pitch (gap 4). New branch = one joined outline path.
const A=Math.PI*1.25,PL=4*A+16,seg=d=>d.replace(/^M[^C]*/,'');
const pt=(cx,cy,a)=>[cx+2.5*Math.cos(a*Math.PI/180),cy+2.5*Math.sin(a*Math.PI/180)];
function rim(p,y,start){let L=p*PL,end=start,d='';
 const go=(len,f0,fn)=>{const f=Math.min(1,L/len);L=Math.max(0,L-len);if(f>0){const r=fn(f);d+=r[0];end=r[1];}else d+=f0(end);};
 const ar=(cx,a)=>go(A,e=>` C${e} ${e} ${e}`,f=>[' '+seg(arc(cx,y,2.5,a,a+90*f)),pt(cx,y,a+90*f)]);
 const ln=(x0,x1,yy)=>go(8,e=>` L${e}`,f=>[` L${mix(x0,x1,f)} ${yy}`,[mix(x0,x1,f),yy]]);
 ar(16.5,180);ln(16.5,24.5,y-2.5);ar(24.5,270);ar(24.5,0);ln(24.5,16.5,y+2.5);ar(16.5,90);return d;}
const mvT=P([[0,16],[.26,7,'io']],[[0,16],[.16,7]]);
const mvS=P([[.24,8],[.36,14,'out']],[[.16,8],[.26,14]]);
const mvP=P([[.38,0],[.6,1,'io']],[[.26,0],[.46,1]]);
const mvTint=P([[.54,0],[.64,1,'smooth']],[[.4,0],[.52,1]]);
const branchD=(t,s)=>`M8 16 L8 ${mvT(t,s)} L${mvS(t,s)} ${mvT(t,s)}`;
const modelVersion=make('model-version','Model version','The trunk grows a new branch and its version record materializes at the tip.',[
 ...tube('trunk','M8 25 L8 16'),
 ...tube('stub-2','M8 16 L14 16'),...framed('v2',rect(14,13.5,13,5,2.5)),
 ...tube('stub-1','M8 25 L14 25'),...framed('v1',rect(14,22.5,13,5,2.5)),
 stroke('branch',(t,s)=>branchD(t,s)+rim(mvP(t,s),7,[mvS(t,s),mvT(t,s)])),
 {name:'branch-solid',style:'solid',type:'stroke',width:2.6,d:branchD},
 fill('record-tint',rect(14,4.5,13,5,2.5),'tint',{visibility:(t,s)=>mvTint(t,s)}),
 fill('record-solid',(t,s)=>rect(14,4.5,13*mvP(t,s),5,2.5))],
 .65,1,['branch','record']);

// 14 usage-meter — dial centre (16,19) r11, needle pivot (16,19), 150°→294°, tick at 342°
const umAng=P([[0,0],[.48,144,'out']],[[0,0],[.05,8,'pass'],[.14,90,'pass'],[.24,144]]);
const umTick=P([[.38,0],[.54,1,'out']],[[.24,0],[.34,1]]);
const T0=[21.47,17.22],T1=[23.13,16.68];
const usageMeter=Object.assign(make('usage-meter','Usage meter','The needle sweeps to current usage and the limit mark appears ahead of it.',[
 ...tube('dial',arc(16,19,11,150,390)),
 ...posed(tube('needle','M16 19 L9.505 22.75'),rigid((t,s)=>({angle:umAng(t,s),pivot:[16,19]}))),
 ...bothFill('hub',circle(16,19,2.75)),
 ...shown(dyn(line('tick',T0,T1),(t,s,so)=>{const q=umTick(t,s);return cap(T0,[mix(T0[0],T1[0],q),mix(T0[1],T1[1],q)],so);}),(t,s)=>umTick(t,s)>.001?1:0)],
 .6,.65,['needle','tick']),{duotoneFill:false});

// 15 ai-connection — cable ends on ring centrelines (7.5,16)/(24.5,16) r3.5; node B fixed
const acEnd=P([[0,8.5],[.42,12.97,'in']],[[0,8.5],[.12,6],[.42,12.97,'hit']]);
const acC2x=P([[0,21],[.42,22,'in']],[[0,21],[.12,20.3],[.42,22]]);
const acC2y=P([[0,3.5],[.42,5,'in']],[[0,3.5],[.12,2],[.42,5],[.5,3.9],[.6,5.3],[.68,5]]);
const acC1y=P([[0,5]],[[0,5],[.42,5],[.5,4],[.6,5.3],[.68,5]]);
const acLink=P([[.4,11],[.58,21,'out']],[[.48,11],[.58,21]]);
const aiConnection=make('ai-connection','AI connection','The cable swings from one node onto the other, settles, and the lower link completes.',[
 ...framed('node-a',circle(7.5,16,3.5)),...framed('node-b',circle(24.5,16,3.5)),
 ...dyn(tube('cable','M9.25 12.97 C10 5 21 3.5 22.75 8.5'),(t,s)=>`M9.25 12.97 C10 ${acC1y(t,s)} ${acC2x(t,s)} ${acC2y(t,s)} 22.75 ${acEnd(t,s)}`),
 ...dyn(line('link',[11,16],[21,16]),(t,s,so)=>cap([11,16],[acLink(t,s),16],so))],
 .6,.75,['cable','link']);

export const icons=[localModel,modelDeploy,modelVersion,usageMeter,aiConnection];
