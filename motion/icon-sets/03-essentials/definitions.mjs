import { directEssentials } from '../library/directed-essentials.mjs';
import { flow } from '../library/direction.mjs';
import svgpath from 'svgpath';

export const mix=(a,b,p)=>a+(b-a)*p;
export function phrase(keys,t){if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i];return mix(a[1],b[1],(1-Math.cos(Math.PI*(t-a[0])/(b[0]-a[0])))/2);}
export function rotate(x,y,a,cx=16,cy=16){const r=a*Math.PI/180,c=Math.cos(r),s=Math.sin(r);return[cx+(x-cx)*c-(y-cy)*s,cy+(x-cx)*s+(y-cy)*c];}
export const circle=(x,y,r)=>`M${x} ${y-r} C${x+r*.552285} ${y-r} ${x+r} ${y-r*.552285} ${x+r} ${y} C${x+r} ${y+r*.552285} ${x+r*.552285} ${y+r} ${x} ${y+r} C${x-r*.552285} ${y+r} ${x-r} ${y+r*.552285} ${x-r} ${y} C${x-r} ${y-r*.552285} ${x-r*.552285} ${y-r} ${x} ${y-r} Z`;
export const rect=(x,y,w,h,r)=>`M${x+r} ${y} L${x+w-r} ${y} C${x+w-r*.448} ${y} ${x+w} ${y+r*.448} ${x+w} ${y+r} L${x+w} ${y+h-r} C${x+w} ${y+h-r*.448} ${x+w-r*.448} ${y+h} ${x+w-r} ${y+h} L${x+r} ${y+h} C${x+r*.448} ${y+h} ${x} ${y+h-r*.448} ${x} ${y+h-r} L${x} ${y+r} C${x} ${y+r*.448} ${x+r*.448} ${y} ${x+r} ${y} Z`;
// One final Close is required by this Engine exporter. Symmetric holes can
// reverse winding through an x reflection, joined by a zero-width seam.
export const hole=(outer,inner,cx=16)=>outer.replace(/Z\s*$/,'')+' '+svgpath(inner).scale(-1,1).translate(cx*2,0).toString().replace(/^M/,'L');
function roundedPolygon(vertices,r){const corners=vertices.map((v,i)=>{const before=vertices[(i+vertices.length-1)%vertices.length],after=vertices[(i+1)%vertices.length];const radius=Array.isArray(r)?r[i%r.length]:r;const step=p=>{const d=Math.hypot(p[0]-v[0],p[1]-v[1]);return[v[0]+(p[0]-v[0])*radius/d,v[1]+(p[1]-v[1])*radius/d];};return {v,a:step(before),b:step(after)};});return corners.map(({v,a,b},i)=>`${i?'L':'M'}${a} C${mix(a[0],v[0],2/3)} ${mix(a[1],v[1],2/3)} ${mix(b[0],v[0],2/3)} ${mix(b[1],v[1],2/3)} ${b}`).join(' ')+' Z';}
function capsule(a,b,width){const len=Math.hypot(b[0]-a[0],b[1]-a[1]);return svgpath(rect(-width/2,-width/2,len+width,width,width/2)).rotate(Math.atan2(b[1]-a[1],b[0]-a[0])*180/Math.PI).translate(...a).toString();}
const fill=(name,d,style='solid',extra={})=>({name,d,style,...extra});
const stroke=(name,d,extra={})=>({name,d,style:'outline',type:'stroke',width:2,...extra});
const starVertices=(outer,inner)=>Array.from({length:10},(_,i)=>{const a=(-90+i*36)*Math.PI/180,r=i%2?inner:outer;return[16+Math.cos(a)*r,16.4+Math.sin(a)*r];});
const starOutline=roundedPolygon(starVertices(10.5,5.1),.85),starSolid=roundedPolygon(starVertices(10.7,4.9),[1.8,.85]);
function starPose(x,y,t,strong){const angle=phrase(strong?[[0,0],[.06,-6],[.25,28],[.31,28],[.58,-3],[.74,0]]:[[0,0],[.045,-1.5],[.16,8],[.32,-.8],[.44,0]],t);[x,y]=rotate(x,y,angle,16,16.4);return[x,y-angle/(strong?28:8)*(strong?.85:.22)];}

