import{make,tube,fill,stroke,framed,bothFill,rect,polygon,rigid,curve,hole}from'./helpers.mjs';
const parentX=(t,s)=>curve(t,s,[[0,0],[.08,-.4],[.4,3,'out'],[.7,3,'hold'],[1.05,0]],[[0,0],[.1,-1],[.36,6.4,'pass'],[.5,6],[.75,6,'hold'],[1.15,0]]);
const childX=(t,s)=>curve(t,s,[[0,0],[.18,0],[.55,3,'out'],[.82,3,'hold'],[1.2,0]],[[0,0],[.22,0],[.5,6.6,'pass'],[.66,6],[.9,6,'hold'],[1.35,0]]);
const whip=(t,s,solid=false)=>{const a=parentX(t,s),b=childX(t,s);return(solid?`M${10.6+b} 22 L`:'M')+`${9+b} 22 C${4+b} 22 ${4+a} 13 ${9+a} 13`+(solid?` L${9+a} 11.4`:'');};
const parent=make('parent-layers','Parent layers','A parent moves first and pulls its linked child through a flexible pick whip. Strong doubles the travel and gives the child a short delayed catch-up.',[
 ...framed('parent',rect(5,7,17,6,3),undefined,{pose:rigid((t,s)=>({dx:parentX(t,s)}))}),...framed('child',rect(9,19,13,6,3),undefined,{pose:rigid((t,s)=>({dx:childX(t,s)}))}),stroke('whip',(t,s)=>whip(t,s)),stroke('whip-solid',(t,s)=>whip(t,s,true),{style:'solid',width:2.6})
],1.2,1.35);parent.duotoneFill=true;
const gather=(t,s)=>({dx:curve(t,s,[[0,0],[.5,5,'out']],[[0,0],[.08,-.6],[.32,5]]),dy:curve(t,s,[[0,0],[.5,3.1,'out']],[[0,0],[.08,-.4],[.32,3.1]])});
const precompose=make('precompose','Precompose','Separate layers gather into an aligned composition. Its enclosing frame appears after the layers land; Strong gathers them faster after a brief outward preparation.',[
 ...framed('top',rect(4,4,12,3.6,1.8),undefined,{pose:rigid(gather)}),...framed('middle',rect(9,14.2,12,3.6,1.8)),...framed('bottom',rect(14,24.4,12,3.6,1.8),undefined,{pose:rigid((t,s)=>{const a=gather(t,s);return{dx:-a.dx,dy:-a.dy};})}),
 stroke('frame',rect(5.5,3.6,21,24.8,4),{visibility:(t,s)=>curve(t,s,[[0,0],[.51,0],[.72,1]],[[0,0],[.33,0],[.52,1]])}),fill('frame-solid',hole(rect(4.2,2.3,23.6,27.4,5.3),rect(6.8,4.9,18.4,22.2,2.7)),'solid',{visibility:(t,s)=>curve(t,s,[[0,0],[.51,0],[.72,1]],[[0,0],[.33,0],[.52,1]])})
],.72,.52,['top','bottom','frame']);precompose.duotoneFill=true;
const hx=(t,s)=>curve(t,s,[[0,0],[.55,-6,'out']],[[0,0],[.1,.6],[.36,-6.4,'pass'],[.48,-6]]);
const trim=make('trim-layer','Trim layer','The out-point handle cuts the layer duration while the time ruler stays fixed. Strong pulls back, trims decisively and settles once.',[
 ...framed('bar',(t,s)=>rect(5,12,18+hx(t,s),6,3)),...tube('handle','M27.5 9 L27.5 21',{pose:rigid((t,s)=>({dx:hx(t,s)}))}),...tube('baseline','M5 25 L27 25')
],.55,.48,['bar','handle']);trim.duotoneFill=true;
const cut=(t,s)=>curve(t,s,[[0,0],[.14,3,'hit'],[.3,11,'out'],[.42,11,'hold'],[.8,0]],[[0,0],[.08,-.8],[.16,3,'hit'],[.26,11],[.4,11,'hold'],[.85,0]]);
const splitX=(t,s)=>curve(t,s,[[0,0],[.14,0],[.26,3.5,'out']],[[0,0],[.16,0],[.3,3.7,'pass'],[.42,3.5]]);
const split=make('split-layer','Split layer','A razor contacts the clip and the two cut halves separate. Strong prepares the blade, drives it down faster and opens the split with a short recoil.',[
 ...framed('left',polygon([[8,16],[16,16],[16,22],[8,22]],[3,0,0,3]),polygon([[8,16],[16.1,16],[16.1,22],[8,22]],[3,0,0,3]),{pose:rigid((t,s)=>({dx:-splitX(t,s)}))}),...framed('right',polygon([[16,16],[24,16],[24,22],[16,22]],[0,3,3,0]),polygon([[15.9,16],[24,16],[24,22],[15.9,22]],[0,3,3,0]),{pose:rigid((t,s)=>({dx:splitX(t,s)}))}),...tube('razor','M16 5 L16 11',{pose:rigid((t,s)=>({dy:cut(t,s)}))})
],.8,.85,['left','right']);split.duotoneFill=true;
const width=(t,s)=>curve(t,s,[[0,13],[.6,21,'out']],[[0,13],[.1,12.4],[.4,21.8,'pass'],[.55,21]]);
const stretch=make('time-stretch','Time stretch','Extending the clip spreads its keyframe times in the same proportion. Strong first compresses the interval, then makes a decisive extension and settles.',[
 ...framed('bar',(t,s)=>rect(5,17,width(t,s),6,3)),...tube('baseline','M5 27 L27 27'),...[.3,.7].flatMap((f,i)=>bothFill('key-'+i,polygon([[5+13*f,9.1],[6.9+13*f,11],[5+13*f,12.9],[3.1+13*f,11]],.4),{pose:rigid((t,s)=>({dx:f*(width(t,s)-13)}))}))
],.6,.55,['bar','key']);stretch.duotoneFill=true;
export const layers={'parent-layers':parent,precompose,'trim-layer':trim,'split-layer':split,'time-stretch':stretch};
