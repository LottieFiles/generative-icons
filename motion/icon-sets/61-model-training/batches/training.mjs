import {make,state,stateTrack,F,P,tube,framed,rect,circle,annulus,holes,bothFill,fill,anchor,panel,star,finish,polygon} from './helpers.mjs';

// GPU modules share their translating boundary with their bus terminals.
const load=F([[0,0],[.25,.8],[.7,0,'smooth']],[[0,0],[.08,-.2],[.26,1.8],[.44,-.35],[.62,.12],[.78,0]]);
const coords=[[9,9],[23,9],[9,23],[23,23]];
const centre=(i,t,s)=>coords[i].map(v=>v+(v<16?1:-1)*load(t,s));
const cluster=make('gpu-cluster','GPU cluster',
 'Four linked compute modules draw inward under a synchronized load and release. Light travels 0.8 units per axis; Strong prepares outward, drives 1.8 units inward, rebounds once and rests. Bus terminals are derived from the actual moving module edges.',[
 ...[[0,1],[2,3]].flatMap(([a,b],i)=>tube('bus-horizontal-'+i,(t,s)=>{const p=centre(a,t,s),q=centre(b,t,s);return `M${p[0]+3.25} ${p[1]} L${q[0]-3.25} ${q[1]}`;})),
 ...[[0,2],[1,3]].flatMap(([a,b],i)=>tube('bus-vertical-'+i,(t,s)=>{const p=centre(a,t,s),q=centre(b,t,s);return `M${p[0]} ${p[1]+3.25} L${q[0]} ${q[1]-3.25}`;})),
 ...coords.flatMap(([x,y],i)=>anchor(framed('processor-'+i,rect(x-3.25,y-3.25,6.5,6.5,1.25),holes(rect(x-3.8,y-3.8,7.6,7.6,1.8),rect(x-1.1,y-1.1,2.2,2.2,.4))),[x,y],(t,s)=>({dx:centre(i,t,s)[0]-x,dy:centre(i,t,s)[1]-y})))
],.74,.82);

const paramMoves=[
 F([[0,0],[.3,2.5],[.72,0,'smooth']],[[0,0],[.08,-.5],[.28,6],[.48,-.6],[.78,0]]),
 F([[0,0],[.34,-2.5],[.76,0,'smooth']],[[0,0],[.1,.5],[.33,-6],[.54,.6],[.84,0]]),
 F([[0,0],[.38,2.5],[.8,0,'smooth']],[[0,0],[.12,-.5],[.38,6],[.6,-.6],[.9,0]])
];
const paramY=[9,22,12];
const hyper=make('hyperparameter','Hyperparameter',
 'Three independent parameter handles sweep candidate values and return to the selected configuration. Light explores 2.5 units; Strong prepares against each direction and explores six units with a staggered response. Rails stop at each handle boundary, preserving the open counters.',[
 ...[6,16,26].flatMap((x,i)=>[
  ...tube('rail-top-'+i,(t,s)=>`M${x} 4.5 L${x} ${paramY[i]+paramMoves[i](t,s)-2.4}`),
  ...tube('rail-bottom-'+i,(t,s)=>`M${x} ${paramY[i]+paramMoves[i](t,s)+2.4} L${x} 27.5`),
  ...anchor(framed('handle-'+i,rect(x-2.5,paramY[i]-2.4,5,4.8,1.25)),[x,paramY[i]],(t,s)=>({dy:paramMoves[i](t,s)}))
 ])
],.84,.94);

const stoppingY=stateTrack([[0,0],[.08,0],[.76,13,'smooth']],[[0,0],[.06,0],[.28,11],[.42,13]],
 [[0,13],[.06,13],[.7,0,'smooth']],[[0,13],[.06,13],[.36,0]]);
const stopping=state('early-stopping','Early stopping',
 'A stop block descends to intercept the flattened loss trace. Light lowers it through 13 units; Strong drops through the main travel by 0.28 s, brakes at the curve and holds. Reverse lifts the stop block clear and returns to monitoring.',[
 ...tube('axes','M5 5 L5 28 L28 28'),...tube('plateau','M9 9 C11 20 15 23 21 23'),
 ...anchor(framed('stop-block',rect(20.5,3.5,7,7,1.25)),[24,7],(t,s,r)=>({dy:stoppingY(t,s,r)}))
],.8,.46,'monitoring','stopped at plateau',.74,.4);

const filter=polygon([[6,5],[26,5],[19,15.5],[19,18],[13,18],[13,15.5]],.65);
const filterSolid=holes(polygon([[4.9,3.7],[27.1,3.7],[20.3,16],[20.3,19.3],[11.7,19.3],[11.7,16]],.85),
 polygon([[8.8,6.3],[23.2,6.3],[17.7,14.8],[17.7,16.7],[14.3,16.7],[14.3,14.8]],.5));
const dropY=P([[0,0],[.06,0],[.76,16,'linear'],[.81,16],[.82,0,'linear']],
 [[0,0],[.04,0],[.42,16,'linear'],[.46,16],[.47,0,'linear'],[.52,0],[.9,16,'linear'],[.94,16],[.95,0,'linear']]);
const dropAlpha=P([[0,1],[.65,1],[.78,0],[.85,0],[1,1,'smooth']],
 [[0,1],[.34,1],[.44,0],[.48,0],[.53,1],[.82,1],[.92,0],[.98,0],[1.1,1,'smooth']]);
const distill=make('distillation','Distillation',
 'A knowledge packet passes through a wide teacher funnel into a small student receiver. Light sends one packet over 0.76 s; Strong sends two faster packets. Each packet leaves the channel before an invisible recycle; the funnel and receiver remain visible throughout.',[
 ...framed('teacher-filter',filter,filterSolid),
 ...framed('student',circle(16,25.2,3.3),annulus(16,25.2,3.3,2.6)),
 ...anchor(bothFill('knowledge-packet',circle(16,8,1.05)).map(p=>({...p,visibility:dropAlpha})),[16,8],(t,s)=>({dy:dropY(t,s)}))
],1.04,1.14);

const adapterX=stateTrack([[0,0],[.07,0],[.72,-4,'smooth']],[[0,0],[.08,.5],[.28,-4.8],[.48,-4]],
 [[0,-4],[.07,-4],[.66,0,'smooth']],[[0,-4],[.08,-4.4],[.28,.4],[.46,0]]);
const adapter=state('adapter-layer','Adapter layer',
 'A compact adapter slides left between two fixed model layers and stays aligned. Light moves four units quietly; Strong pulls back, inserts past the alignment by 0.8 units and seats. Reverse extracts it to the right with a separate release and settle.',[
 ...framed('base-upper',rect(5,4.5,22,5,1.6)),...framed('base-lower',rect(5,22.5,22,5,1.6)),
 ...anchor([...framed('adapter',rect(12,13.5,16,5,1.4)),...tube('adapter-pin','M12 16 L9.5 16')],[20,16],(t,s,r)=>({dx:adapterX(t,s,r)}))
],.76,.52,'adapter released','adapter inserted',.7,.5);

export const icons=[cluster,hyper,stopping,distill,adapter].map(finish);
