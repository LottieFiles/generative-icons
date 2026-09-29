import {icon,progress,kick,F,box,framed,tube,fill,rect,circle,polygon,posed,shown,rigid,ringRect} from './helpers.mjs';
const auto=icon('auto-layout','Auto layout','Three free child objects become an evenly padded row inside their container. Light: each child moves directly into place. Strong: the outer children prepare in opposite directions and the row closes quickly. Reverse restores the free positions.',[
 ...box('container',4,5,24,22,2.2),
 ...[ [9.5,19],[16,12],[22.5,14] ].flatMap(([x,y],j)=>posed(framed('child-'+j,rect(x-1.8,y-2.8,3.6,5.6,.8)),rigid((t,s,r)=>({pivot:[x,y],dy:(16-y)*progress(t,s,r)+(j===0?1:j===2?-1:0)*kick(t,s)}))))
],{state:['Free layout','Automatic row'],fixed:['container-frame'],rigidParts:['child-0-frame','child-1-frame','child-2-frame']});
// Two generic six-sided cells communicate linked instances without app-specific marks.
const cell=(cx,cy)=>polygon([[cx-4,cy-2],[cx,cy-4.4],[cx+4,cy-2],[cx+4,cy+2],[cx,cy+4.4],[cx-4,cy+2]],.65);
const nod=F([[0,0],[.24,1],[.72,0]],[[0,0],[.08,-.45],[.28,2.4],[.48,-.4],[.74,0]]);
const comp=icon('component-instance','Component instance','Two matching six-sided objects share a live connector. Light: the instance moves one unit and the connector follows. Strong: the instance moves 2.4 units with a short opposite preparation and returns to the same source relationship.',[
 ...framed('source',cell(9,10)),
 ...tube('link',(t,s)=>`M9 14.4 L9 21 L${18+nod(t,s)} 21`),
 ...posed(framed('instance',cell(22,21)),rigid((t,s)=>({pivot:[22,21],dx:nod(t,s)})))
],{fixed:['source-frame'],rigidParts:['instance-frame']});
const sep=(t,s,r)=>2.2*progress(t,s,r)-.65*kick(t,s);
const detach=icon('detach-instance','Detach instance','Separate an instance from its source while preserving both objects. Light: the connector retracts and the instance moves away. Strong: it prepares toward the source, then pulls 2.2 units clear. Reverse reconnects the two cells.',[
 ...framed('source',cell(9,10)),
 ...tube('link-source',(t,s,r)=>`M9 14.4 L9 ${21-4*progress(t,s,r)}`),
 ...shown(tube('link-instance',(t,s,r)=>`M9 21 L${19+sep(t,s,r)} 21`),(t,s,r)=>1-progress(t,s,r)),
 ...posed(framed('instance',cell(22,21)),rigid((t,s,r)=>({pivot:[22,21],dx:sep(t,s,r)})))
],{state:['Linked instance','Detached instance'],fixed:['source-frame'],rigidParts:['instance-frame']});
const resize=F([[0,0],[.3,1],[.76,0]],[[0,0],[.08,-.45],[.3,2.5],[.5,1],[.76,0]]);
const pin=icon('pin-constraints','Pin constraints','A child remains pinned to the top and left edges while its container resizes. Light: the frame grows one unit. Strong: the frame draws in then grows 2.5 units. The child and its two pin contacts remain fixed.',[
 ...box('child',11,11,8,8,1.3),
 ...tube('frame',(t,s)=>{const d=resize(t,s);return rect(5,5,21+d,21,.8);}),
 ...tube('pin-top','M15 5 L15 11'),...tube('pin-left','M5 15 L11 15'),
 ...tube('pin-top-cap','M12.8 5 L17.2 5'),...tube('pin-left-cap','M5 12.8 L5 17.2')
],{fixed:['child-frame','pin-top','pin-left']});
const frameSize=F([[0,0],[.28,1],[.78,0]],[[0,0],[.1,-.6],[.3,2.5],[.5,.5],[.78,0]]);
const artboard=icon('artboard-frame','Artboard frame','Resize a complete artboard from its fixed upper-left origin. Light: the board expands one unit. Strong: it draws in, expands 2.5 units, and settles. The four short frame ticks remain attached to the moving edges.',[
 ...framed('board',(t,s)=>rect(8,8,16+frameSize(t,s),16+frameSize(t,s),.8),(t,s)=>ringRect(8,8,16+frameSize(t,s),16+frameSize(t,s),.8)),
 ...Array.from({length:8},(_,j)=>tube('tick-'+j,(t,s)=>{const e=24+frameSize(t,s);return [`M8 4.5 L8 8`,`M4.5 8 L8 8`,`M${e} 4.5 L${e} 8`,`M${e} 8 L${e+2} 8`,`M8 ${e} L4.5 ${e}`,`M8 ${e} L8 ${e+2}`,`M${e} ${e} L${e+2} ${e}`,`M${e} ${e} L${e} ${e+2}`][j];})).flat()
],{light:.9,strong:.9});
export const icons=[auto,comp,detach,pin,artboard];
