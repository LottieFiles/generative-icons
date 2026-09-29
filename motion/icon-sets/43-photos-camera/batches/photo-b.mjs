// Photos & camera batch B (modes): portrait-mode, live-photo, burst-mode, panorama, exposure.
import {make,toggle,F,tube,framed,rigid,posed,shown,fill,stroke,circle,rect,polygon,bothFill,holes,capsule} from './helpers.mjs';
import {arcN,clamp01,cardParts,hillsParts,sunParts,backCardD,backCardSolidD} from './photo.mjs';

// ---------- PORTRAIT MODE ----------
// A portrait card with a person and two background bokeh discs. On: the background discs blur (grow and
// fade to 35%) while the person comes forward. Reverse brings the background back.
const headC=[16,12.2],SHOULDERS='M10.6 23.4 C10.6 19.6 12.8 18.4 16 18.4 C19.2 18.4 21.4 19.6 21.4 23.4';
const BOKEH=[[10.6,8.6],[21.4,8.8]];
const blurF=F([[0,0],[.06,0],[.46,1,'smooth']],[[0,0],[.06,0,'hold'],[.24,1,'out']]);
const blurB=F([[0,1],[.44,0,'smooth']],[[0,1],[.06,1,'hold'],[.26,0,'out']]);
const blur=(t,s,r)=>r?blurB(t,s):blurF(t,s);
const personF=F([[0,1],[.46,1]],[[0,1],[.06,1],[.2,1.09,'hit'],[.32,.98],[.42,1]]);
const personPose=(x,y,t,s,r)=>{const k=r?1:personF(t,s);return [16+(x-16)*k,17+(y-17)*k];};
const bokehPose=i=>(x,y,t,s,r)=>{const k=1+.35*blur(t,s,r),[cx,cy]=BOKEH[i];return [cx+(x-cx)*k,cy+(y-cy)*k];};
const portraitMode=toggle('portrait-mode','Portrait mode',
 'Off to on: the two background discs blur, growing and fading to 35%, while the subject stays sharp. Strong snaps the background away and the subject pops forward with one settle. Reverse restores the background.',[
 ...cardParts('card',7,4.5,18,23,2.4),
 ...BOKEH.flatMap(([x,y],i)=>posed(shown(bothFill('bokeh-'+i,circle(x,y,1.4)),(t,s,r)=>1-.65*blur(t,s,r)),bokehPose(i))),
 ...posed([...tube('head',circle(...headC,2.6)),fill('head-solid-core',circle(...headC,2.6),'solid'),...tube('shoulders',SHOULDERS),fill('shoulders-solid-core',SHOULDERS+' Z','solid')],personPose)
],.46,.42,'off','on',.44,.4);
portraitMode.duotoneFill=true;

// ---------- LIVE PHOTO ----------
// Centre dot, inner ring, six outer dashes. The dashes index one dash step (60°) while the dot pulses and
// the ring breathes. Strong winds back and indexes two steps with an overshoot (cycleDegrees 60).
const DASH=34,DR=11.4;
const spinL=F([[0,0],[.06,0],[.6,60,'smooth']],[[0,0],[.08,-14],[.36,132],[.5,120]]);
const pulse=F([[0,1],[.1,1],[.26,1.25],[.54,1,'smooth']],[[0,1],[.08,.9],[.22,1.5,'hit'],[.36,.95],[.46,1]]);
const breathe=F([[0,6.4],[.12,6.4],[.32,6.7],[.6,6.4,'smooth']],[[0,6.4],[.1,6.2],[.26,7.2,'hit'],[.4,6.3],[.5,6.4]]);
const ringD=(t,s)=>circle(16,16,breathe(t,s));
const livePhoto=make('live-photo','Live photo',
 'Light: the dashed outer ring indexes one step while the centre dot pulses and the inner ring breathes. Strong: a short wind-back, then two steps with an overshoot, a bigger pulse and a wider breath.',[
 fill('ring-interior',ringD,'tint'),...tube('ring',ringD),
 ...posed(bothFill('dot',circle(16,16,2.4)),(x,y,t,s)=>{const k=pulse(t,s);return [16+(x-16)*k,16+(y-16)*k];}),
 ...posed([0,1,2,3,4,5].flatMap(i=>tube('dash-'+i,arcN(16,16,DR,-90-DASH/2+60*i,-90+DASH/2+60*i,1))),rigid((t,s)=>({angle:spinL(t,s),pivot:[16,16]})))
],.6,.5);
livePhoto.duotoneFill=true;livePhoto.cycleDegrees=60;

