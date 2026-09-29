import svgpath from 'svgpath';
import {circle,rect,polygon,hole,holes,stroke,fill,tube,line,framed,rigid,rotate,mix,arc,annulus} from './drawing.mjs';
import {track,flow,drive as motion,timePair,revise,resultAction,has,rePose,clamp} from './direction.mjs';
export function directTools(icons){for(const icon of icons){const n=icon.name;
 if(n==='gift'){
  const lift=(t,s)=>motion(t,s,[[0,0],[.05,-.12],[.19,1.4,'out'],[.28,1.4],[.46,0,'in'],[.52,0]],[[0,0],[.05,-.2,'launch'],[.20,3.5],[.33,2.6],[.58,0,'hit'],[.63,.22],[.69,0]]);
  const lid=rigid((t,s)=>({dy:-lift(t,s)})),bow=(side)=>rigid((t,s)=>({dy:-lift(t,s),angle:side*(lift(t-.035,s)-lift(t,s))*4,pivot:[16,10.5]}));
  const body=rect(7,15.5,18,11.5,1.8),cover=rect(5.5,10.5,21,5,1.3);
  revise(icon,{light:.54,strong:.78,posesLight:[0,.05,.19,.28,.46,.54],posesStrong:[0,.06,.21,.30,.37,.43,.64,.78],intent:'The rigid lid opens above a complete box rim; the attached bow follows and settles after contact.',parts:[fill('gift-field',body,'tint'),stroke('gift-box',body),stroke('gift-ribbon','M16 15.5 L16 27'),fill('gift-solid-box',hole(body,rect(14.7,16.8,2.6,8.9,1))),...framed('gift-lid',cover,cover,{pose:lid}),...tube('gift-bow-left','M16 10.5 C14 7 11 5.7 9.6 7 C8.2 8.3 10.6 10.5 16 10.5',{pose:bow(-1)}),...tube('gift-bow-right','M16 10.5 C18 7 21 5.7 22.4 7 C23.8 8.3 21.4 10.5 16 10.5',{pose:bow(1)})]});
 }
 if(n==='printer'){
  const feed=(t,s)=>motion(t,s,[[0,0],[.045,0],[.34,2.6,'out']],[[0,0],[.045,.06,'launch'],[.10,1.4,'pass'],[.22,2.6]]);
  // A 10-unit page translates behind the 18-unit slot plane. Only the visible
  // section is authored; its printed mark shares the entire physical translation.
  const sheet=(t,s)=>rect(11,18,10,5.4+feed(t,s),.9),input=(t,s)=>`M10 12.5 L10 ${7+feed(t,s)} C10 ${6.2+feed(t,s)} 10.7 ${5.5+feed(t,s)} 11.5 ${5.5+feed(t,s)} L20.5 ${5.5+feed(t,s)} C21.3 ${5.5+feed(t,s)} 22 ${6.2+feed(t,s)} 22 ${7+feed(t,s)} L22 12.5`;
  const fixed=icon.parts.filter(p=>has(p,'field','housing'));
  const parts=[...fixed,...tube('printer-input',input),stroke('printer-sheet',sheet),stroke('printer-line',(t,s)=>`M14 ${20.7+feed(t,s)} L18 ${20.7+feed(t,s)}`),fill('printer-solid-sheet',(t,s)=>hole(sheet(t,s),rect(13.5,19.85+feed(t,s),5,1.7,.8)))];
  revise(icon,{parts,light:.38,strong:.29,posesLight:[0,.05,.13,.24,.34,.38],posesStrong:[0,.045,.08,.13,.18,.22,.29],intent:'The rollers feed a fixed-size page; its printed line travels with the paper and the fed result holds.'});resultAction(icon,p=>has(p,'input','sheet','line'));
 }
 if(n==='book'){
  const turn=(t,s)=>motion(t,s,[[0,0],[.05,.03],[.33,.90,'smooth'],[.46,1,'out']],[[0,0],[.06,.05],[.12,.34,'pass'],[.22,.92,'pass'],[.31,1],[.38,1]]),vis=(t,s)=>Math.min(1,turn(t,s)*16,(1-turn(t-(s?.06:.025),s))*20);
  const left='M16 8 C12 5.8 8.3 5.5 5.5 6 L5.5 24 C9 23.5 12.5 24 16 26',right='M16 8 C20 5.8 23.7 5.5 26.5 6 L26.5 24 C23 23.5 19.5 24 16 26';
  const leaf=(t,s)=>{const q=turn(t,s),qb=turn(t-(s?.06:.025),s),c=Math.cos(Math.PI*q),cb=Math.cos(Math.PI*qb),b=Math.sin(Math.PI*q)*(s?3.0:.85),bb=Math.sin(Math.PI*qb)*(s?3.0:.85),x=16+10.5*c,xb=16+10.5*cb;return `M16 8 C${16+4*c} ${5.8-b} ${16+8*c} ${5.5-b} ${x} ${6-b*.4} L${xb} ${24+bb*.25} C${16+7*cb} ${23.5+bb} ${16+3.5*cb} ${24+bb} 16 26 Z`;};
  revise(icon,{light:.52,strong:.42,posesLight:[0,.07,.16,.25,.36,.52],posesStrong:[0,.06,.12,.22,.31,.38,.42],intent:'One complete leaf bows across a fixed spine, exposes the next page, and settles on the left.',parts:[fill('book-left-field',left+' Z','tint'),fill('book-right-field',right+' Z','tint'),stroke('book-left-page',left),stroke('book-right-page',right),fill('book-left-solid',left.replace('M16','M14.8').replace(' 16 26',' 14.8 26')+' Z'),fill('book-right-solid',right.replace('M16','M17.2').replace(' 16 26',' 17.2 26')+' Z'),...line('book-spine',[16,8],[16,26]),stroke('book-turning-leaf',leaf,{visibility:vis}),fill('book-turning-leaf-solid',leaf,'solid',{visibility:vis})]});
 }
 if(n==='bluetooth'){
  const rune='M10 10 L22 21 L16 26 L16 6 L22 11 L10 22';
  const upper=(t,s)=>motion(t,s,[[0,1],[.05,.5],[.13,1],[.31,1]],[[0,1],[.05,.25],[.13,1],[.19,.65],[.27,1],[.46,1]]),lower=(t,s)=>motion(t,s,[[0,1],[.10,.5],[.21,1],[.31,1]],[[0,1],[.11,.25],[.19,1],[.25,.65],[.33,1],[.46,1]]);
  revise(icon,{light:.34,strong:.46,posesLight:[0,.05,.1,.15,.22,.34],posesStrong:[0,.05,.13,.19,.25,.33,.46],intent:'The Bluetooth rune keeps its geometry while an upper-path signal leads the lower response.',parts:[fill('bluetooth-upper-field','M16 6 L22 11 L16 16 Z','tint'),fill('bluetooth-lower-field','M16 16 L22 21 L16 26 Z','tint'),...tube('bluetooth-upper','M10 22 L22 11 L16 6 L16 16',{visibility:upper}),...tube('bluetooth-lower','M10 10 L22 21 L16 26 L16 16',{visibility:lower})]});
 }
 if(n==='user'){
  const head=rigid((t,s)=>({dy:motion(t,s,[[0,0],[.09,.9,'in'],[.14,.9],[.28,0,'out']],[[0,0],[.045,-.4,'launch'],[.15,2],[.25,1.3],[.40,-.14],[.50,0]])}));
  const shoulder=rigid((t,s)=>({dy:motion(t,s,[[0,0],[.13,0],[.20,.18],[.33,0]],[[0,0],[.12,0],[.22,.65],[.35,-.10],[.48,0]])}));
  rePose(icon,p=>has(p,'head'),head);rePose(icon,p=>has(p,'shoulder'),shoulder);
  revise(icon,{light:.35,strong:.56,intent:'A vertical nod leads a small shoulder response; the round head keeps its shape.'});
 }
 if(n==='users'){
  const approach=(t,s)=>motion(t,s,[[0,0],[.18,-1.2,'out'],[.29,-1.2],[.48,0]],[[0,0],[.055,.3,'launch'],[.21,-2],[.35,-1.4],[.56,.12],[.64,0]]);
  rePose(icon,p=>has(p,'rear'),rigid((t,s)=>({dx:approach(t,s),dy:s?.95*approach(t,s):0})));
  rePose(icon,p=>has(p,'front-head'),rigid((t,s)=>({dy:motion(t,s,[[0,0],[.17,0],[.25,.45],[.37,0]],[[0,0],[.20,0],[.30,1.2],[.40,-.12],[.52,0]])})));
  revise(icon,{light:.50,strong:.69,intent:'The second person steps into the group; the foreground person acknowledges the arrival with a delayed nod.'});
 }
 if(n==='key'){
  const ins=(t,s)=>motion(t,s,[[0,0],[.14,1.0,'out'],[.31,1],[.46,0]],[[0,0],[.05,-.45,'launch'],[.19,1.95],[.29,1.7],[.43,1.7],[.64,-.15],[.74,0]]);
  const p=rigid((t,s)=>({dx:ins(t,s),dy:ins(t,s),angle:motion(t,s,[[0,0],[.14,0],[.23,-5],[.32,0]],[[0,0],[.20,0],[.33,-13],[.43,-9.5],[.56,.7],[.64,0]]),pivot:[23,24.5]}));
  rePose(icon,()=>true,p);revise(icon,{light:.49,strong:.78,intent:'The key inserts along its shaft, turns briefly at the inserted tip, then withdraws as one rigid object.'});
 }
 if(n==='scan'){
  icon.parts=icon.parts.filter(p=>p.style!=='tint');
  const travel=(t,s)=>motion(t,s,[[0,0],[.04,0],[.10,1,'in'],[.32,9,'linear'],[.38,10,'out']],[[0,0],[.08,0],[.12,.8,'pass'],[.28,9.2,'pass'],[.34,10]]);
  rePose(icon,p=>has(p,'bar'),rigid((t,s)=>({dy:travel(t,s)-2})));
  rePose(icon,p=>has(p,'corner'),(x,y,t,s,r)=>{if(r)return[x,y];const k=s?flow([[0,1],[.06,.93],[.12,1.02],[.17,1]],t):1;return[16+(x-16)*k,16+(y-16)*k];});
  revise(icon,{light:.41,strong:.40,intent:'The brackets focus first, then one scan crosses at steady speed and stops at the lower edge.'});resultAction(icon,p=>has(p,'bar'));
 }
 if(n==='battery'){
  const charge=(t,s)=>motion(t,s,[[0,4],[.06,4],[.32,12,'out']],[[0,4],[.035,4],[.085,5.2,'pass'],[.19,12]]);
  for(const p of icon.parts.filter(p=>has(p,'charge')))p.d=(t,s)=>rect(8,13,charge(t,s),6,1);
  revise(icon,{light:.36,strong:.23,intent:'Charge reaches a retained level in one continuous fill; Strong charges with a faster decisive stroke.'});resultAction(icon,p=>has(p,'charge'));
 }
 if(n==='plug'){
  const travel=(t,s)=>motion(t,s,[[0,0],[.05,.25],[.19,-1.35,'out'],[.32,-1.35],[.48,0]],[[0,0],[.06,.55,'launch'],[.20,-2.8,'hit'],[.32,-2.15],[.54,.18],[.66,0]]);
  rePose(icon,p=>!has(p,'cable'),rigid((t,s)=>({dy:travel(t,s)})));
  for(const p of icon.parts.filter(p=>has(p,'cable')))p.d=(t,s)=>`M16 ${22+travel(t,s)} C16 ${24+travel(t,s)} ${16+travel(t-.035,s)*.35} 25.5 16 27`;
  revise(icon,{light:.51,strong:.72,intent:'The plug seats upward along its prongs; a connected cable bends after the pull and relaxes on release.'});
 }
 if(n==='monitor'){
  const body=rect(5,6.5,22,15,2.4),screen=rect(7.7,9.2,16.6,9.6,1);
  const wake=(t,s)=>motion(t,s,[[0,0],[.06,0],[.24,1,'out']],[[0,0],[.05,0],[.12,.85],[.17,.55],[.30,1]]);
  const panel=(i)=>(t,s)=>{const p=s?flow([[0,0],[.14,0],[.22,.45,'pass'],[.38,1]],t):1;return rect(i?mix(22,16.3,p):9,11,i?mix(.05,5,p):mix(11.7,4.4,p),5.8,.7);};
  revise(icon,{light:.28,strong:.45,intent:'A stationary display wakes its content, then the Strong layout resolves into two panels.',parts:[...framed('monitor-screen',body,hole(body,screen)),...line('monitor-stand',[16,21.5],[16,26.5]),...line('monitor-foot',[11,26.5],[21,26.5]),...[0,1].flatMap(i=>[fill('monitor-panel-'+i,panel(i),'outline',{visibility:(t,s)=>wake(t-(s?i*.10:0),s)}),fill('monitor-panel-solid-'+i,panel(i),'solid',{visibility:(t,s)=>wake(t-(s?i*.10:0),s)})])]});
  // Wake ends at the same complete layout for both profiles.
  icon.timings=timePair(.30,.56);resultAction(icon,p=>has(p,'panel'));
 }
 if(n==='smartphone'){
  const travel=(t,s)=>motion(t,s,[[0,0],[.04,0],[.28,7,'out']],[[0,0],[.05,-.5,'launch'],[.10,2,'pass'],[.17,6.7,'pass'],[.24,7]]);
  const visibleTile=(i,t,s)=>{const top=11+i*7-travel(t,s),a=Math.max(10.6,top),b=Math.min(24.3,top+4.5);return rect(13.3,a,[5.4,4.2,5.0][i],Math.max(.02,b-a),.7);};
  const visibility=(i,t,s)=>{const y=11+i*7-travel(t,s);return y<24.3&&y+4.5>10.6?1:0;};
  const body=rect(10,4.5,12,23,2.6);
  revise(icon,{light:.32,strong:.30,intent:'A clipped content stack scrolls inside a stationary phone; Strong flicks quickly and decelerates into the next item.',parts:[...framed('smartphone-frame',body,holes(body,rect(12.5,10,7,15,.7),rect(14,7.3,4,1.5,.75))),stroke('smartphone-speaker','M14 8 L18 8'),...[0,1,2].flatMap(i=>[fill('smartphone-tile-'+i,(t,s)=>visibleTile(i,t,s),'outline',{visibility:(t,s)=>visibility(i,t,s)}),fill('smartphone-tile-solid-'+i,(t,s)=>visibleTile(i,t,s),'solid',{visibility:(t,s)=>visibility(i,t,s)})])]});resultAction(icon,p=>has(p,'tile'));
 }
 if(n==='laptop'){
  const opening=(t,s)=>motion(t,s,[[0,.24],[.04,.24],[.32,1,'out']],[[0,.24],[.07,.19],[.29,1.05],[.36,.985],[.42,1.01],[.50,1]]);
  const lid=(x,y,t,s)=>[x,23+(y-20.5)*opening(t,s)];
  const body=rect(7,6.5,18,14,1.8);
  revise(icon,{light:.36,strong:.56,intent:'A rigid lid opens from the continuous lower hinge and holds at its working angle; the base stays planted.',parts:[...framed('laptop-lid',body,body,{pose:lid}),...tube('laptop-base','M7 23 L4.5 24.5 C5 25.7 6 26.5 7.5 26.5 L24.5 26.5 C26 26.5 27 25.7 27.5 24.5 L25 23'),...line('laptop-hinge',[7,23],[25,23])]});resultAction(icon,p=>has(p,'lid'));
 }
 if(n==='document'){
  const body=icon.parts.find(p=>p.name==='document-page').d;
  const length=(i,t,s)=>motion(t,s,[[0,.05],[.045+i*.12,.05],[.22+i*.12,i?5.5:8,'out']],[[0,.05],[.04+i*.10,.05],[.065+i*.10,i?2.2:3.2,'pass'],[.13+i*.10,i?5.5:8]]);
  const ink=(i,t,s)=>rect(11,15+i*5.5,length(i,t,s),2,1);
  const remap=(t,s,r)=>r?(1-track([[0,0],[.24,1,'out']],t))*.62:t;
  revise(icon,{light:.38,strong:.30,intent:'Ink writes left to right, with the second line following the first; the completed document holds.',parts:[fill('document-field',body,'tint'),stroke('document-page',body),stroke('document-fold','M18.5 6.5 L18.5 10 C18.5 10.7 18.8 11 19.5 11 L23 11'),fill('document-solid',(t,s,r)=>holes(body,ink(0,remap(t,s,r),s),ink(1,remap(t,s,r),s),polygon([[19.5,7.5],[22,10],[19.5,10]],.3))),...[0,1].map(i=>fill('document-ink-'+i,(t,s,r)=>ink(i,remap(t,s,r),s),'outline'))]});resultAction(icon,()=>false);
 }
 if(n==='clipboard'){
  const hinge=rigid((t,s)=>({angle:motion(t,s,[[0,0],[.10,-5.6],[.18,-5.6],[.32,0,'in']],[[0,0],[.045,1,'launch'],[.15,-13.5],[.26,-10],[.43,0,'hit'],[.48,-1.2],[.54,0]]),pivot:[12,5.5]}));
  for(const p of icon.parts.filter(p=>has(p,'clip')&&!has(p,'clipboard-field','clipboard-board','solid-board','line'))) {p.d=rect(12,5.5,8,4,.7);p.pose=hinge;}
  const seat=(t,s)=>s?flow([[0,0],[.12,0],[.23,-.95],[.32,-.6],[.43,0]],t):0;
  rePose(icon,p=>has(p,'line'),rigid((t,s)=>({dy:seat(t,s)})));
  icon.parts.find(p=>p.name==='clipboard-solid-board').d=(t,s)=>holes(rect(8,8.5,16,18,2),rect(12,15+seat(t,s),8,2,1),rect(12,20.5+seat(t,s),5.5,2,1));
  revise(icon,{light:.36,strong:.58,intent:'A rigid top clip hinges open; Strong lets the paper seat beneath it before the clip closes.'});
 }
 if(n==='credit-card'){
  const pose=rigid((t,s)=>({dy:motion(t,s,[[0,0],[.20,1.6,'out'],[.30,1.6],[.49,0]],[[0,0],[.055,-.6,'launch'],[.20,2.4,'hit'],[.32,1.9],[.56,-.25],[.68,0]]),angle:motion(t,s,[[0,0],[.20,-2],[.30,-2],[.49,0]],[[0,0],[.055,1.2,'launch'],[.20,-5],[.32,-3.6],[.56,.7],[.68,0]])}));
  rePose(icon,()=>true,pose);icon.parts.unshift(...tube('credit-card-reader','M10 27.5 L22 27.5'));
  revise(icon,{light:.52,strong:.72,intent:'The whole card approaches a fixed reader, holds for the tap, and withdraws with its chip and stripe attached.'});
 }
 if(n==='wallet'){
  const q=(t,s)=>motion(t,s,[[0,0],[.08,.12],[.22,.7],[.32,.7],[.52,0,'in']],[[0,0],[.055,.05],[.20,1.05],[.36,.94],[.62,0]]);
  const flap=rect(20,13.5,7,7,1.4),project=(x,y,t,s)=>[27+(x-27)*Math.cos(q(t,s)),y],clasp=(t,s)=>circle(27-3.5*Math.cos(q(t,s)),17,.85),projectedFlap=(t,s)=>svgpath(flap).translate(-27,0).scale(Math.cos(q(t,s)),1).translate(27,0).toString();
  icon.parts=icon.parts.filter(p=>!has(p,'flap','clasp'));
  icon.parts.push(stroke('wallet-hinged-flap',flap,{pose:project}),fill('wallet-clasp',clasp,'outline'),fill('wallet-hinged-flap-solid',(t,s)=>hole(projectedFlap(t,s),clasp(t,s))));
  const rise=(t,s)=>s?flow([[0,0],[.18,0],[.31,2.5],[.41,1.8],[.55,0]],t):0;
  const card=(t,s)=>rect(10,8.5-rise(t,s),10,Math.max(.02,rise(t,s)),.65),visible=(t,s)=>Math.min(1,rise(t,s)*8);
  icon.parts.unshift(stroke('wallet-card-edge',card,{visibility:visible}),fill('wallet-card-edge-solid',card,'solid',{visibility:visible}));
  revise(icon,{light:.55,strong:.74,intent:'The leather flap pivots from its right seam; its round clasp follows the hinge, Strong exposes and reseats one card edge, then the flap closes.'});
 }
 if(n==='flag'){
  rePose(icon,p=>has(p,'cloth'),(x,y,t,s)=>{const w=(x-8)/17,d=s?.82:.54,env=s?flow([[0,0],[.06,1],[.28,1],[.55,.5],[.82,0]],t):track([[0,0],[.08,1],[d-.12,1],[d,0]],t),a=s?2.6:.75,phase=t*(s?16:10);return[x,y+env*a*w*Math.sin(phase-w*3.4)];});
  revise(icon,{light:.57,strong:.85,intent:'A crest and trough travel in one direction toward the free edge; the pole and cloth attachment stay fixed.'});
 }
 if(n==='chart'){
  const target=[10,16,20],start=[4,6,9];icon.parts=[];
  // A narrow outlined rectangle paints two parallel strokes and reads heavier
  // than the family's single-line controls. Use one round-capped centerline.
  for(let i=0;i<3;i++){const h=(t,s)=>motion(t,s,[[0,start[i]],[.035+i*.045,start[i]],[.25+i*.045,target[i],'out']],[[0,start[i]],[.035+i*.045,start[i]],[.075+i*.045,start[i]+(target[i]-start[i])*.4,'pass'],[.135+i*.045,target[i]]]);icon.parts.push(...tube('chart-bar-'+i+'-frame',(t,s)=>`M${8+i*8} 26 L${8+i*8} ${26-h(t,s)}`));}
  revise(icon,{light:.38,strong:.26,intent:'Three data bars rise from one baseline into staggered target values and retain the result without overshooting the data.'});resultAction(icon,()=>true);
 }
 if(n==='filter'){
  const width=(i,t,s)=>motion(t,s,[[0,[20,17,12][i]],[.025+i*.06,[20,17,12][i]],[.16+i*.06,[20,12,5][i],'out']],[[0,[20,17,12][i]],[.03+i*.04,[20,17,12][i]],[.05+i*.04,[20,14.5,8.5][i],'pass'],[.09+i*.04,[20,12,5][i]]]);
  icon.parts=[];
  for(let i=0;i<3;i++)icon.parts.push(...tube('filter-tier-'+i,(t,s)=>`M${16-width(i,t,s)/2} ${8+i*8} L${16+width(i,t,s)/2} ${8+i*8}`));
  revise(icon,{light:.33,strong:.22,intent:'The source remains full width while the middle subset and lower result narrow in order and hold.'});resultAction(icon,p=>!has(p,'tier-0'));
 }
}}
