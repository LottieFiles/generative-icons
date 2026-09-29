// Claude proposal-03. Head splay, bracket flex and label-led drawing.
import {route,arrowhead,annotation,prefix,visible} from './reveal.mjs';
import {tube,rect,framed,stroke,rigid} from './helpers.mjs';
import {driver,phase,flex,anchor} from './drivers.mjs';
const ep=phase(.10,.95,.06,.56,.08,.24),eh=phase(.95,1.25,.56,.72,0,.08);
const angle=driver([[0,45]],[[.56,30],[.72,53],[.86,42],[.98,45]],[[0,45]]);
const barb=sign=>(t,s,r)=>{const a=angle(t,s,r)*Math.PI/180;return [25-5.66*Math.cos(a),8+sign*5.66*Math.sin(a)];};
const elbow=annotation('annotation-elbow','Elbow callout','The line turns through a rounded elbow; the arrowhead opens and settles.',[
 ...anchor('origin',8,25,2.2),...route('route','M8 25 L8 11.5 C8 9.567 9.567 8 11.5 8 L25 8',ep),...arrowhead('head',[25,8],barb(-1),barb(1),eh)
],1.3,1,.28);
const leader=phase(.10,.55,.06,.30,.14,.26),bracket=phase(.55,1.15,.30,.66,0,.14);
const ba=flex([[.60,0],[.72,-4],[.86,1.2],[.98,0]]);
const bracketIcon=annotation('annotation-bracket','Bracket callout','The source connects, then the bracket opens to mark a range.',[
 ...anchor('origin',24,16,2.2),...route('leader','M24 16 L11 16',leader),
 ...route('upper','M11 16 L11 8.5 C11 7.12 9.88 6 8.5 6 L6 6',bracket,{pose:rigid((t,s,r)=>({angle:ba(t,s,r),pivot:[11,16]}))}),
 ...route('lower','M11 16 L11 23.5 C11 24.88 9.88 26 8.5 26 L6 26',bracket,{pose:rigid((t,s,r)=>({angle:-ba(t,s,r),pivot:[11,16]}))})
],1.2,1,.28);
const cp=phase(.10,.85,.06,.50,.08,.26),ch=phase(.85,1.15,.50,.66,0,.08),cf=flex([[.50,0],[.60,.8],[.76,-.25],[.92,0]]);
const cd=solid=>(t,s,r)=>{const k=cf(t,s,r);return `M12 ${solid?12:13} C12 ${(solid?17.2:17.5)+1.4*k} ${17+k} ${17+k} 20 20 L25.5 25.5`;};
const callout=annotation('annotation-callout','Callout pointer','A label box sends a curved leader toward the marked detail.',[
 ...framed('label',rect(5,5,14,8,3)),
 ...[false,true].map(solid=>({name:solid?'route-solid':'route',style:solid?'solid':'outline',type:'stroke',width:solid?2.6:2,d:(t,s,r)=>prefix(cd(solid)(t,s,r),cp(t,s,r)),visibility:(t,s,r)=>visible(cp(t,s,r))})),
 ...arrowhead('head',[25.5,25.5],[20.3,25.5],[25.5,20.3],ch)
],1.2,1,.28,true);
export const callouts=[elbow,bracketIcon,callout];