// ---------- BURST MODE ----------
// One photo; two more shots pop out behind it in quick succession and stay fanned. Each back card is
// drawn only where it shows past the card in front, so nothing paints through an Outline card.
const BX=4.5,BY=11.5,BW=18,BH=13;
const o2=F([[0,0],[.06,0],[.36,2.6,'out']],[[0,0],[.06,0,'hold'],[.16,3.2,'hit'],[.28,2.6]]);
const o3=F([[0,0],[.16,0],[.48,5.2,'out']],[[0,0],[.16,0,'hold'],[.28,5.9,'hit'],[.4,5.2]]);
const burstJolt=F([[0,0],[.48,0]],[[0,0],[.14,0],[.17,-.4,'hit'],[.24,0],[.26,0],[.29,-.4,'hit'],[.38,0]]);
const burstPose=rigid((t,s)=>({dx:burstJolt(t,s)}));
const backCard=(name,offset,front)=>{
 const rel=(t,s)=>front?offset(t,s)-front(t,s):offset(t,s),org=(t,s)=>front?front(t,s):0;
 return [
  stroke(name,(t,s)=>backCardD(BX+org(t,s),BY-org(t,s),BW,BH,Math.max(.01,rel(t,s))),{visibility:(t,s)=>rel(t,s)>.35?1:0}),
  fill(name+'-solid',(t,s)=>backCardSolidD(BX+org(t,s),BY-org(t,s),BW,BH,Math.max(.4,rel(t,s))),'solid',{visibility:(t,s)=>rel(t,s)>.35?1:0})
 ];};
const burstMode=make('burst-mode','Burst mode',
 'Two extra shots pop out behind the photo in quick succession and stay fanned up to the right. Strong fires them faster with overshoot, and the photo kicks on each shot. The fan is retained; preview reset rearms.',[
 ...posed([...cardParts('card',BX,BY,BW,BH,2),...hillsParts('hills',[[7,21.4],[10.6,16.6],[13.2,19.4],[15,17.6],[20.6,21.4]])],burstPose),
 ...posed(backCard('card-2',o2),burstPose),
 ...posed(backCard('card-3',o3,o2),burstPose)
],.48,.4,['card-2','card-3']);
burstMode.duotoneFill=true;

