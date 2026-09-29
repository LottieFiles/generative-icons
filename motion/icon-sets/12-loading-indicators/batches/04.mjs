import{make,tube,circle,phase,sn,cs,point,at,dot,curve,curvedArc,fill,rigid}from'./helpers.mjs';
const band=(x,u,s,side)=>{const q=(x-6)/10-u*(s?2:1),y=2*sn(q)+(s?.6*sn(2*q):0);return side===0?11+y:21-y;};
const morph=(u,s)=>{const blend=(1-cs(u))/2,n=2+(s?3:2)*blend,rotation=s?Math.PI/2*u:0;return a=>{const q=a*Math.PI*2,qs=q-rotation,c=Math.cos(qs),v=Math.sin(qs),r=9/Math.pow(Math.abs(c)**n+Math.abs(v)**n,1/n);const aspect=Math.sqrt(1+(s?.16:.10)*sn(u));return[16+r*Math.cos(q)*aspect,16+r*Math.sin(q)/aspect];};};
const atom=(u,k)=>{const x=10.5*cs(u),y=4.5*sn(u),a=k*Math.PI/3;return[16+x*Math.cos(a)-y*Math.sin(a),16+x*Math.sin(a)+y*Math.cos(a)];};
export const icons=[
 make('sweep-hand','Sweep hand','A clean hand sweeps a circular dial.',[
  ...tube('dial',circle(16,16,11.3),{visibility:()=>.2}),
  ...dot('hub',1,{pose:at(()=>[16,16])}),
  ...tube('hand','M16 12.5 L16 9.2',{pose:rigid((t,s)=>({angle:360*(phase(t)+(s?.06*sn(phase(t)):0))}))}),
  ...tube('trail',(t,s)=>{const u=phase(t),h=u+(s?.06*sn(u):0),span=(1+(s?.6*cs(u):0))/6;return curvedArc(16,16,8,h-.25-1/12-span,span);},{visibility:(_,s)=>s?.6:0})
 ],'Light keeps a clear hand, hub and dial. Strong adds a short trailing arc with separate paint clearance.'),
 make('helix-strands','Helix','Two flat signal bands move with attached crossbars.',[
  ...[0,1].flatMap(i=>tube('strand-'+i,(t,s)=>curve(v=>{const x=6+20*v;return[x,band(x,phase(t),s,i)];},8))),
  ...[9,16,23].flatMap((x,i)=>tube('rung-'+i,(t,s)=>`M${x} ${band(x,phase(t),s,0)} L${x} ${band(x,phase(t),s,1)}`))
 ],'Rungs remain attached to two separate flat wave bands. Strong adds a second harmonic and greater signal activity.'),
 make('atom-rings','Atom','Three paths carry separate circulating electrons.',[
  ...[0,1,2].flatMap(i=>tube('orbit-'+i,curve(u=>atom(u,i),16,0,1,true),{visibility:()=>.35})),
  ...[0,1,2].flatMap(i=>dot('electron-'+i,1,{pose:at((u,s)=>atom(u*(s?[1,-1,2][i]:1)+i/3,i))})),
  ...dot('nucleus',1,{pose:at(()=>[16,16])})
 ],'Three fixed flat orbits. Strong keeps the first electron steady, counter-rotates the second, and doubles the third, with integer turns and no simulated depth.'),
 make('morph-ring','Morph ring','One continuous outline changes between rounded forms.',tube('contour',(t,s)=>curve(morph(phase(t),s),8,0,1,true)),'Light changes a circle into a rounded square. Strong turns the evolving contour through rounded diamonds before returning to the same circle.'),
 make('stack-cycle','Stack cycle','Three horizontal bars exchange places on a closed route.',Array.from({length:3},(_,i)=>tube('bar-'+i,'M-3 0 L3 0',{pose:rigid((t,s)=>{const u=phase(t),q=u+i/3+(s?.045*sn(3*u):0);return{dx:16+7.5*sn(q),dy:16-(s?8:6.5)*cs(q),angle:s?14*sn(q):0,pivot:[0,0]};}),visibility:(t,s)=>{const q=phase(t)+i/3+(s?.045*sn(3*phase(t)):0);return.55+.45*(1+cs(q))/2;}})).flat(),'Three rigid horizontal strokes circulate on a closed oval. Strong opens the spacing and adds a periodic handoff surge; nothing teleports.')
];
