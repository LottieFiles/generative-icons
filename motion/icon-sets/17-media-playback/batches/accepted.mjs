// Accepted mechanisms from the first actual Claude proposals.
import{make,toggle,tube,stroke,fill,framed,rect,polygon,rigid,posed,curve}from'./helpers.mjs';
const stop=make('stop','Stop','The stop block lands firmly. Strong lifts farther, brakes at contact and settles with one small rebound.',framed('block',rect(9,9,14,14,2.5),rect(8,8,16,16,3.5),{pose:rigid((t,s)=>({dy:curve(t,s,[[0,0],[.16,-1.5,'out'],[.34,.4,'in'],[.50,0,'out']],[[0,0],[.14,-3.2],[.32,.9,'hit'],[.44,-.5],[.58,0]])}))}),.52,.6);stop.duotoneFill=true;
function skip(previous){
 const name=previous?'previous-track':'next-track',sign=previous?-1:1;
 const translate=rigid((t,s)=>({dx:sign*curve(t,s,[[0,0],[.28,2.2,'out'],[.30,-3],[.56,0,'out']],[[0,0],[.12,-1.2],[.30,2.2,'launch'],[.31,2.2,'hold'],[.32,-3,'hold'],[.52,.6,'pass'],[.66,0]])}));
 const visible=(t,s)=>curve(t,s,[[0,1],[.14,1],[.27,0,'out'],[.30,0],[.46,1,'out']],[[0,1],[.16,1],[.29,0],[.32,0],[.44,1]]);
 const outline=polygon(previous?[[25,9],[14,16],[25,23]]:[[7,9],[18,16],[7,23]],1.2),solid=polygon(previous?[[26,8],[12.5,16],[26,24]]:[[6,8],[19.5,16],[6,24]],1.5);
 const icon=make(name,previous?'Previous track':'Next track','A track moves toward the boundary; the next track arrives from the opposite side. Strong has a prepared launch, boundary response and controlled arrival.',[
 ...framed('track',outline,solid,{pose:translate,visibility:visible}),
 ...tube('stop-bar',`M${previous?8:24} 9 L${previous?8:24} 23`,{pose:rigid((t,s)=>({dx:s?sign*curve(t,true,[],[[0,0],[.30,0],[.36,.7,'hit'],[.56,0]]):0}))})
 ],.58,.68);icon.duotoneFill=true;return icon;
}
const next=skip(false),previous=skip(true);
function shuffleShift(t,s,index){return curve(t,s,[[0,0],[.34,1.6,'out'],[.78,0]],[[0,0],[.1+index*.06,-.8],[.34+index*.06,3,'pass'],[.58+index*.06,-.6],[.82+index*.06,0]]);}
const shuffle=make('shuffle','Shuffle','Two crossed routes send tracks into different lanes. Strong extends each route farther with a short stagger and recoil.',[0,1].flatMap(i=>{
 const start=i?10:22,end=32-start,dir=end===22?1:-1;
 const d=(t,s,solid=false)=>{const dx=shuffleShift(t,s,i);return `M4.5 ${start} L10.5 ${start} C13.5 ${start} 15 ${start===10?12:20} 16 16 C${17+.6*dx} ${end===22?20:12} ${18.5+dx} ${end} ${21.5+dx} ${end} L${(solid?23.5:25.5)+dx} ${end}`;};
 const pose=rigid((t,s)=>({dx:shuffleShift(t,s,i)}));
 return [stroke('route-'+i,(t,s)=>d(t,s)),{...stroke('route-'+i+'-solid',(t,s)=>d(t,s,true)),style:'solid',width:2.6},stroke('head-'+i,`M22.7 ${end-2.8} L25.5 ${end} L22.7 ${end+2.8}`,{pose}),fill('head-'+i+'-solid',polygon([[22,end-3.6],[26.3,end],[22,end+3.6]],.8),'solid',{pose})];
}),.78,.88);shuffle.duotoneFill=false;
const spin=rigid((t,s)=>({angle:curve(t,s,[[0,0],[.6,180,'out']],[[0,0],[.12,-14],[.5,193,'pass'],[.68,177],[.86,180]]),pivot:[16,16]}));
const repeat=make('repeat','Repeat','Two arrows turn to the next equivalent loop position. Strong winds up briefly, turns decisively and settles.',posed([
 stroke('top','M8 17 L8 12.5 C8 10 10 8 12.5 8 L24 8'),stroke('head-top','M21.2 5.2 L24 8 L21.2 10.8'),stroke('bottom','M24 15 L24 19.5 C24 22 22 24 19.5 24 L8 24'),stroke('head-bottom','M10.8 21.2 L8 24 L10.8 26.8'),
 {...stroke('top-solid','M8 17 L8 12.5 C8 10 10 8 12.5 8 L22.2 8'),style:'solid',width:2.6},{...stroke('bottom-solid','M24 15 L24 19.5 C24 22 22 24 19.5 24 L9.8 24'),style:'solid',width:2.6},
 fill('head-top-solid',polygon([[20.6,4.6],[24.9,8],[20.6,11.4]],.8)),fill('head-bottom-solid',polygon([[11.4,27.4],[7.1,24],[11.4,20.6]],.8))
],spin),.6,.86);repeat.duotoneFill=false;repeat.cycleDegrees=180;
const brackets=['M8.5 13 L8.5 8.5 L13 8.5','M19 8.5 L23.5 8.5 L23.5 13','M8.5 19 L8.5 23.5 L13 23.5','M19 23.5 L23.5 23.5 L23.5 19'];
const expansion=(t,s,r)=>r?curve(t,s,[[0,3],[.04,3],[.44,0]],[[0,3],[.04,3],[.2,3.6],[.44,0]]):curve(t,s,[[0,0],[.06,0],[.46,3,'out']],[[0,0],[.04,0],[.16,-1],[.34,4,'pass'],[.5,2.6,'hit'],[.68,3]]);
const fullscreen=toggle('fullscreen','Fullscreen','Four window corners expand into fullscreen. Strong gathers inward, opens farther and settles at the same selected size.',brackets.flatMap((d,i)=>tube('corner-'+i,d,{pose:rigid((t,s,r)=>({dx:(i%2?1:-1)*expansion(t,s,r),dy:(i<2?-1:1)*expansion(t,s,r)}))})),.46,.68,'Windowed','Fullscreen',.44,.44);fullscreen.duotoneFill=false;
export const accepted={stop,'next-track':next,'previous-track':previous,shuffle,repeat,fullscreen};