function checkProgress(t,strong,reverse){if(reverse)return 1-(strong?flow([[0,0],[.07,.12],[.30,1]],t):phrase([[0,0],[.25,1]],t));return strong?flow([[0,0],[.035,.03],[.10,.72,'pass'],[.17,1.02],[.24,1]],t):phrase([[0,0],[.10,.42],[.32,1]],t);}
// Keep the painted check clear of its ring at the strongest pose and boldest weight.
function checkPoints(t,strong,reverse){const p=checkProgress(t,strong,reverse),q=checkProgress(Math.max(0,t-(reverse?0:strong?.025:.02)),strong,reverse);return [[mix(13.4,9.8,p),mix(18.5,16.4,p)],[13.4,mix(18.5,20.1,p)],[mix(13.42,21.0,q),mix(18.48,12.3,q)]].map(([x,y])=>[16+(x-16)*.85,16+(y-16)*.85]);}
function checkLine(t,s,r){const [a,b,c]=checkPoints(t,s,r);const along=(p,d)=>{const n=Math.hypot(p[0]-b[0],p[1]-b[1]);return[b[0]+(p[0]-b[0])*Math.min(.65,d)/Math.max(n,.001),b[1]+(p[1]-b[1])*Math.min(.65,d)/Math.max(n,.001)];};const e=along(a,Math.hypot(a[0]-b[0],a[1]-b[1])*.25),f=along(c,Math.hypot(c[0]-b[0],c[1]-b[1])*.25);return `M${a} L${e} C${mix(e[0],b[0],2/3)} ${mix(e[1],b[1],2/3)} ${mix(f[0],b[0],2/3)} ${mix(f[1],b[1],2/3)} ${f} L${c}`;}
function checkVisible(t,strong,reverse){return reverse?phrase([[0,1],[strong?.22:.17,1],[strong?.30:.25,0]],t):phrase([[0,0],[.05,1]],t);}
const checkParts=[fill('check-tint',circle(16,16,10.35),'tint'),stroke('check-rim',circle(16,16,10.35)),fill('check-solid-rim',hole(circle(16,16,10.7),circle(16,16,8.3))),stroke('check-mark',checkLine,{visibility:checkVisible}),fill('check-solid-short',(t,s,r)=>{const[a,b]=checkPoints(t,s,r);return capsule(a,b,2.65);},'solid',{visibility:checkVisible}),fill('check-solid-long',(t,s,r)=>{const[,b,c]=checkPoints(t,s,r);return capsule(b,c,2.65);},'solid',{visibility:checkVisible})];

function sliderX(row,t,strong){const delay=row?(strong?.055:.035):0;const phase=phrase(strong?[[0,0],[.05,-.06],[.25,1],[.34,1],[.74,0]]:[[0,0],[.15,1],[.42,0]],t-delay);return(row?21:11)+phase*(row?-1:1)*(strong?6.2:2.0);}
const sliderParts=[];
for(let row=0;row<2;row++){
 const y=row?21:11,center=(t,s)=>sliderX(row,t,s);
 sliderParts.push(fill(`sliders-tint-${row+1}`,(t,s)=>circle(center(t,s),y,3),'tint'));
 for(const side of ['left','right']){
  const ends=(t,s)=>side==='left'?[[6,y],[center(t,s)-3,y]]:[[center(t,s)+3,y],[26,y]];
  sliderParts.push(stroke(`sliders-rail-${row+1}-${side}`,(t,s)=>{const[a,b]=ends(t,s);return `M${a} L${b}`;}));
  sliderParts.push(fill(`sliders-solid-rail-${row+1}-${side}`,(t,s)=>{const[a,b]=ends(t,s);return capsule(a,b,1.8);}));
 }
 sliderParts.push(stroke(`sliders-handle-${row+1}`,(t,s)=>circle(center(t,s),y,3)),fill(`sliders-solid-handle-${row+1}`,(t,s)=>circle(center(t,s),y,3.4)));
}

const lockBody=rect(7.5,14.5,17,12,2.5),lockSolid=hole(rect(7.5,14.5,17,12,2.5),rect(14.8,18.7,2.4,4.3,1.2));
// End both shackle legs on the body top edge. The right joint is the rigid pivot.
const shackle='M10.7 14.5 L10.7 11 C10.7 8.073 13.073 5.7 16 5.7 C18.927 5.7 21.3 8.073 21.3 11 L21.3 14.5';
// Closed vector band for the solid U, including its rounded free ends.
const solidShackle='M9.4 14.5 L9.4 11 C9.4 7.355 12.355 4.4 16 4.4 C19.645 4.4 22.6 7.355 22.6 11 L22.6 14.5 C22.6 15.218 20 15.218 20 14.5 L20 11 C20 8.791 18.209 7 16 7 C13.791 7 12 8.791 12 11 L12 14.5 C12 15.218 9.4 15.218 9.4 14.5 Z';
function lockAngle(t,strong,reverse){return reverse?(strong?flow([[0,0],[.20,31.5],[.29,27.2],[.40,28]],t):phrase([[0,0],[.36,28]],t)):(strong?flow([[0,28],[.06,33,'launch'],[.23,0,'hit'],[.29,1.4],[.40,0]],t):phrase([[0,28],[.32,0],[.40,0]],t));}
function lockRecoil(t,strong,reverse){return strong?flow(reverse?[[0,0],[.05,.45,'hit'],[.16,0]]:[[0,0],[.23,0],[.29,.75,'hit'],[.36,-.2],[.44,0]],t):0;}
const lockPose=(moving)=>(x,y,t,s,r)=>{if(moving)[x,y]=rotate(x,y,lockAngle(t,s,r),21.3,14.5);return[x,y+lockRecoil(t,s,r)];};
const lockParts=[stroke('lock-shackle',shackle,{pose:lockPose(true)}),fill('lock-solid-shackle',solidShackle,'solid',{pose:lockPose(true)}),fill('lock-tint',lockBody,'tint',{pose:lockPose(false)}),stroke('lock-body',lockBody,{pose:lockPose(false)}),stroke('lock-keyway','M16 19.4 L16 21.9',{pose:lockPose(false)}),fill('lock-solid-body',lockSolid,'solid',{pose:lockPose(false)})];

