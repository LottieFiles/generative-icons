// Legal & compliance batch D: disclaimer, copyright, whistleblower, evidence-bag, legal-hold.
import {make,toggle,dir,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,rotate,holes,annulus,bothFill,line,arcN,L} from './helpers.mjs';

// ---------- DISCLAIMER ----------
// A footnote asterisk (three arms through (10.5,16), radius 5.5) beside three lines of fine print.
// Light turns it one sixth of a turn; Strong winds back, spins two sixths and pops to 115%.
const AC=[10.5,16];
const spinA=F([[0,0],[.06,0],[.5,60,'smooth']],[[0,0],[.1,-15],[.36,128],[.5,117],[.6,120]]);
const popA=F([[0,1],[.5,1]],[[0,1],[.1,.95],[.3,1.15],[.46,.98],[.56,1]]);
const armPose=(x,y,t,s)=>{const k=popA(t,s),[rx,ry]=rotate(x,y,spinA(t,s),...AC);return [AC[0]+(rx-AC[0])*k,AC[1]+(ry-AC[1])*k];};
const arm=a=>{const c=Math.cos(a*Math.PI/180)*5.5,q=Math.sin(a*Math.PI/180)*5.5;return [[AC[0]-c,AC[1]-q],[AC[0]+c,AC[1]+q]];};
const disclaimer=make('disclaimer','Disclaimer',
 'Light: the asterisk turns one sixth of a turn beside the fine print. Strong: it winds back, spins two sixths past its stop and pops to 115% before settling.',[
 ...posed([90,30,150].flatMap((a,i)=>line('arm-'+i,...arm(a))),armPose),
 ...line('print-0',[19,12],[27.5,12]),...line('print-1',[19,16],[27.5,16]),...line('print-2',[19,20],[24.5,20])
],.5,.6);
disclaimer.duotoneFill=false;disclaimer.cycleDegrees=60;

// ---------- COPYRIGHT ----------
// The © mark: a ring (r 11) around a C (r 5.2, open to the right). Light presses it to 93% and back.
// Strong lifts it to 115%, slams it to 90% and settles.
const pressC=F([[0,1],[.06,1],[.2,.93,'out'],[.44,1,'smooth']],[[0,1],[.12,1.15],[.24,.9,'hit'],[.36,1.04],[.48,1]]);
const copyright=make('copyright','Copyright',
 'Light: the © mark is pressed to 93% and springs back. Strong: it lifts to 115%, slams down to 90% and settles.',
 posed([...framed('ring',circle(16,16,11),annulus(16,16,11,2.6)),...tube('c',arcN(16,16,5.2,45,315,4))],(x,y,t,s)=>{const k=pressC(t,s);return [16+(x-16)*k,16+(y-16)*k];}),.44,.48);
copyright.duotoneFill=true;

// ---------- WHISTLEBLOWER ----------
// A whistle: chamber (15,19.5) r 5.6 with a mouthpiece to (27.5,16.4). Blowing tilts it 5° about
// the mouthpiece end and one sound arc blinks. Strong jerks it 12° with a snap and both arcs burst.
const WP=[27.5,16.4],WC=[15,19.5];
const whistleD='M15 13.9 L25 13.9 C26.38 13.9 27.5 15.02 27.5 16.4 C27.5 17.78 26.38 18.9 25 18.9 L20.568 18.9 '+L(arcN(15,19.5,5.6,-6.15,270,4))+' Z';
const blow=F([[0,0],[.06,0],[.2,5,'out'],[.5,0,'smooth']],[[0,0],[.06,-3],[.16,12,'hit'],[.28,-4],[.4,2],[.5,0]]);
const arc0=F([[0,0],[.08,0],[.16,1,'out'],[.34,1],[.46,0,'smooth']],[[0,0],[.1,0],[.16,1],[.4,1],[.5,0]]);
const arc1=F([[0,0],[.5,0]],[[0,0],[.16,0],[.22,1],[.44,1],[.56,0]]);
const burst=F([[0,1],[.5,1]],[[0,1],[.1,.75],[.32,1]]);
const arcPose=(x,y,t,s)=>{const k=burst(t,s);return [WC[0]+(x-WC[0])*k,WC[1]+(y-WC[1])*k];};
const whistleblower=make('whistleblower','Whistleblower',
 'Light: the whistle tilts 5° as it is blown and one sound arc blinks. Strong: it jerks 12° with a snap, shakes, and both sound arcs burst outward.',[
 ...posed(framed('whistle',whistleD),rigid((t,s)=>({angle:blow(t,s),pivot:WP}))),
 ...shown(posed(tube('sound-0',arcN(...WC,9,220,262,2)),arcPose),arc0),
 ...shown(posed(tube('sound-1',arcN(...WC,12,220,262,2)),arcPose),arc1)
],.5,.56);
whistleblower.duotoneFill=true;

