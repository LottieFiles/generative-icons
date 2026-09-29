import {make,state,stateTrack,F,tube,framed,rect,circle,holes,bothFill,stroke,anchor,panel,finish,polygon} from './helpers.mjs';

const split=stateTrack([[0,0],[.08,0],[.7,1,'smooth']],[[0,0],[.08,-.1],[.32,1.15],[.52,1]],
 [[0,1],[.1,1],[.64,0,'smooth']],[[0,1],[.08,1.08],[.3,-.06],[.48,0]]);
const validation=state('validation-split','Validation split',
 'A held-out sample block separates to the right while the training blocks move left. Light splits them quietly; Strong takes a small preparation, opens the separation farther, and settles to the same partition. Reverse reunites the three blocks.',[
 ...[5,14,23].flatMap((y,i)=>anchor(framed('partition-'+i,rect(7,y,15,4,1.4)),[14.5,y+2],(t,s,r)=>({dx:(i===2?4:-2)*split(t,s,r)})))
],.74,.56,'combined samples','held-out validation',.68,.52);

const augmentTurn=F([[0,0],[.3,12],[.7,0,'smooth']],[[0,0],[.08,-7],[.27,25],[.47,-7],[.64,2],[.8,0]]);
const augmentLift=F([[0,0],[.3,-.4],[.7,0,'smooth']],[[0,0],[.08,.3],[.27,-1],[.5,.2],[.8,0]]);
const augmentation=make('data-augmentation','Data augmentation',
 'The source image stays fixed while a second view rotates about its own centre. Light tips the copy 12 degrees; Strong prepares in the other direction, turns it 25 degrees and lifts it one unit before settling.',[
 ...panel('source',5,13,9,12,1.5),...bothFill('source-mark',polygon([[7.6,21.8],[9.5,18.9],[11.4,21.8]],.35)),...bothFill('source-sun',circle(10.9,16.5,.75)),
 ...anchor([...panel('augmented',18,6,8,11,1.5),...bothFill('augmented-mark',polygon([[20.35,14],[22,11.4],[23.65,14]],.3)),...bothFill('augmented-sun',circle(23,9,.7))],[22,11.5],(t,s)=>({angle:augmentTurn(t,s),dy:augmentLift(t,s)}))
],.74,.84);

const tagY=stateTrack([[0,-9],[.07,-9],[.7,0,'smooth']],[[0,-9],[.08,-9.5],[.3,.8],[.5,0]],
 [[0,0],[.08,0],[.65,-9,'smooth']],[[0,0],[.08,.5],[.3,-9.5],[.48,-9]]);
const tagD=polygon([[18,18],[22,14],[27,14],[27,22],[22,22]],.7);
const labeling=state('data-labeling','Data labeling',
 'An attached annotation tag moves from the first record to the selected lower record. Light slides down nine units; Strong lifts slightly, moves down decisively and settles. Reverse returns the tag to the first record. Its short connector follows the tag and stays attached to the record edge.',[
 ...panel('record',5,5,12,22,1.8),...tube('record-first','M8.5 10 L12.5 10'),...tube('record-second','M8.5 17 L12.5 17'),...tube('record-third','M8.5 23 L12.5 23'),
 ...tube('connector',(t,s,r)=>`M17 ${18+tagY(t,s,r)} L18 ${18+tagY(t,s,r)}`),
 ...anchor([...framed('tag',tagD,holes(tagD,circle(24,18,1.4))),...bothFill('tag-hole',circle(24,18,1.1)).filter(p=>p.style==='outline')],[18,18],(t,s,r)=>({dy:tagY(t,s,r)}))
],.74,.54,'first record tagged','selected record tagged',.69,.52);

const tokenize=stateTrack([[0,0],[.07,0],[.76,1,'smooth']],[[0,0],[.06,.02],[.28,.85],[.42,1]],
 [[0,1],[.07,1],[.7,0,'smooth']],[[0,1],[.05,.98],[.36,0]]);
const tokenX=(i,p)=>(4.3+7.8*i)*(1-p)+(4.2+9.1*i)*p;
const tokenD=i=>(t,s,r)=>{const p=tokenize(t,s,r);return rect(tokenX(i,p),14,7.8-2.4*p,10,1.4);};
const tokenizer=state('tokenizer','Tokenizer',
 'One joined word is split into three enclosed token cells. Light opens the two boundaries gradually; Strong separates most of the word by 0.28 s, then locks the boundaries. Solid tokens are complete filled blocks. Reverse joins the cells into the original word.',[
 ...tube('word','M6 6.5 L26 6.5'),
 ...[0,1,2].flatMap(i=>framed('token-'+i,tokenD(i)))
],.8,.46,'joined word','separate tokens',.74,.4);

const shuffle=i=>F([[0,0],[.14+i*.035,0],[.34+i*.035,(i%2?-1:1)*2],[.72+i*.035,0,'smooth']],
 [[0,0],[.08+i*.025,(i%2?1:-1)*.6],[.26+i*.025,(i%2?-1:1)*4],[.48+i*.025,(i%2?1:-1)*2.2],[.8+i*.025,0]]);
const shuffleDrives=[0,1,2].map(shuffle);
const shuffling=make('data-shuffle','Data shuffle',
 'Three sample packets are mixed in opposing horizontal lanes. Light makes one two-unit exchange; Strong crosses four units, mixes back through a second shorter pass and parks the packets. Each packet remains rigid and its vertical lane remains clear.',[
 ...[4.5,13.5,22.5].flatMap((y,i)=>anchor(framed('packet-'+i,rect(i===1?10:8,y,i===1?12:16,5,1.6)),[16,y+2.5],(t,s)=>({dx:shuffleDrives[i](t,s)})))
],.84,.92);

export const icons=[validation,augmentation,labeling,tokenizer,shuffling].map(finish);
