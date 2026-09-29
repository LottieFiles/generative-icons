import {rect,hole,stroke,fill,tube,line,framed,rigid,rotate,polygon} from './drawing.mjs';
import {track,flow,drive as motion,revise,resultAction,has,rePose,timePair} from './direction.mjs';
export function directActions(icons){for(const icon of icons){const n=icon.name;
 // Open trays, arrows, and menu bars have no enclosed surface to tint.
 if(['download','upload','refresh','menu'].includes(n))icon.parts=icon.parts.filter(p=>p.style!=='tint');
 if(n==='download'){
  const travel=(t,s)=>motion(t,s,[[0,0],[.18,2.0,'in'],[.28,2.0],[.45,0,'out']],[[0,0],[.045,-.6,'launch'],[.22,4.6,'hit'],[.33,3.4],[.53,-.25],[.62,0]]);
  rePose(icon,p=>!has(p,'tray'),rigid((t,s)=>({dy:travel(t,s)})));
  rePose(icon,p=>has(p,'tray'),rigid((t,s)=>({dy:motion(t,s,[[0,0],[.18,0],[.23,.18],[.34,0]],[[0,0],[.22,0],[.28,.55],[.35,-.15],[.42,0]])})));
  revise(icon,{light:.49,strong:.66,intent:'The arrow lifts briefly, drives into the tray and rebounds; the tray dips on contact and the arrow springs home.'});
 }
 if(n==='upload'){
  rePose(icon,p=>!has(p,'tray'),rigid((t,s)=>({dy:motion(t,s,[[0,0],[.07,.4],[.25,-2.2,'in']],[[0,0],[.065,.9,'launch'],[.23,-3],[.32,-2.05],[.40,-2.2]])})));
  rePose(icon,p=>has(p,'tray'),rigid((t,s)=>({dy:s?flow([[0,0],[.09,0],[.14,.45],[.21,-.12],[.28,0]],t):0})));
  revise(icon,{light:.29,strong:.40,intent:'A deeper preload launches the arrow upward; the tray recoils under it and settles while the arrow stays advanced until preview reset.'});resultAction(icon,p=>!has(p,'tray'));
 }
 if(n==='copy'){
  const pose=rigid((t,s)=>({dx:motion(t,s,[[0,0],[.18,1.3,'out']],[[0,0],[.055,-.3,'launch'],[.23,2.4],[.35,1.15],[.44,1.3]]),dy:motion(t,s,[[0,0],[.18,1,'out']],[[0,0],[.055,-.15,'launch'],[.23,1.8],[.35,.9],[.44,1]]),angle:motion(t,s,[[0,0],[.08,1.5],[.18,0]],[[0,0],[.14,6],[.30,-1.5],[.38,.6],[.44,0]]),pivot:[11,11]}));
  rePose(icon,p=>has(p,'sheet'),pose);revise(icon,{light:.22,strong:.46,intent:'The sheet tucks toward its source, then peels away with a lifting corner that springs flat as the page holds separated.'});resultAction(icon,p=>has(p,'sheet'));
 }
 if(n==='trash'){
  rePose(icon,p=>has(p,'lid','handle'),rigid((t,s)=>({angle:motion(t,s,[[0,0],[.14,9,'out'],[.23,9],[.39,0,'in']],[[0,0],[.05,-1,'launch'],[.18,22],[.50,0,'hit'],[.55,2.2],[.62,0]]),pivot:[25,11.5]})));
  rePose(icon,p=>has(p,'body','slot'),rigid((t,s)=>({dy:motion(t,s,[[0,0],[.39,0],[.43,.15],[.50,0]],[[0,0],[.50,0],[.55,.45],[.61,-.1],[.68,0]])})));
  // The complete rim meets the closed lid's painted underside.
  const body='M9 12.8 L10 24 C10.17 25.42 11.38 26.5 12.8 26.5 L19.2 26.5 C20.62 26.5 21.83 25.42 22 24 L23 12.8 Z';
  for(const p of icon.parts.filter(p=>has(p,'body')))p.d=p.name==='trash-solid-body'?hole(body,rect(14.8,18,2.4,5.2,1.2)):body;
  revise(icon,{light:.53,strong:.70,intent:'The rigid lid snaps open past its rest, corrects, drops shut with a chatter, and the complete bin rim dips on contact.'});
 }
 if(n==='edit'){
  const end=3.6,strongEnd=3.6,write=(t,s)=>motion(t,s,[[0,0],[.10,0],[.29,end,'linear']],[[0,0],[.13,0],[.23,strongEnd]]),lift=(t,s)=>motion(t,s,[[0,0],[.09,3.8,'in'],[.29,3.8],[.37,0,'out']],[[0,0],[.045,-.6,'launch'],[.12,3.8],[.23,3.8],[.32,-.5],[.40,0]]);
  icon.parts=icon.parts.filter(p=>!has(p,'writing-rule'));
  rePose(icon,()=>true,rigid((t,s)=>({dx:write(t,s),dy:lift(t,s)})));
  const ink=(t,s,r)=>{const tt=r?(1-track([[0,0],[.23,1]],t))*.60:t;return`M8 25.8 L${8+Math.max(.025,write(tt,s))} 25.8`;};
  icon.parts.unshift(...tube('edit-written-ink',ink,{visibility:(t,s,r)=>Math.min(1,write(r?(1-track([[0,0],[.23,1]],t))*.60:t,s)*20)}));
  revise(icon,{light:.41,strong:.42,intent:'The pencil lifts, lands, drives the ink stroke quickly, then lifts with follow-through; the written mark stays.'});resultAction(icon,p=>!has(p,'written-ink'));
 }
 if(n==='send'){
  rePose(icon,()=>true,rigid((t,s)=>{const d=motion(t,s,[[0,0],[.055,-.25],[.25,2.4,'in']],[[0,0],[.065,-.9,'launch'],[.175,2.4]]);return{dx:d,dy:-d};}));
  revise(icon,{light:.29,strong:.22,intent:'The plane draws back further, launches along its nose direction with real acceleration and stays advanced; a quiet separate reset rearms it.'});resultAction(icon,()=>true);
 }
 if(n==='bookmark'){
  const seat=(t,s)=>motion(t,s,[[0,0],[.18,1.2,'out']],[[0,0],[.06,-.7,'launch'],[.22,1.6],[.32,1.1],[.43,1.2]]);
  rePose(icon,()=>true,(x,y,t,s)=>{const w=Math.max(0,(y-12)/13),flex=motion(t,s,[[0,0],[.14,.35],[.27,0]],[[0,0],[.07,-.4],[.22,1.4],[.32,-.5],[.40,.2],[.46,0]]);return[x+w*w*flex,y+seat(t,s)];});
  revise(icon,{light:.31,strong:.48,intent:'The bookmark lifts, seats down with contact, and its paper tail whips then damps to rest; the saved position remains.'});resultAction(icon,()=>true);
 }
 if(n==='eye'){
  // Existing circular pupil clipping and matching Solid aperture are retained.
  for(const p of icon.parts.filter(p=>has(p,'pupil')))p.pose=rigid((t,s)=>({dx:s?flow([[0,0],[.06,1.3],[.11,.9],[.14,1.05],[.175,0]],t):0}));
  revise(icon,{light:.27,strong:.44,intent:'Light makes one full fast blink. Strong glances wide, the lids snap shut and reopen with a damped flutter; the pupil remains clipped inside the aperture.'});
 }
 if(n==='menu'){
  revise(icon,{light:.24,strong:.40,intent:'The outer bars pull apart, then converge and rotate past the X before settling; the middle bar clears early and the reverse springs the three-bar menu back.'});icon.reverseTimings=timePair(.24,.40);
 }
}}
