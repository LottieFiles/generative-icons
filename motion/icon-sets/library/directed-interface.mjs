import {circle,rect,polygon,capsule,hole,holes,stroke,fill,tube,line,framed,rigid,rotate,mix,arc,annulus} from './drawing.mjs';
import {track,flow,drive as motion,timePair,revise,resultAction,has,rePose,clamp} from './direction.mjs';
export function directInterface(icons){for(const icon of icons){const n=icon.name;
 if(n==='settings'){
  const pose=rigid((t,s)=>({angle:motion(t,s,[[0,0],[.26,60,'out']],[[0,0],[.045,-5,'launch'],[.16,40,'pass'],[.32,127],[.40,118],[.49,120]])}));
  rePose(icon,()=>true,pose);revise(icon,{light:.30,strong:.52,posesLight:[0,.04,.10,.17,.26,.30],posesStrong:[0,.045,.16,.26,.32,.40,.52],intent:'The rigid gear sweeps through two detents in one movement and springs into its final index.',cycleDegrees:60});
 }
 if(n==='camera'){
  const close=(t,s)=>motion(t,s,[[0,0],[.04,0],[.10,1,'in'],[.12,1],[.24,0,'out']],[[0,0],[.045,.02,'launch'],[.085,1],[.13,1],[.20,-.05],[.27,0]]);
  const button=rigid((t,s)=>({dy:motion(t,s,[[0,0],[.04,.55,'out'],[.18,.55],[.28,0]],[[0,0],[.03,-.12],[.09,.95,'hit'],[.19,.72],[.42,0]])}));
  const aperture=(t,s)=>{const q=close(t,s),r=3.15*(1-q)+.12;return polygon([[16,18-r],[16+r,18],[16,18+r],[16-r,18]],Math.min(.8,r*.4));};
  const body=icon.parts.find(p=>p.name==='camera-body').d;
  revise(icon,{light:.30,strong:.42,posesLight:[0,.04,.08,.11,.17,.24,.3],posesStrong:[0,.03,.085,.13,.19,.23,.42],intent:'The button dips, strikes and settles; four iris facets snap shut, hold and spring back open inside a rigid lens.',parts:[fill('camera-interior',body,'tint'),stroke('camera-body',body),stroke('camera-lens',circle(16,18,4.7)),fill('camera-solid',hole(body,circle(16,18,4.5))),fill('camera-iris',(t,s)=>hole(circle(16,18,3.8),aperture(t,s)),'outline'),fill('camera-solid-iris',(t,s)=>hole(circle(16,18,3.8),aperture(t,s))),...line('camera-button',[22.7,8.2],[24.2,8.2],{pose:button})]});
 }
 if(n==='home'){
  const shell=icon.parts.find(p=>p.name==='home-shell').d,door=rect(13,17.5,6,8.5,1);
  const angle=(t,s)=>motion(t,s,[[0,0],[.06,.08],[.22,.65,'out'],[.31,.65],[.51,0,'in']],[[0,0],[.04,.04],[.20,1.22],[.31,1.08],[.64,0]]);
  const leaf=(x,y,t,s)=>[13+(x-13)*Math.cos(angle(t,s)),y];
  revise(icon,{light:.54,strong:.70,intent:'A separate door leaf swings open from its left hinge, overshoots and settles, then closes in one continuous move; the wall does not change shape.',parts:[fill('home-interior',shell,'tint'),stroke('home-shell',shell),stroke('home-doorway',door),fill('home-solid-wall',hole(shell,door)),...tube('home-door-leaf',door,{pose:leaf})]});
 }
 if(n==='calendar'){
  const page=rect(6,8,20,18.5,2.4),q=(t,s)=>motion(t,s,[[0,0],[.06,.05],[.28,.65],[.42,1,'out']],[[0,0],[.045,.04],[.18,.62,'pass'],[.32,1]]);
  const turn=(x,y,t,s)=>[x,14.5+(y-14.5)*Math.cos(Math.PI*q(t,s))],v=(t,s)=>Math.max(0,Math.min(1,(.88-q(t,s))*8));
  const leaf='M6 14.5 L26 14.5 L26 24.1 C26 25.425 24.925 26.5 23.6 26.5 L8.4 26.5 C7.075 26.5 6 25.425 6 24.1 Z',tile=rect(10,18,3.5,3.5,.8),next=(t,s)=>clamp((q(t,s)-.48)*12);
  revise(icon,{light:.46,strong:.37,intent:'A complete date leaf lifts, flips fast over the binding line and lands; its date mark stays on the paper and the next page is exposed.',parts:[...framed('calendar-frame',page,hole(page,rect(8,14.5,16,10,.9))),...line('calendar-header',[6.5,14.5],[25.5,14.5]),fill('calendar-next-day',rect(18,18,3.5,3.5,.8),'outline',{visibility:next}),fill('calendar-next-day-solid',rect(18,18,3.5,3.5,.8),'solid',{visibility:next}),stroke('calendar-turning-page',leaf,{pose:turn,visibility:v}),fill('calendar-turning-page-solid',hole(leaf,tile),'solid',{pose:turn,visibility:v}),fill('calendar-day',tile,'outline',{pose:turn,visibility:v}),...line('calendar-binding-left',[11,5.5],[11,10]),...line('calendar-binding-right',[21,5.5],[21,10])]});resultAction(icon,p=>has(p,'turning','calendar-day','next-day'));
 }
 if(n==='clock'){
  const a=(t,s)=>motion(t,s,[[0,0],[.04,0],[.34,90,'out']],[[0,0],[.035,0],[.085,22,'pass'],[.19,90]]);
  rePose(icon,p=>has(p,'minute'),rigid((t,s)=>({angle:a(t,s)})));rePose(icon,p=>has(p,'hour'),rigid((t,s)=>({angle:a(t,s)/12})));
  revise(icon,{light:.38,strong:.24,intent:'Both hands advance in one fast continuous movement at a true 12:1 ratio and retain the new time; preview reset is a separate action.'});resultAction(icon,p=>has(p,'minute','hour'));
 }
 if(n==='image'){
  const shift=(t,s)=>motion(t,s,[[0,0],[.04,0],[.29,2.4,'out']],[[0,0],[.04,-.35,'launch'],[.20,2.8],[.28,2.3],[.34,2.4]]);
  const hills=(t,s)=>{const d=shift(t,s);return polygon([[8,22],[8,22-d*1.3],[13-d,15.5],[17-d,20],[20.5-d,17.5],[24-d,22],[24,22]],[0,0,.9,.7,.8,0,0]);};
  for(const p of icon.parts.filter(p=>has(p,'hills')))p.d=hills;
  rePose(icon,p=>has(p,'sun'),rigid((t,s)=>({dx:-shift(t,s)})));
  revise(icon,{light:.33,strong:.40,intent:'Landscape and sun pan together in one committed move into a new crop inside a fixed image frame, then hold.'});resultAction(icon,p=>has(p,'hills','sun'));
 }
 if(n==='mail'){
  const flap=(t,s)=>motion(t,s,[[0,0],[.06,0],[.22,1,'out'],[.35,1],[.54,0,'in']],[[0,0],[.055,-.04,'launch'],[.15,1.06],[.27,.98],[.46,.98],[.62,0]]);
  const d=(t,s)=>polygon([[7,11.5],[16,17.5-12*flap(t,s)],[25,11.5]],.7);
  const lift=(t,s)=>s?flow([[0,0],[.22,0],[.32,3],[.40,2.5],[.52,0]],t):track([[0,0],[.20,0],[.28,.7],[.38,0]],t);
  const letter=(t,s)=>rect(10,11.5-lift(t,s),12,Math.max(.02,lift(t,s)),.65),letterVisible=(t,s)=>Math.min(1,lift(t,s)*4);
  const body=rect(6,11.5,20,14,2.4);
  revise(icon,{light:.57,strong:.66,intent:'The full triangular flap springs open above its seam, a letter pops from the pocket and settles; the letter returns before the flap closes.',parts:[...framed('mail-flap',d,d),...framed('mail-letter',letter,letter,{visibility:letterVisible}),...framed('mail-pocket',body,body)]});
 }
 if(n==='message'){
  const dot=(i,t,s)=>{const q=motion(t-i*(s?.10:.065),s,[[0,0],[.045,0],[.115,1],[.19,0]],[[0,0],[.04,-.12],[.12,1],[.18,-.12],[.23,.05],[.27,0]]);return circle(11+i*5,15-q*(s?1.6:.35),1.15+q*(s?.30:.08));};
  const body=icon.parts.find(p=>p.name==='message-body').d;
  icon.parts.find(p=>p.name==='message-solid').d=(t,s)=>holes(body,...[0,1,2].map(i=>dot(i,t,s)));
  for(let i=0;i<3;i++)icon.parts.find(p=>p.name==='message-dot-'+i).d=(t,s)=>dot(i,t,s);
  revise(icon,{light:.36,strong:.50,intent:'A bouncing typing cadence passes through three dots, one reply at a time, then all three return to rest.'});
 }
 if(n==='phone'){
  rePose(icon,()=>true,rigid((t,s)=>({angle:motion(t,s,[[0,0],[.16,-5,'out'],[.28,-5],[.47,0,'in']],[[0,0],[.05,1.5,'launch'],[.19,-13],[.32,-9.5],[.54,.8],[.64,0]]),dy:motion(t,s,[[0,0],[.16,-.5,'out'],[.28,-.5],[.47,0,'in']],[[0,0],[.05,.15,'launch'],[.19,-1.3],[.32,-.95],[.54,.08],[.64,0]]),pivot:s?[16,16]:[16,24]})));
  revise(icon,{light:.51,strong:.66,intent:'The receiver dips, springs up into a listening pose about the handset centre with a damped settle, and returns in one continuous placement.'});
 }
 if(n==='microphone'){
  const pose=rigid((t,s)=>({angle:motion(t,s,[[0,0],[.18,-4,'out'],[.29,-4],[.48,0]],[[0,0],[.06,1.5,'launch'],[.21,-10.5],[.30,-7],[.52,.8],[.62,0]]),pivot:[16,23]}));
  rePose(icon,p=>has(p,'capsule','cradle'),pose);revise(icon,{light:.51,strong:.66,intent:'Capsule and cradle tilt together at the stand joint, hold toward the speaker, then return as one assembly.'});
 }
 if(n==='volume'){
  for(let i=0;i<2;i++)for(const p of icon.parts.filter(p=>has(p,'wave-'+i))){p.pose=rigid((t,s)=>({dx:s?flow([[0,0],[.04,-.2,'launch'],[.15,1.6],[.25,1.1],[.39,-.08],[.44,0]],t-i*.085):0}));p.visibility=(t,s)=>motion(t-i*(s?.085:.06),s,[[0,.7],[.03,.7],[.12,1],[.25,.7]],[[0,.7],[.04,.7],[.13,1],[.20,.88],[.25,1],[.44,.7]]);}
  revise(icon,{light:.34,strong:.54,intent:'Sound emphasis travels from the near arc to the far arc; in Strong each arc is pushed outward and springs back while the speaker stays fixed.'});
 }
 if(n==='wifi'){
  icon.parts=icon.parts.filter(p=>p.style!=='tint');
  for(let i=0;i<3;i++)for(const p of icon.parts.filter(p=>has(p,'band-'+i))){p.d=(t,s)=>arc(16,25,6+i*5+(s?flow([[0,0],[.04,-.9,'launch'],[.12,.4],[.18,-.12],[.23,0]],t-i*.07):0),-135,-45);p.visibility=(t,s)=>motion(t-i*(s?.07:.055),s,[[0,.15],[.02,.15],[.13,1,'out']],[[0,.15],[.02,.15],[.09,1,'hit']]);}
  revise(icon,{light:.28,strong:.40,intent:'Signal acquires from the source outward; in Strong each band gathers in, snaps to full strength and settles into its retained connected state.'});resultAction(icon,p=>has(p,'band'));
 }
 if(n==='cloud'){
  rePose(icon,()=>true,(x,y,t,s)=>{const end=s?.72:.48,env=s?flow([[0,0],[.18,1],[.44,1],[.72,0]],t):track([[0,0],[end*.28,1],[end*.65,1],[end,0]],t),travel=s?flow([[0,0],[.05,-.2,'launch'],[.23,1.6],[.36,1.1],[.72,0]],t):env*.35,w=(x-4.8)/22.7,shape=env*Math.sin(t*(s?13:10)-w*3)*(s?.4:.10);return[x+travel,y+shape];});
  revise(icon,{light:.52,strong:.72,intent:'A gust leans the cloud back, drives it forward with a damped settle and a travelling contour ripple, then lets it drift home; its area stays nearly constant.'});
 }
 if(n==='sun'){
  rePose(icon,p=>has(p,'ray'),(x,y,t,s)=>{const a=motion(t,s,[[0,0],[.35,45,'out']],[[0,0],[.05,-3,'launch'],[.28,50],[.37,44],[.47,45]]),delta=motion(t,s,[[0,0],[.14,.25],[.35,0]],[[0,0],[.06,-.15,'launch'],[.20,1.1],[.31,.6],[.47,0]]);[x,y]=rotate(x,y,a);const d=Math.hypot(x-16,y-16);return[x+(x-16)/d*delta,y+(y-16)/d*delta];});
  revise(icon,{light:.39,strong:.51,intent:'The rays open outward as a clockwise phase passes around a still disc, ending at the next symmetric ray position.',cycleDegrees:45});
 }
 if(n==='moon'){
  rePose(icon,()=>true,rigid((t,s)=>({angle:motion(t,s,[[0,0],[.22,3],[.51,0]],[[0,0],[.05,-1.5,'launch'],[.24,8],[.40,5.2],[.57,-.8],[.68,0]]),dx:motion(t,s,[[0,0],[.22,.22],[.51,0]],[[0,0],[.05,-.12,'launch'],[.24,.75],[.40,.5],[.57,-.05],[.68,0]]),dy:motion(t,s,[[0,0],[.22,-.4],[.51,0]],[[0,0],[.05,.15,'launch'],[.24,-1.2],[.40,-.8],[.57,.08],[.68,0]])})));
  revise(icon,{light:.55,strong:.72,intent:'Light follows one quiet shallow arc; Strong dips, rises through a springy arc with a damped settle, then returns with a long soft descent.'});
 }
 if(n==='location'){
  const pose=rigid((t,s)=>({dy:motion(t,s,[[0,0],[.10,-1.25,'out'],[.25,0,'in']],[[0,0],[.05,.3],[.16,-2.9],[.34,0,'hit']]),angle:motion(t,s,[[0,0],[.25,0],[.31,1.1],[.40,0]],[[0,0],[.34,0],[.40,3],[.46,-1.6],[.51,.7],[.56,0]]),pivot:[16,27]}));
  rePose(icon,()=>true,pose);revise(icon,{light:.44,strong:.60,intent:'The rigid map pin lifts and lands on a fixed point, followed by one small contact response; its circular counter never squashes.'});
 }
 if(n==='compass'){
  rePose(icon,p=>has(p,'needle'),rigid((t,s)=>({angle:motion(t,s,[[0,0],[.07,-12],[.23,2,'out'],[.35,0]],[[0,0],[.04,5],[.12,-48],[.24,9],[.31,-4],[.37,1.5],[.43,0]])})));
  revise(icon,{light:.39,strong:.48,intent:'The needle searches briefly, crosses the correct bearing once, then damps into a clear stable direction.'});
 }
 if(n==='link'){
  for(let i=0;i<2;i++)rePose(icon,p=>has(p,'loop-'+i),rigid((t,s)=>{const d=motion(t-(i?(s?.07:.035):0),s,[[0,0],[.16,.5,'out'],[.23,.5],[.40,0]],[[0,0],[.04,-.15,'launch'],[.18,1.25],[.30,.9],[.50,-.08],[.58,0]])*(i?-1:1);return{dx:-d,dy:d};}));
  revise(icon,{light:.47,strong:.66,intent:'The leading loop takes up slack; the second loop follows after tension reaches the join, and both remain interlocked.'});
 }
 if(n==='shield'){
  const brace=(t,s)=>motion(t,s,[[0,0],[.08,-2.5,'in'],[.23,-.8,'out'],[.35,0]],[[0,0],[.05,1.5],[.14,-7,'hit'],[.23,1.8],[.30,-.6],[.36,0]]);
  rePose(icon,()=>true,rigid((t,s)=>({angle:brace(t,s),pivot:[16,27]})));
  const hit=rotate(25,14,-7,16,27),at=t=>{const q=flow([[0,0],[.05,0],[.14,1,'hit'],[.27,2]],t);return q<=1?[mix(27.7,hit[0],q),mix(10.3,hit[1],q)]:[mix(hit[0],27,q-1),mix(hit[1],7.7,q-1)];};
  const signal=(t,s)=>capsule(at(t-.035),at(t),1.15),visible=(t,s)=>s?flow([[0,0],[.045,0],[.07,1],[.21,1],[.28,0]],t):0;
  icon.parts.push(fill('shield-deflected-signal',signal,'outline',{visibility:visible}),fill('shield-deflected-signal-solid',signal,'solid',{visibility:visible}));
  revise(icon,{light:.39,strong:.46,intent:'The shield leans in, takes the hit, and rings down through two damped corrections at its lower point. Strong redirects one incoming signal at the edge while the attached check stays stable.'});
 }
 if(n==='cart'){
  const dx=(t,s)=>motion(t,s,[[0,0],[.20,1.2,'out'],[.30,1.2],[.50,0]],[[0,0],[.05,-.4,'launch'],[.24,2.9],[.36,2.3],[.68,0]]),pitch=(t,s)=>motion(t,s,[[0,0],[.18,0],[.25,1.4],[.34,0]],[[0,0],[.22,0],[.28,3.8,'hit'],[.34,-1.2],[.39,.4],[.44,0]]);
  rePose(icon,p=>has(p,'basket','handle'),rigid((t,s)=>({dx:dx(t,s),angle:pitch(t,s),pivot:[16,25.5]})));
  for(let i=0;i<2;i++)rePose(icon,p=>has(p,'wheel-'+i),(x,y,t,s)=>{const d=dx(t,s),p=rotate(x,y,d/1.9*180/Math.PI,i?21.5:12,25.5);return[p[0]+d,p[1]];});
  revise(icon,{light:.54,strong:.70,intent:'The cart rocks back, rolls forward with a damped settle, brakes so the basket pitches and rings down, then rolls back to rest.'});
 }
}}
