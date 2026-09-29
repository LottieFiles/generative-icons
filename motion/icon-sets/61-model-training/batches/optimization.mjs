import {make,state,stateTrack,F,tube,framed,bothFill,fill,circle,rect,arc,annulus,anchor,panel,rotate,finish} from './helpers.mjs';

// Epoch counter: the repeat arrows orbit a fixed tally capsule. The full revolutions
// are sampled paths so the final drawing is exactly the initial drawing.
const epochTurn=F([[0,0],[.08,0],[.86,360,'smooth']],[[0,0],[.08,-16],[.72,720],[.86,720]]);
const epochPose=(x,y,t,s)=>{const a=epochTurn(t,s);return rotate(x,y,a===360||a===720?0:a,16,16);};
const orbit=[...tube('cycle-upper',arc(16,16,10.6,-155,-25)),...tube('cycle-lower',arc(16,16,10.6,25,155)),
 ...tube('head-upper','M23.02 9.76 L25.607 11.52 L25.92 8.4'),...tube('head-lower','M8.98 22.24 L6.393 20.48 L6.08 23.6')];
const epoch=make('epoch-counter','Epoch counter',
 'A completed pass turns two cycle arrows around the fixed tally. Light turns once in 0.86 s; Strong winds back 16 degrees then turns twice in 0.72 s. Both return to the same tally and arrow pose.',[
 ...panel('counter',11.3,10.3,9.4,11.4,1.7),...tube('tally','M15.5 15.1 L17 14.3 L17 18.2'),
 ...orbit.map(p=>({...p,pose:epochPose,pivot:[16,16]}))
],.9,.9);

// Loss curve: endpoint motion and the curve share one scalar; the terminal cap
// is buried in the left side of the hollow loss marker, never through its counter.
const lossP=stateTrack([[0,0],[.06,0],[.78,1,'smooth']],[[0,0],[.06,.02],[.2,.62],[.4,1]],
 [[0,1],[.06,1],[.7,0,'smooth']],[[0,1],[.05,.98],[.36,0]]);
const lossY=(t,s,r)=>10+10*lossP(t,s,r);
const loss=state('loss-curve','Loss curve',
 'The recorded loss drops from 10 to 20 on the plot and stays. Light bends down gradually; Strong commits most of the drop by 0.20 s and levels off by 0.40 s. Reverse restores the earlier loss trace along an authored rising path.',[
 ...tube('axes','M5 5 L5 28 L28 28'),
 ...tube('trace',(t,s,r)=>`M9 8 C11 ${8+8*lossP(t,s,r)} 14 ${lossY(t,s,r)} 22.4 ${lossY(t,s,r)}`),
 ...anchor(framed('endpoint',circle(25,10,2.6),annulus(25,10,2.6,2.6)),[25,10],(t,s,r)=>({dy:lossY(t,s,r)-10}))
],.8,.44,'high loss','lower loss',.72,.4);

// Learning-rate schedule: later step sizes shrink; the first step stays fixed.
// Wide filled Solid blocks replace outlined capsules, not their interior details.
const schedule=i=>stateTrack([[0,0],[.08+i*.12,0],[.58+i*.12,1,'smooth']],
 [[0,0],[.03+i*.035,0],[.3+i*.08,1]],
 [[0,1],[.06+(1-i)*.1,1],[.6+(1-i)*.1,0,'smooth']],
 [[0,1],[.04+(1-i)*.04,1],[.34+(1-i)*.06,0]]);
const rates=[schedule(0),schedule(1)];
const learning=state('learning-rate','Learning rate',
 'Three step-size bars begin equal. The later two shrink into a decay schedule while the first stays fixed. Light contracts them in sequence; Strong starts them close together and drives them to their exact values. Reverse increases the later step sizes in the opposite order.',[
 ...framed('initial-step',rect(5,5,22,4.5,1.6)),
 ...[0,1].flatMap(i=>framed('step-'+i,(t,s,r)=>rect(5,13.75+i*8.75,22-(i?15:8)*rates[i](t,s,r),4.5,1.6)))
],.76,.46,'constant rate','decay schedule',.76,.46);

// Gradient direction is a gesture, not a value advancing and then being undone.
const gradientNudge=F([[0,0],[.24,1.2],[.65,0,'smooth']],[[0,0],[.08,-.6],[.28,2.7,'hit'],[.48,-.3],[.7,0]]);
const gradient=make('gradient-step','Gradient step',
 'A descending vector points to the next optimization step beside a fixed staircase. Light presses 1.2 units along each axis; Strong draws back, presses 2.7 units, recoils once and settles. The parameter block and stair remain fixed.',[
 ...tube('stairs','M5 10 L11 10 L11 17 L18 17 L18 24 L27 24'),
 ...framed('parameter',rect(5,4.5,6,5.5,1.2),rect(3.7,3.2,8.6,8.1,2.5)),
 ...anchor([...tube('vector','M17 5.5 L24 12.5'),...tube('vector-head','M19.8 12.5 L24 12.5 L24 8.3')],[24,12.5],(t,s)=>({dx:gradientNudge(t,s),dy:gradientNudge(t,s)}))
],.68,.74);

// Overfitting: exaggerate the fitted curve around fixed samples; the graph window
// is a real enclosed surface. Keep the Solid window open with an explicit counter.
const wiggle=F([[0,0],[.28,1.8],[.7,0,'smooth']],[[0,0],[.08,-.8],[.25,3.7],[.42,-1.4],[.62,.5],[.82,0]]);
const overfit=make('overfitting','Overfitting',
 'A fitted curve flexes around three fixed observations in a plot window. Light adds a restrained 1.8-unit wave; Strong pulls the fit 3.7 units around the samples, releases it and settles. Observation points and the window never move.',[
 ...panel('plot',5,5,22,22,2.5),
 ...tube('fit',(t,s)=>{const a=wiggle(t,s);return `M8.7 19 C10 ${10-a} 13 ${10-a} 16 17 C18 ${24+a*.25} 21 ${24+a*.25} 23.3 12`; }),
 ...[[9.5,15.7],[16,17],[22,17.2]].flatMap(([x,y],i)=>bothFill('sample-'+i,circle(x,y,1.65)))
],.74,.86);

export const icons=[epoch,loss,learning,gradient,overfit].map(finish);
