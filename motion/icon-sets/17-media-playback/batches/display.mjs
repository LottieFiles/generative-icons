import{toggle,tube,stroke,fill,framed,rect,polygon,rigid,posed,curve,arc,circle,holes,arcTube}from'./helpers.mjs';
const caller=(t,s,r)=>r?curve(t,s,[[0,1],[.04,1],[.36,0]],[[0,1],[.04,1],[.30,0]]):curve(t,s,[[0,0],[.06,0],[.46,1,'out']],[[0,0],[.30,0],[.50,1]]);
const nod=rigid((t,s,r)=>({angle:r||!s?0:curve(t,true,[],[[0,0],[.04,0],[.14,3],[.32,-9,'pass'],[.46,1.5,'hit'],[.68,0]]),pivot:[12,16]}));
const video=toggle('video-call','Video call','The camera connects and reveals a caller. Strong gives the whole camera a clear nod before the caller settles into view.',posed([
 fill('wedge-solid',polygon([[18.5,13],[26,10.5],[26,21.5],[18.5,19]],.8)),
 fill('body-interior',rect(4.5,9.5,15,13,2.5),'tint'),stroke('body',rect(4.5,9.5,15,13,2.5)),stroke('wedge','M19.5 13 L26 10.5 L26 21.5 L19.5 19'),
 fill('caller-head',circle(12,14.5,1.6),'outline',{visibility:caller}),stroke('caller-shoulders',arc(12,22.5,3,180,360),{visibility:caller}),
 fill('body-solid',(t,s,r)=>{const a=caller(t,s,r);return holes(rect(4.5,9.5,15,13,2.5),circle(12,14.5,1.6*a),arc(12,21.2,2.8*a,180,360)+` L${12-2.8*a} 21.2 Z`);})
],nod),.46,.68,'Idle','Connected',.36,.30);video.duotoneFill=true;
const captionLength=(t,s,r)=>r?curve(t,s,[[0,14],[.04,14],[.36,.4,'io'],[.40,0,'linear']],[[0,14],[.04,14],[.26,.4],[.30,0]]):curve(t,s,[[0,0],[.06,0],[.50,14,'out']],[[0,0],[.04,0],[.26,15.6,'pass'],[.40,13.4,'hit'],[.56,14]]);
const cc=toggle('closed-captions','Closed captions','A caption line opens beneath the letters. Strong writes the line faster, reaches farther and settles to the same enabled state.',[
 fill('plate-interior',rect(4.5,7.5,23,17,3),'tint'),stroke('plate',rect(4.5,7.5,23,17,3)),
 ...[11.25,20.75].map((x,i)=>stroke('letter-'+i,arc(x,14,2.75,45,315))),
 stroke('caption',(t,s,r)=>{const w=captionLength(t,s,r);return`M${16-w/2} 20.5 L${16+w/2} 20.5`;},{visibility:(t,s,r)=>captionLength(t,s,r)<.4?0:1}),
 fill('plate-solid',(t,s,r)=>{const w=Math.max(0,captionLength(t,s,r)),h=Math.min(2.6,w);return holes(rect(4.5,7.5,23,17,3),arcTube(11.25,14,2.75,45,315,2.6),arcTube(20.75,14,2.75,45,315,2.6),rect(16-w/2,20.5-h/2,w,h,h/2));})
],.50,.56,'Off','On',.40,.30);cc.duotoneFill=true;
const docking=(t,s,r)=>r?curve(t,s,[[0,1],[.04,1],[.46,0]],[[0,1],[.04,1],[.20,1.06],[.46,0]]):curve(t,s,[[0,0],[.06,0],[.50,1,'out']],[[0,0],[.04,0],[.16,-.12],[.36,1.06,'pass'],[.50,.97,'hit'],[.66,1]]);
const pip=toggle('picture-in-picture','Picture-in-picture','The video window moves into its dock. Strong gathers briefly, travels faster and settles at the same corner position.',[
 ...framed('frame',rect(4.5,7,23,18,2.5),holes(rect(3.5,6,25,20,3.5),rect(5.5,8,21,16,1.5))),
 stroke('window',rect(15.5,14,8,6,1.5),{pose:rigid((t,s,r)=>({dx:-6*(1-docking(t,s,r)),dy:-3*(1-docking(t,s,r))}))}),
 fill('window-solid',rect(14.5,13,10,8,2.5),'solid',{pose:rigid((t,s,r)=>({dx:-6*(1-docking(t,s,r)),dy:-3*(1-docking(t,s,r))}))})
],.50,.66,'Expanded','Docked',.46,.46);pip.duotoneFill=true;
const slashProgress=(t,s,r)=>r?curve(t,s,[[0,1],[.04,1],[.36,.03,'in'],[.40,0,'linear']],[[0,1],[.04,1],[.26,.03],[.30,0]]):curve(t,s,[[0,0],[.05,0],[.45,1,'out']],[[0,0],[.04,0],[.20,1.05,'pass'],[.30,.98,'hit'],[.42,1]]);
const camera=toggle('camera-off','Camera off','A diagonal stroke switches the camera off. Strong draws the stroke in one fast move with a small extension and a clean settle.',[
 fill('wedge-solid',polygon([[19,13.5],[26.5,10.5],[26.5,21.5],[19,18.5]],.8)),fill('body-interior',rect(5,9.5,15,13,2.5),'tint'),stroke('body',rect(5,9.5,15,13,2.5)),stroke('wedge','M20 13.5 L26.5 10.5 L26.5 21.5 L20 18.5'),
 fill('body-solid',(t,s,r)=>{const end=7+18*slashProgress(t,s,r),a=12.2,b=Math.min(17.3,Math.max(a,end+2.3)),h=1.63;return holes(rect(5,9.5,15,13,2.5),polygon([[a+h,a-h],[b+h,b-h],[b-h,b+h],[a-h,a+h]],.6));}),
 ...tube('slash',(t,s,r)=>{const p=7+18*slashProgress(t,s,r);return`M7 7 L${p} ${p}`;},{visibility:(t,s,r)=>slashProgress(t,s,r)<.03?0:1})
],.45,.42,'On','Off',.40,.30);camera.duotoneFill=true;
export const display={'video-call':video,'closed-captions':cc,'picture-in-picture':pip,'camera-off':camera};
