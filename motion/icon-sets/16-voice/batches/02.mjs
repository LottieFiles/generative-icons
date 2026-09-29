import{make,toggle,tube,stroke,fill,framed,bothFill,circle,capsule,hole,rigid,posed,curve,timePair}from'./helpers.mjs';
import{geometry as G}from'./geometry.mjs';
const g=G['microphone-muted'];
const muteProgress=(t,s,r)=>r?curve(t,s,[[0,1],[.30,.04,'out']],[[0,1],[.24,.04]]):curve(t,s,[[0,.04],[.36,1,'out']],[[0,.04],[.05,.04,'hold'],[.20,1.05,'hit'],[.30,1]]);
const groove=capsule([14.014,10.674],[17.139,14.299],1.5);
const muted=toggle('microphone-muted','Microphone muted','The slash closes the microphone input. Strong draws decisively through the full diagonal with a short end response. The selected mute state remains.',[
 ...framed('capsule',g.capsule,hole(g.capsule,groove)),...tube('cradle',g.cradle),...tube('stem',g.stem),...tube('foot',g.foot),
 ...tube('slash',(t,s,r)=>{const p=muteProgress(t,s,r);return `M9.5 8.5 L${9.5+12.5*p} ${8.5+14.5*p}`;},{visibility:(t,s,r)=>muteProgress(t,s,r)<.06?0:1})
],.36,.30,'Unmute','Mute',.30,.24);
muted.duotoneFill=true;muted.constraints=['capsule-frame','cradle','stem','foot'].map(p=>({type:'fixed',part:'microphone-muted-'+p}));
const wake=G['wake-word'];
const wakePose=rigid((t,s)=>({dy:curve(t,s,[[0,0],[.16,-2.2,'out'],[.5,0]],[[0,0],[.06,.6],[.2,-3.6],[.32,.5,'hit'],[.46,-.3],[.6,0]])}));
const wakeWord=make('wake-word','Wake word','A smart speaker acknowledges its wake word with an upward nod. Strong prepares, rises farther and settles once; the receive arc stays attached to the dot.',[
 ...framed('puck',wake.puck),...posed(bothFill('dot',circle(16,14,1.5)),wakePose),...posed(tube('arc',wake.arc),wakePose)
],.5,.6);
wakeWord.duotoneFill=true;wakeWord.constraints=[{type:'fixed',part:'wake-word-puck-frame'}];
const sw=G['switch-voice'];
function marker(t,s,r){
 const dx=r?curve(t,s,[[0,13],[.45,0]],[[0,13],[.1,14.2],[.5,0,'hit']]):curve(t,s,[[0,0],[.45,13]],[[0,0],[.1,-1.2],[.4,13,'hit']]);
 const dy=r?curve(t,s,[[0,0],[.22,-1.4,'out'],[.45,0,'in']],[[0,0],[.1,-.4],[.3,-2.0],[.5,0,'hit']]):curve(t,s,[[0,0],[.22,-1.4,'out'],[.45,0,'in']],[[0,0],[.1,-.4],[.25,-2.0],[.4,0,'hit']]);return{dx,dy};
}
const switchVoice=toggle('switch-voice','Switch voice','The voice marker transfers from one speaker to the other and stays there. Strong has a prepared leap and a brief receiving nod.',[
 ...['a','b'].flatMap((who,i)=>[...framed('head-'+who,sw['head-'+who],sw['head-'+who],{pose:rigid((t,s,r)=>({dy:s&&i===(r?0:1)?curve(t-(r?.1:0),true,[],[[0,0],[.4,0,'hit'],[.47,.4],[.6,-.1],[.7,0]]):0}))}),...tube('body-'+who,sw['body-'+who])]),
 ...tube('marker',sw.marker,{pose:rigid(marker)})
],.45,.7,'Voice A','Voice B',.45,.8);
switchVoice.duotoneFill=true;
switchVoice.constraints=['body-a','body-b'].map(p=>({type:'fixed',part:'switch-voice-'+p}));
const noiseDriver=(t,s,r,outer)=>{
 t-=outer?(s?.06:.05):0;
 if(r){return {k:curve(t,s,[[0,.12],[.35,1,'out']],[[0,.12],[.25,1.12],[.4,1]]),dx:curve(t,s,[[0,.6],[.35,0,'out']],[[0,.6],[.25,-.1],[.4,0]]),alpha:curve(t,s,[[0,.5],[.35,1,'out']],[[0,.5],[.3,1]])};}
 return {k:curve(t,s,[[0,1],[.4,.12]],[[0,1],[.08,1.25],[.3,.12,'hit']]),dx:curve(t,s,[[0,0],[.4,.6]],[[0,0],[.08,-.15],[.3,.6,'hit']]),alpha:curve(t,s,[[0,1],[.4,.5]],[[0,1],[.08,1],[.3,.5]])};
};
const isolation=toggle('voice-isolation','Voice isolation','The voice remains steady while surrounding noise contracts and moves aside. Strong briefly gathers the noise before a decisive reduction.',[
 ...tube('voice-l','M12 12 L12 20'),...tube('voice-mid','M16 9 L16 23'),...tube('voice-r','M20 12 L20 20'),
 ...[4,8,24,28].flatMap((x,i)=>tube('noise-'+i,(t,s,r)=>{const d=noiseDriver(t,s,r,i===0||i===3),h=(i===0||i===3?1.5:3)*d.k,xx=x+(i<2?-1:1)*d.dx;return `M${xx} ${16-h} L${xx} ${16+h}`;},{visibility:(t,s,r)=>noiseDriver(t,s,r,i===0||i===3).alpha}))
],.45,.36,'Mixed voice','Isolated voice',.4,.46);
isolation.duotoneFill=false;isolation.constraints=['voice-l','voice-mid','voice-r'].map(p=>({type:'fixed',part:'voice-isolation-'+p}));
const sp=G['voice-speed'];
const tick=(t,s,r,i)=>i===0?0:r?curve(t,s,[[0,-1.4*i],[.4,0,'out']],[[0,-1.4*i],[.06,-1.2*i],[.34,0,'hit']]):curve(t,s,[[0,0],[.4,-1.4*i,'out']],[[0,0],[.08,.25*i],[.26,-1.4*i,'hit']]);
const speed=toggle('voice-speed','Voice speed','Cadence marks gather toward Play to select faster speech. Strong prepares outward, then closes the spacing with a faster drive.',[
 ...framed('play',sp.play,sp.play,{pose:rigid((t,s,r)=>({dx:s&&!r?curve(t,true,[],[[0,0],[.26,0,'hit'],[.32,-.4],[.44,.1],[.52,0]]):0}))}),
 ...[1,2,3].flatMap((i)=>tube('tick-'+i,sp['tick-'+i],{pose:rigid((t,s,r)=>({dx:tick(t,s,r,i-1)}))}))
],.4,.52,'Normal speed','Faster speech',.4,.34);
speed.duotoneFill=true;speed.constraints=[{type:'fixed',part:'voice-speed-tick-1'}];
export const icons=[muted,wakeWord,switchVoice,isolation,speed];