// ---------- EVIDENCE BAG ----------
// A bag with a zip band across its top and a label. Sealing slides the zip pull 11 units across the
// band; the sealed bag is retained and the reverse slides it back. Strong snaps it shut and the bag jolts.
const BAG=rect(7,5,18,22.5,2),LABEL=rect(10.5,16,11,7,1);
const pullX=dir(F([[0,10.5],[.06,10.5],[.6,21.5,'smooth']],[[0,10.5],[.08,9.9],[.26,21.5,'hit'],[.36,21],[.44,21.5]]),F([[0,21.5],[.55,10.5,'smooth']],[[0,21.5],[.08,22.1],[.26,10.5,'hit'],[.36,11],[.44,10.5]]));
const bagJolt=dir(F([[0,0],[.6,0]],[[0,0],[.26,0],[.29,.4,'hit'],[.4,0]]),F([[0,0],[.55,0]],[[0,0],[.26,0],[.29,.4,'hit'],[.4,0]]));
const evidenceBag=toggle('evidence-bag','Evidence bag',
 'Light: the zip pull slides across the band and seals the bag. Strong: it is snapped shut and the bag jolts as it seats. The sealed bag is retained; the reverse unseals it.',
 posed([...framed('bag',BAG,holes(BAG,rect(8.3,6.3,15.4,19.9,.9))),...tube('band','M7 10.5 L25 10.5'),
  ...framed('label',LABEL,holes(LABEL,rect(11.8,17.3,8.4,4.4,.4))),
  ...bothFill('pull',(t,s,r)=>rect(pullX(t,s,r)-1.7,8.8,3.4,3.4,.9))],
  rigid((t,s,r)=>({dy:bagJolt(t,s,r)}))),
 .6,.44,'open','sealed',.55,.44,{forward:'Seal',reverse:'Unseal'});
evidenceBag.duotoneFill=true;

// ---------- LEGAL HOLD ----------
// A document beside a padlock. Placing the hold closes the shackle: it drops 2.6 units into the body
// while its short leg extends. Stateful: released → held; the reverse lifts the shackle out.
const held=dir(F([[0,0],[.06,0],[.4,1,'smooth']],[[0,0],[.1,-.12],[.26,1,'hit'],[.3,1]]),F([[0,1],[.4,0,'smooth']],[[0,1],[.06,1],[.24,-.1],[.34,0]]));
const shackleD=(t,s,r)=>{const u=held(t,s,r),dy=-2.6*(1-u),leg=17.6+.9*u;return `M21.4 18.5 L21.4 ${16.5+dy} C21.4 ${13.7+dy} 25.6 ${13.7+dy} 25.6 ${16.5+dy} L25.6 ${leg+dy}`;};
const bodyJolt=dir(F([[0,0],[.4,0]],[[0,0],[.26,0],[.29,.4,'hit'],[.4,0]]),F([[0,0],[.4,0]],[[0,0],[.34,0]]));
const DOC=rect(4,5,12.5,22,2);
const legalHold=toggle('legal-hold','Legal hold',
 'Light: the padlock shackle drops into its body, placing the document on hold. Strong: it slams shut and the body jolts. The hold is retained; the reverse lifts the shackle out.',[
 ...framed('doc',DOC,holes(DOC,rect(5.3,6.3,9.9,19.4,.9))),
 ...[[10,6.5],[14,5],[18,6.5]].flatMap(([y,w],i)=>tube('line-'+i,`M7 ${y} L${7+w} ${y}`)),
 ...tube('shackle',shackleD),
 ...posed(framed('body',rect(19,18.5,9,8.5,1.8)),rigid((t,s,r)=>({dy:bodyJolt(t,s,r)})))
],.4,.4,'released','held',.4,.4,{forward:'Hold',reverse:'Release'});
legalHold.duotoneFill=true;

export const icons=[disclaimer,copyright,whistleblower,evidenceBag,legalHold];
