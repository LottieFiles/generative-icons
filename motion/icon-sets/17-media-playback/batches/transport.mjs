import svgpath from 'svgpath';
import {make,toggle,tube,stroke,fill,framed,rect,polygon,rigid,curve,arc,mix} from './helpers.mjs';
const bezier=(v,o,i)=>'M'+v[0]+v.map((p,k)=>{const n=(k+1)%v.length;return ' C'+[p[0]+o[k][0],p[1]+o[k][1],v[n][0]+i[n][0],v[n][1]+i[n][1],...v[n]];}).join('')+' Z';
const z=Array.from({length:8},()=>[0,0]);
const playV=[[10,9],[11.4,7.9],[23.6,15.2],[24.7,16],[23.6,16.8],[11.4,24.1],[10,23],[10,16]],playO=[[0,-1],[0,0],[.75,.45],[0,.5],[0,0],[-.85,.55],[0,1],[0,0]],playI=[[0,0],[-.85,-.55],[0,0],[0,-.5],[.75,-.45],[0,0],[0,0],[0,0]];
const pauseV=[[10.5,9],[12.5,7],[14.5,9],[14.5,16],[14.5,23],[12.5,25],[10.5,23],[10.5,16]],pauseO=[[0,-1.1],[1.1,0],[0,0],[0,0],[0,1.1],[-1.1,0],[0,0],[0,0]],pauseI=[[0,0],[-1.1,0],[0,-1.1],[0,0],[0,0],[1.1,0],[0,1.1],[0,0]];
const clock=(t,s,r)=>r?Math.max(0,(s?.50:.46)-t):t;
const morph=(t,s,r)=>curve(clock(t,s,r),s,[[0,0],[.06,0],[.38,1,'out']],[[0,0],[.12,0],[.24,.75,'pass'],[.30,1]]);
const tip=(t,s,r)=>s?curve(clock(t,s,r),true,[],[[0,0],[.12,1],[.30,-.5,'pass'],[.42,0]]):0;
const tri=(t,s,r)=>{const p=morph(t,s,r),dx=tip(t,s,r);return polygon([[mix(11,12.5,p),mix(8,9,p)],[mix(24,12.5,p)+dx,16],[mix(11,12.5,p),mix(24,23,p)],[mix(11,12.5,p),16]],1.2);};
const solidTri=(t,s,r)=>{const p=morph(t,s,r),dx=tip(t,s,r),lerp=(a,b)=>a.map((v,k)=>v.map((n,j)=>mix(n,b[k][j],p)));const v=lerp(playV,pauseV);for(const k of [2,3,4])v[k][0]+=dx;return bezier(v,lerp(playO,pauseO),lerp(playI,pauseI));};
const height=(t,s,r)=>curve(clock(t,s,r),s,[[0,0],[.26,0],[.44,7,'out']],[[0,0],[.25,0],[.36,7.6,'pass'],[.46,7]]);
const rightPose=rigid((t,s,r)=>({dx:s?curve(clock(t,s,r),true,[],[[0,0],[.25,0],[.36,.6,'pass'],[.48,0]]):0}));
const rightVisible=(t,s,r)=>curve(clock(t,s,r),s,[[0,0],[.26,0],[.34,1,'out']],[[0,0],[.25,0],[.31,1]]);
const playPause=toggle('play-pause','Play / Pause','The play triangle folds into the left pause bar. The right bar grows in the space it clears. Strong pushes forward, folds decisively and settles with a small response.',[
 ...framed('tri-bar',tri,solidTri),stroke('bar-right',(t,s,r)=>`M19.5 ${16-height(t,s,r)} L19.5 ${16+height(t,s,r)}`,{pose:rightPose,visibility:rightVisible}),fill('bar-right-solid',(t,s,r)=>rect(17.5,14-height(t,s,r),4,4+2*height(t,s,r),2),'solid',{pose:rightPose,visibility:rightVisible})
],.46,.5,'Play','Pause',.46,.5);playPause.duotoneFill=true;
function seek(forward){
 const name=forward?'forward-10':'replay-10',mirror=d=>forward?svgpath(d).matrix([-1,0,0,1,32,0]).toString():d;
 const ring='M13.9 5.5 L16 5.5 '+arc(16,16,10.5,-90,225).replace(/^M[^C]+/,'');
 const pose=rigid((t,s)=>({angle:(forward?-1:1)*curve(t,s,[[0,0],[.86,-360,'io']],[[0,0],[.14,16],[.50,-374,'pass'],[.64,-360]]),pivot:[16,16]}));
 const icon=make(name,forward?'Forward 10 seconds':'Replay 10 seconds','The arrow makes a full directional turn around a fixed 10. Strong winds up, drives through the turn and settles once. The numeral stays upright.',[
 ...tube('ring-body',mirror(ring),{pose}),...tube('ring-head',mirror('M15.3 3.5 L13.9 5.5 L15.44 7.7'),{pose}),...tube('digit-one','M11.75 15 L12.8 13.5 L12.8 18.5'),...tube('digit-zero',rect(16.45,13.5,3.6,5,1.8))
 ],.86,.66);icon.duotoneFill=false;icon.cycleDegrees=360;return icon;
}
const playlist=make('playlist','Playlist','The selected track moves into play. Strong launches the play marker farther and the list follows with a short, orderly stagger.',[
 ...[8,14,20].flatMap((y,i)=>tube('row-'+i,`M6 ${y} L${i===2?11.5:25} ${y}`,{pose:rigid((t,s)=>({dx:s?curve(t,true,[],[[0,0],[.28+i*.07,0],[.46+i*.07,1.6,'pass'],[.70+i*.07,-.3],[.90+i*.07,0]]):0}))})),
 ...framed('play-marker',polygon([[17.5,17.5],[23.5,22],[17.5,26.5]],1),undefined,{pose:rigid((t,s)=>({dx:curve(t,s,[[0,0],[.30,2,'out'],[.75,0]],[[0,0],[.10,-.6],[.32,2.8,'pass'],[.58,-.5],[.85,0]])}))})
],.75,1.04);playlist.duotoneFill=true;
const queueP=(t,s,r)=>r?(t<.16?1:0):curve(t,s,[[0,0],[.30,0],[.70,1,'out']],[[0,0],[.14,0],[.36,1]]);
const queueA=(t,s,r)=>r?curve(t,false,[[0,1],[.08,1],[.15,0,'out']],[]):curve(t,s,[[0,0],[.30,0],[.42,1,'out']],[[0,0],[.14,0],[.20,1]]);
const queue=toggle('add-to-queue','Add to queue','The add control presses and releases as a new row extends into the queue. Strong gives the press more travel and writes the row in one fast move.',[
 ...tube('row-0','M6 8 L26 8'),...tube('row-1','M6 14 L26 14'),...tube('row-2','M6 20 L15 20'),
 ...['M20.5 22 L27.5 22','M24 18.5 L24 25.5'].flatMap((d,i)=>tube('plus-'+i,d,{pose:rigid((t,s,r)=>({angle:r?0:curve(t,s,[[0,0],[.20,-15,'out'],[.45,0]],[[0,0],[.08,12],[.22,-40,'pass'],[.36,8],[.48,0]]),dy:r?0:curve(t,s,[[0,0],[.14,.6,'out'],[.40,0]],[[0,0],[.10,.9,'pass'],[.26,-.3],[.40,0]]),pivot:[24,22]}))})),
 ...tube('new-row',(t,s,r)=>`M6 26 L${6+9*queueP(t,s,r)} 26`,{visibility:queueA})
],.70,.48,'Prepared','Queued',.28,.30);Object.assign(queue,{duotoneFill:false,previewReset:true,reverseSuffix:'-reset',reverseLabel:'Reset preview'});
export const transport={'play-pause':playPause,'replay-10':seek(false),'forward-10':seek(true),playlist,'add-to-queue':queue};
