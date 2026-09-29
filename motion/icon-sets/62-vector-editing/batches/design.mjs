import {icon,progress,kick,F,framed,box,tube,fill,stroke,bothFill,rect,circle,node,polygon,posed,shown,rigid,ringRect,annulus,rotate} from './helpers.mjs';
const lift=F([[0,0],[.26,-1.4],[.78,0]],[[0,0],[.08,.55],[.3,-4],[.5,-1.5],[.78,0]]);
const exportSlice=icon('export-slice','Export slice','Lift a selected slice from a dashed export region. Light: the tile lifts 1.4 units. Strong: it prepares downward, lifts four units, and returns to the selected region. The export arrow remains separate above the tile.',[
 ...['M5 12 L5 17','M5 22 L5 27 L10 27','M15 27 L20 27','M25 27 L27 27 L27 22','M27 17 L27 12'].flatMap((d,j)=>tube('region-'+j,d)),
 ...posed(framed('slice',rect(10,16,12,7,1.5)),rigid((t,s)=>({pivot:[16,19.5],dy:lift(t,s)}))),
 ...tube('arrow','M16 8.4 L16 4.5'),...tube('arrow-head','M12.5 8 L16 4.5 L19.5 8')
],{fixed:['region-0','arrow','arrow-head'],rigidParts:['slice-frame']});
// Same endpoints, distinct topology in separate source/result parts; only opacity changes topology ownership.
const pixels='M6 24 L6 18 L11 18 L11 11 L17 11 L17 16 L22 16 L22 20 L26 20 L26 24 Z';
const vector='M6 24 C8 24 9 8 14 8 C19 8 19 24 26 24 Z';
const vectorize=icon('vectorize-image','Vectorize image','Convert a stair-stepped raster silhouette to a smooth editable vector curve. Light: the stepped edge resolves into the curve. Strong: the pixels spread slightly before a quick trace. Reverse restores the visible raster silhouette.',[
 ...shown(posed(framed('pixels',pixels),rigid((t,s)=>({pivot:[16,24],dy:kick(t,s)*1.2}))), (t,s,r)=>1-progress(t,s,r)),
 ...shown(framed('vector',vector),(t,s,r)=>progress(t,s,r)),
 ...[ [6,24],[14,8],[26,24] ].flatMap(([x,y],j)=>shown(bothFill('node-'+j,node(x,y,1.25)),(t,s,r)=>progress(t,s,r)))
],{state:['Raster silhouette','Vector silhouette']});
const blendMove=F([[0,0],[.26,.7],[.8,0]],[[0,0],[.08,-.35],[.3,1.5],[.52,-.3],[.8,0]]);
const blend=icon('blend-steps','Blend steps','Preview a shape blend from a circle through a rounded intermediate to a square. Light: the intermediate shifts 0.7 units along the blend. Strong: it moves 1.5 units with a short preparation and return. Endpoint shapes stay fixed.',[
 ...framed('circle',circle(6.2,16,2.5)),
 ...posed(framed('intermediate',rect(13,12.6,4.6,6.8,2.2)),rigid((t,s)=>({pivot:[15.3,16],dx:blendMove(t,s)}))),
 ...framed('square',rect(23,11.7,5,8.6,.8))
],{fixed:['circle-frame','square-frame'],rigidParts:['intermediate-frame']});
const slide=F([[0,0],[.3,1.7],[.8,0]],[[0,0],[.1,-1],[.32,4],[.52,1.5],[.8,0]]);
const gradient=icon('gradient-stops','Gradient stops','Adjust a middle color stop on a closed tonal ramp. Light: the middle stop moves 1.7 units. Strong: it prepares left, sweeps four units right, and settles. Its dividing boundary follows the stop, with the two end stops fixed.',[
 ...box('ramp',5,7,22,11,1.4),
 ...shown([fill('ramp-low',rect(6.4,8.4,4.7,8.2,.3),'outline'),fill('ramp-low-solid',rect(6.4,8.4,4.7,8.2,.3),'solid')],()=>.24),
 ...shown([fill('ramp-mid',(t,s)=>rect(11.1,8.4,2.9+slide(t,s),8.2,.3),'outline'),fill('ramp-mid-solid',(t,s)=>rect(11.1,8.4,2.9+slide(t,s),8.2,.3),'solid')],()=>.52),
 ...[fill('ramp-high',(t,s)=>rect(14+slide(t,s),8.4,11.6-slide(t,s),8.2,.3),'outline'),fill('ramp-high-solid',(t,s)=>rect(14+slide(t,s),8.4,11.6-slide(t,s),8.2,.3),'solid')],
 ...[6,14,26].flatMap((x,j)=>posed([
  ...tube('stem-'+j,`M${x} 18 L${x} 22`),
  ...framed('stop-'+j,rect(x-1.7,22,3.4,4.3,.7))
 ],rigid((t,s)=>({pivot:[x,18],dx:j===1?slide(t,s):0}))))
],{fixed:['ramp-frame']});
const nodeNudge=F([[0,0],[.26,1],[.78,0]],[[0,0],[.1,-.4],[.3,2.5],[.52,.65],[.8,0]]);
const nodesVisible=F([[0,1],[.15,.4],[.45,1],[.8,1]],[[0,1],[.1,.2],[.3,1],[.8,1]]);
const curves=icon('convert-to-curves','Convert to curves','Inspect an outlined letter as editable curve nodes. Light: three node squares separate one unit and seat back on their anchors. Strong: the nodes travel 2.5 units before seating. The full letter silhouette remains visible throughout.',[
 ...tube('letter','M6.5 25 L14.4 6.4 C15 5 17 5 17.6 6.4 L25.5 25'),...tube('crossbar','M10.5 16.5 L21.5 16.5'),
 ...[ [6.5,25,1,0],[16,6,0,1],[25.5,25,-1,0] ].flatMap(([x,y,dx,dy],j)=>posed(shown(framed('node-'+j,node(x,y,1.6)),nodesVisible),rigid((t,s)=>({pivot:[x,y],dx:dx*nodeNudge(t,s),dy:dy*nodeNudge(t,s)}))))
],{fixed:['letter']});
const turn=F([[0,0],[.28,18],[.78,0]],[[0,0],[.1,-8],[.32,48],[.54,10],[.8,0]]);
const direction=icon('path-direction','Path direction','Inspect the winding direction of a closed path. Light: the tangent arrow sweeps 18 degrees clockwise along the contour. Strong: it draws back eight degrees, sweeps 48 degrees, and returns. The closed path and arrow share a fixed centre.',[
 ...framed('path',circle(16,16,9),annulus(16,16,9,2.6)),
 ...bothFill('anchor-left',node(7,16,1.5)),...bothFill('anchor-bottom',node(16,25,1.5)),
 ...posed(tube('arrow','M15.5 3.7 L19.1 7.55 L14.6 10.1'),rigid((t,s)=>({pivot:[16,16],angle:turn(t,s)})))
],{fixed:['path-frame'],rigidParts:['arrow']});
export const icons=[exportSlice,vectorize,blend,gradient,curves,direction];
