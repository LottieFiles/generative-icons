// Claude Code design; Codex integration and verification.
// ai-batch-03.mjs — fine-tune, model-evaluation, confidence-score, inference, batch-inference
import {make,framed,tube,line,circle,rect,arc,rigid,rotate,F as P,segment,mix} from '../../library/ai-authoring.mjs';
export const icons=[];

// fine-tune: a tuning fork is struck, rings, and keeps its two pitch arcs.
const ftFlex=P([[0,0],[.05,1,'out'],[.13,-.7],[.21,.45],[.29,-.28],[.37,.15],[.45,0,'out']],
 [[0,0],[.18,0],[.24,1.6],[.31,-1.15],[.38,.8],[.45,-.55],[.52,.36],[.59,-.22],[.66,.12],[.73,0]]);
const ftLean=P([[0,0]],[[0,0],[.1,-7],[.2,2],[.3,0]]);
const ftPose=rigid((t,s)=>({angle:ftLean(t,s),pivot:[14,28]}));
const ftNear=P([[0,0],[.30,0],[.50,1,'out']],[[0,0],[.34,0],[.52,1]]);
const ftFar=P([[0,0],[.38,0],[.58,1,'out']],[[0,0],[.46,0],[.66,1]]);
const pitch=(n,r,span,g)=>tube(n,(t,s)=>{const a=span*Math.max(.02,g(t,s));return arc(14,9,r,-a,a);},{visibility:(t,s)=>g(t,s)>.02?1:0});
icons.push(Object.assign(make('fine-tune','Fine-tune','A struck tuning fork rings and holds its pitch marks.',[
 ...tube('tines',(t,s)=>{const f=ftFlex(t,s);return `M${9-f} 4 C${9-f*.5} 7 9 9 9 11 C9 14.5 19 14.5 19 11 C19 9 ${19+f*.5} 7 ${19+f} 4`;},{pose:ftPose}),
 ...line('stem',[14,13.6],[14,28],{pose:ftPose}),
 ...pitch('near',9,22,ftNear),...pitch('far',12.5,20,ftFar)],.6,1,['near','far']),{duotoneFill:false}));

// model-evaluation: the balance tips and settles level; pans hang from the true beam ends.
const evA=P([[0,0],[.28,-10],[.36,-10],[.65,0,'out']],[[0,0],[.2,-12],[.42,7],[.6,-3],[.76,1],[.9,0]]);
const evBeam=rigid((t,s)=>({angle:evA(t,s),pivot:[16,7]}));
const evEnd=x=>rigid((t,s)=>{const [px,py]=rotate(x,7,evA(t,s),16,7);return{dx:px-x,dy:py-7};});
const pan=x=>`M${x-3.25} 15 L${x+3.25} 15 C${x+3.25} 21 ${x-3.25} 21 ${x-3.25} 15 Z`;
const side=(n,x)=>{const pose=evEnd(x);return[...line(n+'-hanger',[x,7],[x,15],{pose}),...framed(n+'-pan',pan(x),pan(x),{pose})];};
icons.push(make('model-evaluation','Model evaluation','A balance weighs the model, tips, and settles level.',[
 ...line('post',[16,7],[16,26]),...line('base',[11,26],[21,26]),
 ...line('beam',[7.5,7],[24.5,7],{pose:evBeam}),...side('left',7.5),...side('right',24.5)],.65,1));

// confidence-score: the interval tightens around the estimate as the score marker advances.
const csL=P([[0,5.5],[.42,9,'out']],[[0,5.5],[.08,7.8,'pass'],[.22,9]]);
const csR=P([[0,26.5],[.42,23,'out']],[[0,26.5],[.08,24.2,'pass'],[.22,23]]);
const csM=P([[0,11],[.15,11],[.60,21,'out']],[[0,11],[.06,11],[.2,18.5,'pass'],[.3,21]]);
const slide=(fn,x0)=>rigid((t,s)=>({dx:fn(t,s)-x0}));
icons.push(make('confidence-score','Confidence score','The interval tightens around the estimate and the score marker advances.',[
 ...framed('estimate',circle(16,12,3.25)),
 ...tube('whisker-l',(t,s)=>segment([csL(t,s),12],[12.75,12])),
 ...tube('whisker-r',(t,s)=>segment([19.25,12],[csR(t,s),12])),
 ...line('cap-l',[5.5,8.5],[5.5,15.5],{pose:slide(csL,5.5)}),
 ...line('cap-r',[26.5,8.5],[26.5,15.5],{pose:slide(csR,26.5)}),
 ...line('axis',[5,24],[27,24]),
 ...line('marker',[11,21.5],[11,26.5],{pose:slide(csM,11)})],.6,.65,['whisker','cap','marker']));

// inference: inputs are consumed into the node, it works, and the output arrow extends.
const inR=P([[0,5]],[[0,5],[.28,5],[.38,5.7],[.48,4.7],[.56,5.1],[.64,5]]);
const inTip=P([[0,25.5],[.30,25.5],[.58,28,'out']],[[0,25.5],[.3,25.5],[.36,24.8],[.48,28.6],[.58,28]]);
const eat=d=>P([[0,0],[.32,1,'in']],[[0,0],[d,0],[d+.12,1]]);
const feed=(n,tail,deg,d)=>{const g=eat(d),a=deg*Math.PI/180,head=[15+5*Math.cos(a),16+5*Math.sin(a)];
 return tube(n,(t,s)=>{const p=Math.min(g(t,s),.97);return segment([mix(tail[0],head[0],p),mix(tail[1],head[1],p)],head);},{visibility:(t,s)=>g(t,s)<.97?1:0});};
icons.push(make('inference','Inference','Inputs are consumed by the node and the output extends.',[
 ...feed('in-top',[3.5,8.5],-140,.02),...feed('in-mid',[3.5,16],180,.1),...feed('in-low',[3.5,23.5],140,.18),
 ...framed('node',(t,s)=>circle(15,16,inR(t,s))),
 ...tube('output',(t,s)=>{const x=inTip(t,s),r=inR(t,s);return `M${15+r} 16 L${x} 16 L${x-2} 13.5 L${x} 16 L${x-2} 18.5`;})],.7,.75,['in','output']));

// batch-inference: the queued cards move into the fixed collector as one batch (Strong: two groups).
const biA=P([[0,0],[.40,2.5,'out']],[[0,0],[.1,-1],[.3,3],[.42,2.5]]);
const biB=P([[0,0],[.40,2.5,'out']],[[0,0],[.2,0],[.3,-1],[.5,3],[.62,2.5]]);
const card=(n,y,fn)=>framed(n,rect(4.5,y,12,5,2.5),undefined,{pose:rigid((t,s)=>({dx:fn(t,s)}))});
icons.push(make('batch-inference','Batch inference','A queue of cards is pushed into the collector together.',[
 ...card('card-1',5,biA),...card('card-2',13.5,biA),...card('card-3',22,biB),
 ...tube('collector','M22.5 6 L24.5 6 C25.05 6 25.5 6.45 25.5 7 L25.5 25 C25.5 25.55 25.05 26 24.5 26 L22.5 26')],.6,1,['card']));
