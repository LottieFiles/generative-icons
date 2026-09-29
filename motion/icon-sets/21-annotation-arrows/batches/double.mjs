// Claude revision-double geometry; revision-connectors retains the causal reset.
import {route,annotation,tube} from './reveal.mjs';
import {phase,flex} from './drivers.mjs';
const p=phase(0,.86,0,.48,.08,.28),h=phase(.86,1.04,.48,.60,0,.08);
const f=flex([[.08,0],[.48,2,'pass'],[.74,-.5],[.96,0]]);
const length=Math.hypot(3.8,8.8),ux=3.8/length,uy=-8.8/length;
const body=(t,s,r)=>{const k=f(t,s,r);return `M6.5 22 L8.7 16.9 C${12.5+ux*k} ${8.1+uy*k} ${19.5-ux*k} ${8.1+uy*k} 23.3 16.9 L25.5 22`;};
export const double=[annotation('annotation-double-ended','Curved double pointer','The source arrow anchors a broad arch that finishes with a second head.',[
 ...tube('head-source','M6.5 22 L10.2 20.5 L6.5 22 L5 18.3'),...route('body',body,p),...route('head-target','M25.5 22 L21.8 20.5 L25.5 22 L27 18.3',h)
],1.08,1,.28)];
