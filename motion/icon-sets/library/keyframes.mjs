// Design: Claude Code proposals 03/04; integration/native verification: Codex.
import svgpath from 'svgpath';
import {polygon,rect,capsule,framed,bothFill,tube,fill,rigid} from './drawing.mjs';
import {track,drive as motion,timePair} from './direction.mjs';

const inset=.9/(2*Math.sqrt(2));
const apex={oldLeft:7+inset,oldRight:14-inset,newLeft:18+inset,newRight:25-inset,top:16.5+inset};
const end=2.4;
const time=(t,reverse)=>reverse?(t<.15?end:0):t;
const resetOpacity=(t,reverse)=>reverse?track([[0,1],[.12,0,'out'],[.18,0],[.30,1,'out']],t):1;
const lift=(t,s)=>motion(t,s,[[0,0],[.05,0],[.22,1.5,'in'],[.55,0,'out']],[[0,0],[.055,.25,'launch'],[.20,-3.3],[.38,2.2,'hit'],[.49,1.3],[.66,-.25],[.76,0]]);
const clearance=(t,s)=>motion(t,s,[[0,0],[.20,1,'out']],[[0,0],[.25,0],[.36,1]]);
const arrival=(t,s)=>motion(t,s,[[0,0],[.22,0],[.34,1,'out']],[[0,0],[.38,0],[.49,1]]);
const diamond=x=>polygon([[x-3.5,20],[x,16.5],[x+3.5,20],[x,23.5]],.9);
const solidDiamond=x=>polygon([[x-4.9,20],[x,15.1],[x+4.9,20],[x,24.9]],2);

// All capsules share one paint and one opacity. Retraced connectors enclose
// zero area; even where capsules overlap, reset opacity is applied only once.
function joinedCapsules(paths){
 const starts=paths.map(d=>svgpath(d).abs().segments[0].slice(1));
 const loops=paths.map((d,i)=>d.replace(/Z\s*$/,' L'+starts[i]));
 return loops[0]+loops.slice(1).map((d,i)=>' L'+starts[i+1]+d.replace(/^M[^CL]+/,'')+' L'+starts[0]).join(' ')+' Z';
}
const branches=(t,s,r,w,solid=false)=>{
 const now=time(t,r),p=clearance(now,s),half=w/2;
 const leftEnd=21.5+((solid?19:apex.newLeft-half)-21.5)*p;
 const rightStart=21.5+((solid?24:apex.newRight+half)-21.5)*p;
 const stemEnd=20+((solid?17.5:apex.top-half)-20)*p;
 return joinedCapsules([
  capsule([solid?13:apex.oldRight+half,20],[leftEnd,20],w),
  capsule([rightStart,20],[27.5,20],w),
  capsule([21.5,10.5+lift(now,s)],[21.5,stemEnd],w),
 ]);
};
const parts=[
 ...tube('keyframes-rail-left',(t,s,r,w)=>`M4.5 20 L${w===2.6?8:apex.oldLeft-w/2} 20`,{widthAware:true}),
 ...framed('keyframes-key-old',diamond(10.5),solidDiamond(10.5)),
 fill('keyframes-rail-stem',branches,'outline',{widthAware:true,visibility:(t,s,r)=>resetOpacity(t,r)}),
 fill('keyframes-rail-stem-solid',(t,s,r)=>branches(t,s,r,2.6,true),'solid',{visibility:(t,s,r)=>resetOpacity(t,r)}),
 ...framed('keyframes-key-new',diamond(21.5),solidDiamond(21.5),{visibility:(t,s,r)=>arrival(time(t,r),s)*resetOpacity(t,r)}),
 ...bothFill('keyframes-tab',rect(19,7.5,5,3,1.25),{pose:rigid((t,s,r)=>({dy:lift(time(t,r),s)}))}),
];

export const keyframes={
 name:'keyframes',label:'Keyframes',number:'59',heading:'Keyframes',parts,
 description:'The playhead presses, the track clears, and a new keyframe stays on the timeline.',
 intent:'Cause: playhead press. Action: track clears. Response: a rigid diamond appears. Result: two keys remain; the preview resets separately.',
 revision:'claude-directed-v1',stateful:true,previewReset:true,
 initialState:'prepared',finalState:'result',reverseSuffix:'-reset',forwardLabel:'Add keyframe',reverseLabel:'Reset preview',
 timings:timePair(.55,.80,[0,.10,.20,.22,.28,.34,.55],[0,.055,.20,.36,.38,.49,.66,.80]),
 reverseTimings:timePair(.30,.30,[0,.06,.12,.15,.22,.30],[0,.06,.12,.15,.22,.30]),
};