// ---------- PANORAMA ----------
// A photo widens into a panorama whose long edges bow inward; the landscape stretches with it and a sun
// fades in as the frame widens. Reverse narrows it back to a photo.
const panoD=(x0,y0,w,h,r,bow)=>{const x1=x0+w,y1=y0+h,k=.448;return `M${x0} ${y0+r} L${x0} ${y1-r} C${x0} ${y1-r*k} ${x0+r*k} ${y1} ${x0+r} ${y1} C${x0+w/3} ${y1-bow} ${x1-w/3} ${y1-bow} ${x1-r} ${y1} C${x1-r*k} ${y1} ${x1} ${y1-r*k} ${x1} ${y1-r} L${x1} ${y0+r} C${x1} ${y0+r*k} ${x1-r*k} ${y0} ${x1-r} ${y0} C${x1-w/3} ${y0+bow} ${x0+w/3} ${y0+bow} ${x0+r} ${y0} C${x0+r*k} ${y0} ${x0} ${y0+r*k} ${x0} ${y0+r} Z`;};
const uF=F([[0,0],[.06,0],[.6,1,'smooth']],[[0,0],[.08,-.05],[.34,1.05,'hit'],[.48,1]]);
const uB=F([[0,1],[.54,0,'smooth']],[[0,1],[.08,1.04],[.32,-.04,'hit'],[.46,0]]);
const pu=(t,s,r)=>r?uB(t,s):uF(t,s);
const frameAt=u=>{const w=13+11.6*u,x0=16-w/2;return {x0,w,bow:1.6*clamp01(u)};};
const PY=8,PH=16;
const hillsAt=(t,s,r)=>{const {x0,w}=frameAt(pu(t,s,r)),a=x0+2.6,span=w-5.2;return [[0,19.2],[.32,13.8],[.5,16.6],[.68,14.4],[1,19.2]].map(([f,y])=>[a+f*span,y]);};
const sunVis=(t,s,r)=>clamp01((pu(t,s,r)-.5)/.3);
const panorama=toggle('panorama','Panorama',
 'Photo to panorama: the frame widens from 13 to 24.6 units, its long edges bow inward, the landscape stretches with it and a sun fades in. Strong overshoots the width and settles. Reverse narrows it back.',[
 fill('frame-interior',(t,s,r)=>{const {x0,w,bow}=frameAt(pu(t,s,r));return panoD(x0,PY,w,PH,2.2,bow);},'tint'),
 stroke('frame',(t,s,r)=>{const {x0,w,bow}=frameAt(pu(t,s,r));return panoD(x0,PY,w,PH,2.2,bow);}),
 fill('frame-solid',(t,s,r)=>{const {x0,w,bow}=frameAt(pu(t,s,r));return holes(panoD(x0-1.3,PY-1.3,w+2.6,PH+2.6,3.5,bow),panoD(x0+1.3,PY+1.3,w-2.6,PH-2.6,.9,bow));},'solid'),
 ...hillsParts('hills',hillsAt),
 ...posed(shown(sunParts('sun',0,12.6,1.3),sunVis),(x,y,t,s,r)=>[x+frameAt(pu(t,s,r)).x0+5,y])
],.6,.48,'photo','panorama',.54,.46);
panorama.duotoneFill=true;

// ---------- EXPOSURE ----------
// The exposure-compensation square: a diagonal divides + from −. The divider nudges toward + (brighter)
// and returns; Strong drives it farther with a hard stop and a small rebound.
const EX=rect(4,4,24,24,2.8),SQ2=Math.SQRT2;
const kShift=F([[0,0],[.06,0],[.3,1.6],[.62,0,'smooth']],[[0,0],[.06,-.3],[.24,2.3,'hit'],[.4,-.2],[.52,0]]);
const diagD=(t,s)=>{const k=kShift(t,s)*SQ2;return `M4 ${25.2-k} L${25.2-k} 4`;};
const plusHole=polygon([[8.6,7.8],[10.6,7.8],[10.6,8.6],[11.4,8.6],[11.4,10.6],[10.6,10.6],[10.6,11.4],[8.6,11.4],[8.6,10.6],[7.8,10.6],[7.8,8.6],[8.6,8.6]],[.5,.5,.2,.5,.5,.2,.5,.5,.2,.5,.5,.2]);
const exposure=make('exposure','Exposure',
 'Light: the divider slides 1.6 units toward + and eases back. Strong: a short wind-back, a 2.3-unit drive with a hard stop and a small rebound before settling.',[
 fill('dark-interior',(t,s)=>{const k=kShift(t,s)*SQ2;return polygon([[4,25.2-k],[25.2-k,4],[28,4],[28,28],[4,28]],[0,0,2.8,2.8,2.8]);},'tint'),
 stroke('frame',EX),stroke('divider',diagD),
 stroke('plus-h','M7.8 9.6 L11.4 9.6'),stroke('plus-v','M9.6 7.8 L9.6 11.4'),stroke('minus','M20.6 22.4 L24.2 22.4'),
 fill('frame-solid',(t,s)=>{const k=kShift(t,s)*SQ2;return holes(rect(2.75,2.75,26.5,26.5,4),plusHole,capsule([20.6,22.4],[24.2,22.4],2),capsule([6.2,23-k],[23-k,6.2],2));},'solid')
],.62,.52);
exposure.duotoneFill=true;

export const icons=[portraitMode,livePhoto,burstMode,panorama,exposure];
