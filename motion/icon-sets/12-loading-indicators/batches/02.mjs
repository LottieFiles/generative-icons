import{make,tube,circle,phase,sn,cs,point,at,dot,curve,capsulePoint,fill,stroke}from'./helpers.mjs';
const infinity=u=>[16+10*sn(u),16+5*sn(2*u)];
const relay=u=>point(8.5+1.2*cs(3*u),u);
export const icons=[
 make('infinity-run','Infinity runner','A runner passes through a continuous figure eight.',[
  ...tube('track',curve(infinity,32,0,1,true),{visibility:()=>.2}),
  ...Array.from({length:3},(_,i)=>dot('runner-'+i,1,{pose:at((u,s)=>infinity((s?u+.05*sn(2*u):u)-i*.07)),visibility:(_,s)=>i===0||s?1:0})).flat()
 ],'Light follows one runner. Strong adds two spaced followers to show the path direction.'),
 make('bead-relay','Bead relay','Three beads exchange places around a rounded triangle.',[
  ...tube('route',curve(relay,48,0,1,true),{visibility:()=>.2}),
  ...Array.from({length:3},(_,i)=>dot('bead-'+i,1,{pose:at((u,s)=>relay(u+i/3-1/4+(s?sn(3*u)/24:0)))})).flat()
 ],'A steady three-bead relay becomes three smooth surges in Strong, without reversing direction.'),
 make('equalizer-bars','Equalizer','A travelling signal changes five line lengths.',Array.from({length:5},(_,i)=>tube('bar-'+i,(t,s)=>{const u=phase(t),h=s?5.6+3.4*sn(2*u-i/8)+1.8*sn(4*u-i/4):5+3*sn(2*u-i/8);return`M${6+5*i} ${16-h} L${6+5*i} ${16+h}`;})).flat(),'Light uses one travelling waveform; Strong adds a second harmonic and a larger dynamic range.'),
 make('bead-track','Bead track','Packets circulate on a closed capsule track.',[
  ...tube('track',curve(capsulePoint,64,0,1,true),{visibility:()=>.2}),
  ...Array.from({length:4},(_,i)=>dot('packet-'+i,1,{pose:at((u,s)=>capsulePoint(u+i/(s?4:3)+(s?.06*sn(2*u):0))),visibility:(_,s)=>i<3||s?1:0})).flat()
 ],'Three steady packets in Light; four packets surge in Strong. A closed track avoids visible teleporting.'),
 make('planet-orbit','Planet orbit','A fixed center receives continuous orbital activity.',[
  stroke('sun',circle(16,16,2.4)),fill('sun-solid',circle(16,16,3.7),'solid'),fill('sun-tint',circle(16,16,2.4),'tint'),
  ...dot('moon',1,{pose:at(u=>point(10.5,u))}),
  ...dot('inner-moon',1,{pose:at(u=>point(6.3,-2*u)),visibility:(_,s)=>s?1:0})
 ],'A fixed enclosed sun and one steady moon in Light. Strong adds an independent counter-orbit, with clear Bold gaps.')
];
