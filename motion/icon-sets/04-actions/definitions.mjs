import { directActions } from '../library/directed-actions.mjs';
import svgpath from 'svgpath';
import {flow} from '../library/direction.mjs';

export const mix=(a,b,p)=>a+(b-a)*p;
// Each interval is a deliberate eased move or a still hold. Sample once at 60 fps.
export function phrase(keys,t){if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i];return mix(a[1],b[1],(1-Math.cos(Math.PI*(t-a[0])/(b[0]-a[0])))/2);}
export function rotate(x,y,a,cx=16,cy=16){const r=a*Math.PI/180,c=Math.cos(r),s=Math.sin(r);return[cx+(x-cx)*c-(y-cy)*s,cy+(x-cx)*s+(y-cy)*c];}
export const circle=(x,y,r)=>`M${x} ${y-r} C${x+r*.552285} ${y-r} ${x+r} ${y-r*.552285} ${x+r} ${y} C${x+r} ${y+r*.552285} ${x+r*.552285} ${y+r} ${x} ${y+r} C${x-r*.552285} ${y+r} ${x-r} ${y+r*.552285} ${x-r} ${y} C${x-r} ${y-r*.552285} ${x-r*.552285} ${y-r} ${x} ${y-r} Z`;
export const rect=(x,y,w,h,r)=>`M${x+r} ${y} L${x+w-r} ${y} C${x+w-r*.448} ${y} ${x+w} ${y+r*.448} ${x+w} ${y+r} L${x+w} ${y+h-r} C${x+w} ${y+h-r*.448} ${x+w-r*.448} ${y+h} ${x+w-r} ${y+h} L${x+r} ${y+h} C${x+r*.448} ${y+h} ${x} ${y+h-r*.448} ${x} ${y+h-r} L${x} ${y+r} C${x} ${y+r*.448} ${x+r*.448} ${y} ${x+r} ${y} Z`;
export function polygon(vertices,r){const corners=vertices.map((v,i)=>{const before=vertices[(i+vertices.length-1)%vertices.length],after=vertices[(i+1)%vertices.length];const radius=Array.isArray(r)?r[i%r.length]:r;const step=p=>{const d=Math.hypot(p[0]-v[0],p[1]-v[1]),a=Math.min(radius,d*.35);return[v[0]+(p[0]-v[0])*a/d,v[1]+(p[1]-v[1])*a/d];};return {v,a:step(before),b:step(after)};});return corners.map(({v,a,b},i)=>`${i?'L':'M'}${a} C${mix(a[0],v[0],2/3)} ${mix(a[1],v[1],2/3)} ${mix(b[0],v[0],2/3)} ${mix(b[1],v[1],2/3)} ${b}`).join(' ')+' Z';}
export function capsule(a,b,width){const len=Math.hypot(b[0]-a[0],b[1]-a[1]);return svgpath(rect(-width/2,-width/2,len+width,width,width/2)).rotate(Math.atan2(b[1]-a[1],b[0]-a[0])*180/Math.PI).translate(...a).toString();}
// Reverse a closed contour without reflecting its geometry. One final Close
// and a retraced zero-width seam preserve transparent holes in this exporter.
export function reverseContour(d){const segments=svgpath(d).abs().unarc().unshort().segments;let first,last;const edges=[];for(const [cmd,...v]of segments){if(cmd==='M'){first=v;last=v;}else if(cmd==='L'||cmd==='C'){edges.push({cmd,start:last,v});last=v.slice(-2);}else if(cmd==='Z'){if(last[0]!==first[0]||last[1]!==first[1])edges.push({cmd:'L',start:last,v:first});last=first;}else throw Error('Unsupported hole command '+cmd);}return 'M'+first+' '+edges.reverse().map(e=>e.cmd==='L'?'L'+e.start:'C'+[...e.v.slice(2,4),...e.v.slice(0,2),...e.start]).join(' ')+' Z';}
function winding(d){const p=svgpath(d).abs().unarc().unshort().segments.filter(s=>s[0]!=='Z').map(s=>s.slice(-2));return Math.sign(p.reduce((sum,a,i)=>{const b=p[(i+1)%p.length];return sum+a[0]*b[1]-b[0]*a[1];},0))||1;}
export const hole=(outer,inner)=>outer.replace(/Z\s*$/,'')+' L'+svgpath(outer).abs().segments[0].slice(1)+' '+(winding(outer)===winding(inner)?reverseContour(inner):inner).replace(/^M/,'L');
const fill=(name,d,style='solid',extra={})=>({name,d,style,...extra});
const stroke=(name,d,extra={})=>({name,d,style:'outline',type:'stroke',width:2,...extra});
const line=(name,a,b,extra={})=>[stroke(name,`M${a} L${b}`,extra),fill(name+'-solid',capsule(a,b,2.6),'solid',extra)];
const rigid=(fn)=>(x,y,t,s,r)=>{const {dx=0,dy=0,angle=0,pivot=[16,16]}=fn(t,s,r);const p=rotate(x,y,angle,...pivot);return[p[0]+dx,p[1]+dy];};
const gesture=(t,s,light,strong)=>s?flow(strong,t):phrase(light,t);
const tray='M5.5 20.5 L5.5 23.5 C5.5 25.16 6.84 26.5 8.5 26.5 L23.5 26.5 C25.16 26.5 26.5 25.16 26.5 23.5 L26.5 20.5';
const solidTray='M4.3 20.5 C4.3 18.9 6.7 18.9 6.7 20.5 L6.7 23.5 C6.7 24.49 7.51 25.3 8.5 25.3 L23.5 25.3 C24.49 25.3 25.3 24.49 25.3 23.5 L25.3 20.5 C25.3 18.9 27.7 18.9 27.7 20.5 L27.7 23.5 C27.7 25.82 25.82 27.7 23.5 27.7 L8.5 27.7 C6.18 27.7 4.3 25.82 4.3 23.5 Z';
function transferParts(up){const id=up?'upload':'download',tip=up?8.5:16.5,stem=up?19:6.3,head=up?13.3:11.7;
 const pose=rigid((t,s)=>({dy:(up?-1:1)*gesture(t,s,[[0,0],[.16,up?1.2:1.3],[.21,up?1.2:1.3],[.46,0]],[[0,0],[.24,up?3:3.2],[.32,up?3:3.2],[.72,0]])}));
 const solidHead=polygon(up?[[10.8,13.4],[16,7.7],[21.2,13.4],[19.5,15],[16,11.1],[12.5,15]]:[[10.8,11.6],[12.5,10],[16,13.9],[19.5,10],[21.2,11.6],[16,17.3]],.7);
 return[fill(id+'-tray-interior','M5.5 20.5 L26.5 20.5 L26.5 23.5 C26.5 25.16 25.16 26.5 23.5 26.5 L8.5 26.5 C6.84 26.5 5.5 25.16 5.5 23.5 Z','tint'),stroke(id+'-tray',tray),fill(id+'-solid-tray',solidTray),stroke(id+'-shaft',`M16 ${stem} L16 ${tip}`,{pose}),fill(id+'-solid-shaft',capsule([16,stem],[16,tip+(up?1.1:-1.1)],2.6),'solid',{pose}),stroke(id+'-head',`M11.2 ${head} L16 ${tip} L20.8 ${head}`,{pose}),fill(id+'-solid-head',solidHead,'solid',{pose})];}
