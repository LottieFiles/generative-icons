import svgpath from'svgpath';
import{toggle,make,tube,stroke,fill,bothFill,framed,polygon,circle,holes,rigid,curve,cubicEase,mix}from'./helpers.mjs';
function keyChange(remove){
 const name=remove?'remove-keyframe':'add-keyframe';
 const rail=(t,s,r)=>r?(remove?curve(t,false,[[0,16],[.3,8.5,'out']],[]):curve(t,false,[[0,8.5],[.25,8.5],[.5,16,'out']],[])):remove?curve(t,s,[[0,8.5],[.5,8.5],[.8,16,'out']],[[0,8.5],[.38,8.5],[.56,16]]):curve(t,s,[[0,16],[.06,16],[.3,8.5,'out']],[[0,16],[.04,16],[.2,8.5]]);
 const y=(t,s,r)=>r?(remove?(t<.3?16:21):(t<.5?21:16)):remove?curve(t,s,[[0,21],[.1,21],[.5,16,'out']],[[0,21],[.08,21],[.15,21.5,'launch'],[.38,16]]):curve(t,s,[[0,16],[.3,16],[.7,21,'out']],[[0,16],[.2,16],[.44,21.7,'hit'],[.56,20.8],[.68,21]]);
 const alpha=(t,s,r)=>r?(remove?curve(t,false,[[0,0],[.3,0],[.5,1,'linear']],[]):curve(t,false,[[0,1],[.2,0,'linear']],[])):remove?curve(t,s,[[0,1],[.3,1],[.5,0,'linear']],[[0,1],[.24,1],[.38,0]]):curve(t,s,[[0,0],[.3,0],[.42,1,'linear']],[[0,0],[.2,0],[.28,1]]);
 const command=remove?['M12.5 5.55 L19.5 5.55']:['M13.5 5.55 L18.5 5.55','M16 3.3 L16 7.8'];
 const icon=toggle(name,remove?'Remove keyframe':'Add keyframe',remove?'The keyframe lifts out, then the track closes. Strong prepares the key and removes it with a faster main stroke.':'The track opens before a new keyframe lands. Strong drops the key decisively, then settles once.',[
 ...command.flatMap((d,i)=>tube('command-'+i,d,{pose:rigid((t,s,r)=>({dy:r?0:curve(t,s,[[0,0],[.1,.6,'out'],[.26,0]],[[0,0],[.08,1],[.24,0]])}))})),
 ...tube('rail-left',(t,s,r)=>`M5 21 L${rail(t,s,r)} 21`),...tube('rail-right',(t,s,r)=>`M${32-rail(t,s,r)} 21 L27 21`),
 ...framed('key',polygon([[16,12.5],[19.5,16],[16,19.5],[12.5,16]],.6),undefined,{pose:rigid((t,s,r)=>({dy:y(t,s,r)-16})),visibility:alpha})
 ],remove?.8:.7,remove?.56:.68,remove?'Key present':'Prepared',remove?'Key removed':'Key added',.52,.52);
 Object.assign(icon,{duotoneFill:true,previewReset:true,reverseSuffix:'-reset',reverseLabel:'Reset preview'});return icon;
}
const angle=(t,s)=>s?360*cubicEase((t-.12)/.8,.6,0,.1,1):curve(t,false,[[0,0],[.1,0],[1,360]],[]);
const hand=rigid((t,s)=>({angle:angle(t,s),pivot:[16,18]})),crown=rigid((t,s)=>({dy:curve(t,s,[[0,0],[.1,.5,'out'],[.26,0]],[[0,0],[.12,.9],[.3,0]])}));
const handHole=polygon([[15.2,18],[15.2,14.6],[14.5,13.4],[16,11.9],[17.5,13.4],[16.8,14.6],[16.8,18]],.5);
const auto=make('auto-keyframe','Auto keyframe','A crown press records a key while the dial completes one clockwise cycle. Strong uses a faster central turn, without reversing time.',[
 ...tube('crown-bar','M14 6 L18 6',{pose:crown}),stroke('crown-stem','M16 6 L16 8.5',{pose:crown}),{...stroke('crown-stem-solid','M16 6 L16 9.5',{pose:crown}),style:'solid',width:2.6},
 fill('body-interior',circle(16,18,8.5),'tint'),stroke('body',circle(16,18,8.5)),fill('body-solid',(t,s)=>holes(circle(16,18,8.5),svgpath(handHole).rotate(angle(t,s),16,18).toString())),
 fill('pivot',circle(16,18,1),'outline'),stroke('needle','M16 18 L16 14.6',{pose:hand}),fill('key-tip',polygon([[16,11.9],[17.5,13.4],[16,14.9],[14.5,13.4]],.4),'outline',{pose:hand})
],1,1);auto.duotoneFill=true;auto.cycleDegrees=360;
const pointsA=[16,7,19,10,22,13,25,16,22,19,19,22,16,25,13,22,10,19,7,16,10,13,13,10,16,7];
const pointsB=[16,7,21.6,7,19.2,11.5,19.2,16,19.2,20.5,21.6,25,16,25,10.4,25,12.8,20.5,12.8,16,12.8,11.5,10.4,7,16,7];
const easingPath=(t,s,r)=>{const m=r?curve(t,false,[[0,1],[.5,0]],[]):curve(t,s,[[0,0],[.55,1,'out']],[[0,0],[.08,-.06],[.4,1.08],[.56,.97],[.68,1]]),p=pointsA.map((v,i)=>mix(v,pointsB[i],m));return'M'+p.slice(0,2)+[0,1,2,3].map(i=>' C'+p.slice(2+i*6,8+i*6)).join('')+' Z';};
const easing=toggle('keyframe-easing','Keyframe easing','A linear keyframe changes into an eased keyframe. Strong prepares the silhouette, pinches farther and settles to the same key type.',framed('key-type',easingPath),.55,.68,'Linear','Eased',.5,.5);easing.duotoneFill=true;
export const keyframing={'add-keyframe':keyChange(false),'remove-keyframe':keyChange(true),'auto-keyframe':auto,'keyframe-easing':easing};