const folderBody='M8 7.5 L12 7.5 C12.6 7.5 13.1 7.74 13.5 8.18 L15.35 10.2 L24 10.2 C25.1 10.2 26 11.1 26 12.2 L26 23.6 C26 24.925 24.925 26 23.6 26 L8.4 26 C7.075 26 6 24.925 6 23.6 L6 9.5 C6 8.4 6.9 7.5 8 7.5 Z';
function folderOpen(t,strong){return phrase(strong?[[0,0],[.055,-.10],[.24,1],[.36,1],[.78,0]]:[[0,0],[.17,1],[.23,1],[.46,0]],t)*(strong?4.4:1.8);}
const folderParts=[fill('folder-tint',folderBody,'tint'),stroke('folder-shell',folderBody),stroke('folder-front-edge',(t,s)=>{const y=14.1+folderOpen(t,s);return `M6.5 ${y} L25.5 ${y}`;}),fill('folder-solid', (t,s)=>hole(folderBody,rect(7.75,13.25+folderOpen(t,s),16.5,1.7,.85)))];

export const icons=[
 {name:'star',label:'Star',number:'04',stateful:false,heading:'A turn toward attention.',description:'A precise turn. A clear peak. Back to still.',intent:'Acknowledgement of a featured item; no favorite state is changed.',parts:[fill('star-tint',starOutline,'tint',{pose:starPose}),stroke('star-body',starOutline,{pose:starPose}),fill('star-solid',starSolid,'solid',{pose:starPose})],timings:{light:{active:.46,rest:.50,keyPoses:[0,.045,.16,.24,.32,.46]},strong:{active:.76,rest:.80,keyPoses:[0,.06,.25,.31,.45,.58,.76]}}},
 {name:'check',label:'Check',number:'05',stateful:true,initialState:'pending',finalState:'complete',forwardLabel:'Confirm',reverseLabel:'Reset',heading:'Confirmation that stays.',description:'The short arm leads. The long arm opens. Hold complete.',intent:'An explicit completion transition; the result remains until Reset is selected.',parts:checkParts,timings:{light:{active:.36,rest:.40,keyPoses:[0,.05,.10,.20,.32,.36]},strong:{active:.30,rest:.34,keyPoses:[0,.035,.075,.10,.17,.30]}},reverseTimings:{light:{active:.28,rest:.32,keyPoses:[0,.07,.14,.22,.28]},strong:{active:.34,rest:.38,keyPoses:[0,.07,.15,.22,.30,.34]}}},
 {name:'sliders',label:'Sliders',number:'06',stateful:false,heading:'A deliberate adjustment.',description:'One handle leads. The other answers. Both return.',intent:'A demonstration of adjustment controls, not actual stored setting values.',parts:sliderParts,timings:{light:{active:.46,rest:.50,keyPoses:[0,.08,.15,.185,.28,.46]},strong:{active:.80,rest:.85,keyPoses:[0,.05,.25,.305,.395,.57,.80]}}},
 {name:'lock',label:'Lock',number:'07',stateful:true,initialState:'open',finalState:'locked',forwardLabel:'Lock',reverseLabel:'Unlock',heading:'A latch with intent.',description:'The shackle closes. The body receives it. Hold locked.',intent:'A connected padlock closes and retains its state; Unlock is an explicit reverse.',parts:lockParts,timings:{light:{active:.42,rest:.45,keyPoses:[0,.08,.16,.24,.32,.42]},strong:{active:.46,rest:.50,keyPoses:[0,.06,.15,.23,.29,.36,.46]}},reverseTimings:{light:{active:.40,rest:.45,keyPoses:[0,.08,.18,.28,.40]},strong:{active:.42,rest:.46,keyPoses:[0,.05,.12,.20,.29,.42]}}},
 {name:'folder',label:'Folder',number:'08',stateful:false,heading:'Open the folder.',description:'The front opens from its lower edge, then closes.',intent:'A folder-opening preview; it returns to the closed rest pose after the demonstration.',parts:folderParts,timings:{light:{active:.48,rest:.52,keyPoses:[0,.08,.17,.23,.34,.48]},strong:{active:.80,rest:.85,keyPoses:[0,.055,.24,.36,.52,.68,.80]}}},
];

directEssentials(icons);
