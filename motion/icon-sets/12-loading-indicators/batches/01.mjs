import{make,tube,circle,curvedArc,phase,sn,cs,point,at,dot,rigid,spin,mod}from'./helpers.mjs';
export const icons=[
 make('orbit-comet','Comet ring','A comet traces one continuous orbit.',[
  ...tube('track',circle(16,16,10),{visibility:()=>.2}),
  ...tube('comet',(t,s)=>{const u=phase(t),h=u-(s?.06*sn(2*u):0),span=(70+(s?80*(1-cs(2*u))/2:0))/360;return curvedArc(16,16,10,h-span,span);})
 ],'Fixed orbit; Light has a constant short arc. Strong lengthens its tail twice each revolution.'),
 make('counter-arcs','Counter arcs','Two arcs counter-rotate in separate lanes.',[
  ...tube('outer',curvedArc(16,16,10.5,0,1/3),{pose:spin()}),
  ...tube('inner',(t,s)=>curvedArc(16,16,6,-phase(t)*(s?2:1),(100+20*s*(1-cs(2*phase(t))))/360))
 ],'Outer driver rotates steadily. Strong inner arc adds an extra turn and a travelling length change.'),
 make('wave-dots','Wave dots','A signal travels through five spaced points.',Array.from({length:5},(_,i)=>dot('point-'+i,1,{pose:at((u,s)=>[6+5*i+(s?.9*cs(2*u-i/10):0),16+(s?5:2.5)*sn(2*u-i/10)+(s?1.5*sn(4*u-i/5):0)])})).flat(),'Five round-capped points follow a left-to-right wave. Strong adds visible travel and a second harmonic.'),
 make('ripple-rings','Ripple','Signals expand from a fixed source.',[
  ...dot('source',1,{pose:at(()=>[16,16])}),
  ...Array.from({length:4},(_,i)=>tube('ring-'+i,(t,s)=>circle(16,16,3+7.5*mod(phase(t)-i/(s?4:3))),{visibility:(t,s)=>{if(!s&&i===3)return 0;const p=mod(phase(t)-i/(s?4:3));if(p<.07||p>.93)return 0;const a=(p-.07)/.86;return Math.sin(Math.PI*a)**2*(1-.5*a);}})).flat()
 ],'Three rings in Light, four in Strong. Each radius recycles only during a zero-opacity guard; the source remains visible.'),
 make('corner-relay','Corner relay','Attention passes clockwise between four corners.',Array.from({length:4},(_,i)=>tube('corner-'+i,'M9 14 L9 11 C9 9.8954 9.8954 9 11 9 L14 9',{pose:rigid((t,s)=>{const e=2.4*((1+cs(phase(t)-i/4))/2)**2;return{angle:90*i,dx:s?(-1*[1,-1,-1,1][i]*e):0,dy:s?(-1*[1,1,-1,-1][i]*e):0};}),visibility:t=>.3+.7*((1+cs(phase(t)-i/4))/2)**2})).flat(),'Fixed rounded L marks in Light; Strong also opens the active corner. No fill is added between strokes.')
];
