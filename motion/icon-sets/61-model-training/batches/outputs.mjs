import {make,state,stateTrack,F,tube,framed,rect,circle,annulus,anchor,panel,star,finish,roundedLine,bothFill} from './helpers.mjs';

const rewardTurn=F([[0,0],[.26,12],[.66,0,'smooth']],[[0,0],[.08,-10],[.26,32],[.44,-9],[.6,2],[.78,0]]);
const reward=make('reward-model','Reward model',
 'A reward star tips above two candidate outputs. Light turns it 12 degrees; Strong winds back then turns 32 degrees with a short damped response. The outputs stay fixed so the motion reads as a reward signal, not a model being deployed.',[
 ...framed('candidate-left',rect(5,22,8,5,1.5)),...framed('candidate-right',rect(19,22,8,5,1.5)),
 ...anchor(framed('reward',star(16,11,7,3.45)),[16,11],(t,s)=>({angle:rewardTurn(t,s)}))
],.7,.82);

const score=stateTrack([[0,0],[.08,0],[.8,1,'smooth']],[[0,0],[.06,.02],[.23,.7],[.4,1]],
 [[0,1],[.07,1],[.72,0,'smooth']],[[0,1],[.05,.98],[.36,0]]);
const benchmark=state('benchmark-score','Benchmark score',
 'The measured score rises beside a fixed baseline and crosses a separate benchmark rule. Light fills the full score over 0.8 s; Strong reaches most of the score by 0.23 s and completes without overshoot. Reverse returns to the earlier measured score.',[
 ...tube('benchmark','M5 10 L16 10'),...tube('benchmark-cap','M5 7.7 L5 12.3'),
 ...framed('baseline',rect(7,18,6,9,1.4)),
 ...framed('score',(t,s,r)=>{const h=4+17*score(t,s,r);return rect(21,27-h,6,h,1.4);})
],.84,.44,'baseline score','benchmark exceeded',.76,.4);

const quant=stateTrack([[0,0],[.06,0],[.8,1,'smooth']],[[0,0],[.06,.02],[.26,.78],[.42,1]],
 [[0,1],[.06,1],[.72,0,'smooth']],[[0,1],[.05,.98],[.36,0]]);
const STEP=[[5,25],[12,25],[12,20],[19,20],[19,15],[27,15]];
const quantize=state('quantize','Quantize',
 'A continuous-valued slope snaps into three discrete levels beneath three quantization cells. Light rounds into the levels gradually; Strong establishes the main steps by 0.26 s, then locks the corners. Reverse restores the continuous slope using the same fixed path topology.',[
 ...[5,13.5,22].flatMap((x,i)=>framed('level-'+i,rect(x,5,5,5,1.25))),
 ...tube('quantized-signal',(t,s,r)=>{const p=quant(t,s,r);return roundedLine(STEP.map(([x,y],i)=>{
  const xx=5+i*22/5,yy=25-i*10/5;return[xx+(x-xx)*p,yy+(y-yy)*p];}),.7);})
],.84,.46,'continuous values','discrete levels',.76,.4);

// A rigid coefficient cartridge seats against the shelf. Its four coefficients
// move with the housing; the arrow is part of the fixed loading guide.
const seat=F([[0,0],[.2,-1],[.48,0,'smooth']],[[0,0],[.1,-.2],[.27,-3],[.49,.4,'hit'],[.62,-.2],[.78,0]]);
const weights=make('pretrained-weights','Pretrained weights',
 'A coefficient cartridge is seated on a fixed loading shelf. Light lifts it one unit and seats it; Strong lifts three units, lands with one small rebound and rests. All four stored coefficients stay rigidly attached to the cartridge.',[
 ...tube('shelf','M4.5 28 L23.5 28'),
 ...tube('load-guide','M26 7 L26 22'),...tube('load-tip','M23.5 19.5 L26 22 L28.5 19.5'),
 ...anchor([...panel('cartridge',5,8,15,16,1.8),
  ...[[9.4,12.5],[15.6,12.5],[9.4,19.5],[15.6,19.5]].flatMap(([x,y],i)=>bothFill('coefficient-'+i,rect(x-1.25,y-1.25,2.5,2.5,.65)))],[12.5,24],(t,s)=>({dy:seat(t,s)}))
],.52,.82);

const converge=stateTrack([[0,0],[.08,0],[.86,1,'smooth']],[[0,0],[.06,.02],[.24,.7],[.46,1]],
 [[0,1],[.07,1],[.78,0,'smooth']],[[0,1],[.05,.98],[.4,0]]);
const convergence=state('convergence','Convergence',
 'Upper and lower error trajectories approach a shared target region while their starting observations stay fixed. Light closes the error gradually; Strong removes most of the error by 0.24 s and settles monotonically. Reverse restores the earlier wider error range.',[
 ...framed('target',circle(25.5,16,3.1),annulus(25.5,16,3.1,2.6)),
 ...tube('upper-trajectory',(t,s,r)=>{const y=6+8*converge(t,s,r);return `M5 6 C10 6 11 ${y} 19.5 ${y}`;}),
 ...tube('lower-trajectory',(t,s,r)=>{const y=26-8*converge(t,s,r);return `M5 26 C10 26 11 ${y} 19.5 ${y}`;})
],.9,.5,'wide error range','converged range',.82,.44);

export const icons=[reward,benchmark,quantize,weights,convergence].map(finish);