const copyFront=rect(11,11,13,14.5,2.1),copyBack='M21 6 L8 6 C6.62 6 5.5 7.12 5.5 8.5 L5.5 21';
const copyPose=rigid((t,s)=>{const p=gesture(t,s,[[0,0],[.17,1],[.22,1],[.48,0]],[[0,0],[.24,1],[.34,1],[.76,0]]);return{dx:p*(s?2.2:.8),dy:p*(s?1.5:.7)};});
const copyParts=[stroke('copy-source',copyBack),fill('copy-solid-source','M21 4.8 C22.6 4.8 22.6 7.2 21 7.2 L8 7.2 C7.28 7.2 6.7 7.78 6.7 8.5 L6.7 21 C6.7 22.6 4.3 22.6 4.3 21 L4.3 8.5 C4.3 6.46 5.96 4.8 8 4.8 Z'),fill('copy-sheet-interior',copyFront,'tint',{pose:copyPose}),stroke('copy-sheet',copyFront,{pose:copyPose}),fill('copy-solid-sheet',copyFront,'solid',{pose:copyPose})];
const trashBody='M9 16 L10 24 C10.17 25.42 11.38 26.5 12.8 26.5 L19.2 26.5 C20.62 26.5 21.83 25.42 22 24 L23 16';
const trashClosed=trashBody+' Z';
const trashLidPose=rigid((t,s)=>({angle:gesture(t,s,[[0,0],[.16,5.5],[.22,5.5],[.46,0]],[[0,0],[.23,14],[.34,14],[.60,0]]),pivot:[25,11.5]}));
const trashBodyPose=rigid((t,s)=>({dy:s?phrase([[0,0],[.57,0],[.63,.35],[.76,0]],t):0}));
const trashParts=[fill('trash-body-interior',trashClosed,'tint',{pose:trashBodyPose}),stroke('trash-body',trashBody,{pose:trashBodyPose}),stroke('trash-slot','M16 18.5 L16 22.5',{pose:trashBodyPose}),fill('trash-solid-body',hole(trashClosed,rect(14.8,18,2.4,5.2,1.2)),'solid',{pose:trashBodyPose}),...line('trash-lid',[7.5,11.5],[25,11.5],{pose:trashLidPose}),stroke('trash-handle','M13 11.5 L13 8 C13 7.45 13.45 7 14 7 L18.5 7 C19.05 7 19.5 7.45 19.5 8 L19.5 11.5',{pose:trashLidPose}),fill('trash-solid-handle','M11.8 11.5 L11.8 8 C11.8 6.79 12.79 5.8 14 5.8 L18.5 5.8 C19.71 5.8 20.7 6.79 20.7 8 L20.7 11.5 L18.3 11.5 L18.3 8.2 L14.2 8.2 L14.2 11.5 Z','solid',{pose:trashLidPose})];
const pencil=polygon([[8,22],[9.2,17.5],[18.5,8.2],[22.8,12.5],[13.5,21.8]],[.6,.65,1.1,1.1,.65]);
const pencilPose=rigid((t,s)=>({dx:gesture(t,s,[[0,0],[.11,0],[.25,1.2],[.32,1.2],[.52,0]],[[0,0],[.12,0],[.39,3.4],[.49,3.4],[.80,0]]),dy:gesture(t,s,[[0,0],[.11,3.7],[.25,3.7],[.32,0],[.52,0]],[[0,0],[.12,3.7],[.39,3.7],[.49,0],[.80,0]]),angle:gesture(t,s,[[0,0],[.11,-1],[.25,-1],[.36,0]],[[0,0],[.12,-4],[.39,-4],[.53,0]]),pivot:[8,22]}));
const editParts=[...line('edit-writing-rule',[7,26],[24.5,26]),fill('edit-pencil-interior',pencil,'tint',{pose:pencilPose}),stroke('edit-pencil-body',pencil,{pose:pencilPose}),stroke('edit-cap-seam','M16.7 10 L21 14.3',{pose:pencilPose}),fill('edit-solid-pencil',pencil,'solid',{pose:pencilPose})];
const plane=polygon([[6.5,12],[24.5,6.5],[19,24.5],[14.5,17]],[1.5,1.5,1.5,.8]);
const planeSolid=polygon([[6.5,12],[24.5,6.5],[19,24.5],[14.5,17],[20.3,10.9],[12.7,15.9]],[1.4,1.5,1.4,.7,.35,.5]);
const planePose=rigid((t,s)=>{const p=gesture(t,s,[[0,0],[.04,-.15],[.20,.8],[.25,.8],[.51,0]],[[0,0],[.06,-.4],[.27,2.2],[.34,2.2],[.76,0]]);const recover=gesture(t,s,[[0,0],[.25,0],[.38,.18],[.51,0]],[[0,0],[.34,0],[.55,.6],[.76,0]]);return{dx:p,dy:-p+recover};});
const sendParts=[fill('send-plane-interior',plane,'tint',{pose:planePose}),stroke('send-plane',plane,{pose:planePose}),stroke('send-fold','M14.5 17 L23.5 7.5',{pose:planePose}),fill('send-solid-plane',planeSolid,'solid',{pose:planePose})];
const bookmark=polygon([[9,6],[23,6],[23,25],[16,20.5],[9,25]],[1.8,1.8,.85,.9,.85]);
const bookmarkPose=(x,y,t,s)=>{const p=gesture(t,s,[[0,0],[.16,1],[.21,1],[.48,0]],[[0,0],[.055,-.15],[.24,1],[.32,1],[.74,0]]),w=Math.max(0,(y-10)/15);return[x+(s?1.1:.2)*p*w*w,y-(s?3.5:1.2)*p*w];};
const bookmarkParts=[fill('bookmark-interior',bookmark,'tint',{pose:bookmarkPose}),stroke('bookmark-ribbon',bookmark,{pose:bookmarkPose}),fill('bookmark-solid-ribbon',bookmark,'solid',{pose:bookmarkPose})];
const refreshArc='M8 16 C8 11.58 11.58 8 16 8 C19.1 8 21.65 9.65 23 12';
const refreshHead='M23 7.8 L23 12 L18.8 12';
const refreshSolid='M6.8 16 C6.8 10.92 10.92 6.8 16 6.8 C18.3 6.8 20.3 7.6 21.8 8.9 L21.8 7.8 C21.8 6.2 24.2 6.2 24.2 7.8 L24.2 12 C24.2 12.66 23.66 13.2 23 13.2 L18.8 13.2 C17.2 13.2 17.2 10.8 18.8 10.8 L20.65 10.8 C19.4 9.75 17.8 9.2 16 9.2 C12.245 9.2 9.2 12.245 9.2 16 C9.2 17.6 6.8 17.6 6.8 16 Z';
const refreshParts=[fill('refresh-interior',circle(16,16,5.5),'tint')];
for(let half=0;half<2;half++){const pose=(x,y,t,s)=>{[x,y]=rotate(16+(x-16)*.9,16+(y-16)*.9,half*180);const p=gesture(t,s,[[0,0],[.60,360]],[[0,0],[.06,-12],[.56,372],[.66,356],[.74,360]]);const gap=s?flow([[0,0],[.16,2.4],[.30,1.9],[.44,2.3],[.66,0]],t):0;y+=half?gap:-gap;return rotate(x,y,p);};refreshParts.push(stroke(`refresh-arc-${half}`,refreshArc,{pose}),stroke(`refresh-head-${half}`,refreshHead,{pose}),fill(`refresh-solid-arrow-${half}`,refreshSolid,'solid',{pose}));}
const eye='M6 14.9 C11.1 7.1 20.9 7.1 26 14.9 C26.45 15.58 26.45 16.42 26 17.1 C20.9 24.9 11.1 24.9 6 17.1 C5.55 16.42 5.55 15.58 6 14.9 Z';
const eyeInner=svgpath(eye).translate(-16,-16).scale(.77,.66).translate(16,16).toString();
export const eyeClose=(t,s)=>gesture(t,s,[[0,0],[.07,1],[.10,1],[.23,0]],[[0,0],[.13,0],[.175,1,'hit'],[.21,1],[.27,.06],[.32,.3],[.42,0]]);
const eyePose=(x,y,t,s)=>[x,16+(y-16)*(1-eyeClose(t,s))];
// Circle clipped by two horizontal lid planes. Four circular arcs retain fixed
// topology; the top and bottom are real cut edges, not a scaled rubber pupil.
function clippedPupil(t,s){const r=2.7,h=Math.max(.001,Math.min(r,6.6*(1-eyeClose(t,s))-1.6)),x=Math.sqrt(r*r-h*h),a=Math.asin(h/r),k=4/3*Math.tan(a/4);const p=(x,y)=>[16+x,16+y];return `M${p(-x,-h)} L${p(x,-h)} C${p(x+k*h,-h+k*x)} ${p(r,-k*r)} ${p(r,0)} C${p(r,k*r)} ${p(x+k*h,h-k*x)} ${p(x,h)} L${p(-x,h)} C${p(-x-k*h,h-k*x)} ${p(-r,k*r)} ${p(-r,0)} C${p(-r,-k*r)} ${p(-x-k*h,-h+k*x)} ${p(-x,-h)} Z`;}
const pupilVisibility=(t,s)=>Math.max(0,Math.min(1,(6.6*(1-eyeClose(t,s))-1.6)/.7));
const solidEye=(t,s)=>{const p=eyeClose(t,s),flatten=(d,scale)=>svgpath(d).translate(-16,-16).scale(1,scale).translate(16,16).toString();return hole(flatten(eye,1-.82*p),flatten(eyeInner,1-p));};
const eyeParts=[fill('eye-aperture',eye,'tint',{pose:eyePose}),stroke('eye-lids',eye,{pose:eyePose}),fill('eye-solid-lids',solidEye),fill('eye-pupil',clippedPupil,'outline',{visibility:pupilVisibility}),fill('eye-solid-pupil',clippedPupil,'solid',{visibility:pupilVisibility})];
export function menuOpen(t,s,r){const open=gesture(t,s,[[0,0],[.20,1]],[[0,0],[.05,-.06],[.24,1.08],[.31,.97],[.37,1]]);return r?open:1-open;}
function menuPose(row,x,y,t,s,r){const q=menuOpen(t,s,r),spread=0;let angle=0;const yy=row===0?9+7*q-spread*(1-q):row===2?23-7*q+spread*(1-q):16;
 if(row!==1)angle=(row===0?45:-45)*q;
 if(row===1)x=16+(x-16)*Math.max(0,1-q);
 return rotate(x,yy+(y-(row===0?9:row===2?23:16)),angle,16,16);
}
const menuParts=[fill('menu-panel-interior',(t,s,r)=>{const q=menuOpen(t,s,r);return rect(7,9,18,14,1.5+q*2);},'tint')];
for(let row=0;row<3;row++){const y=[9,16,23][row],pose=(x,y,t,s,r)=>menuPose(row,x,y,t,s,r),visibility=(t,s,r)=>row===1?Math.max(0,Math.min(1,1-menuOpen(t,s,r))):1;menuParts.push(...line(`menu-bar-${row+1}`,[7,y],[25,y],{pose,visibility}));}
const timings=(a,b,posesA,posesB)=>({light:{active:a,rest:a+.04,keyPoses:posesA},strong:{active:b,rest:b+.04,keyPoses:posesB}});
const brief=(name,label,number,parts,intent,timing,extra={})=>({name,label,number:String(number).padStart(2,'0'),stateful:false,heading:label,description:intent,intent,parts,timings:timing,...extra});
export const icons=[
 brief('download','Download',9,transferParts(false),'An arrow approaches a receiving tray and returns.',timings(.48,.74,[0,.08,.16,.21,.34,.48],[0,.12,.24,.32,.52,.74])),
 brief('upload','Upload',10,transferParts(true),'An arrow lifts away from its tray and returns.',timings(.48,.74,[0,.08,.16,.21,.34,.48],[0,.12,.24,.32,.52,.74])),
 brief('copy','Copy',11,copyParts,'The front sheet separates from its source, then aligns.',timings(.50,.78,[0,.08,.17,.22,.36,.50],[0,.12,.24,.34,.56,.78])),
 brief('trash','Trash',12,trashParts,'The lid opens about its right contact and closes.',timings(.48,.78,[0,.08,.16,.22,.34,.48],[0,.12,.23,.34,.49,.60,.78])),
 brief('edit','Edit',13,editParts,'The pencil lowers, writes to the right, lifts, and returns.',timings(.54,.82,[0,.11,.18,.25,.32,.54],[0,.12,.25,.39,.49,.64,.82])),
 brief('send','Send',14,sendParts,'The plane advances along its direction and returns on a short arc.',timings(.53,.78,[0,.04,.20,.25,.38,.53],[0,.06,.27,.34,.55,.78])),
 brief('bookmark','Bookmark',15,bookmarkParts,'The attached ribbon gathers upward and releases.',timings(.50,.76,[0,.08,.16,.21,.36,.50],[0,.055,.24,.32,.52,.76])),
 brief('refresh','Refresh',16,refreshParts,'Two arrow assemblies turn clockwise once and settle.',timings(.62,.80,[0,.12,.24,.36,.48,.62],[0,.06,.20,.36,.56,.66,.80])),
 brief('eye','Eye',17,eyeParts,'The eyelids close and reopen around a protected pupil.',timings(.44,.56,[0,.065,.13,.17,.30,.44],[0,.075,.15,.23,.38,.56])),
 brief('menu','Menu',18,menuParts,'Three bars become an X; the reverse restores the collapsed menu.',timings(.34,.62,[0,.08,.16,.24,.34],[0,.055,.16,.28,.38,.62]),{stateful:true,initialState:'expanded',finalState:'collapsed',forwardLabel:'Close menu',reverseLabel:'Open menu',reverseSuffix:'-open',reverseTimings:timings(.34,.62,[0,.08,.16,.24,.34],[0,.055,.16,.28,.38,.62])}),
];

directActions(icons);
