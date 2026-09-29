import{make,tube,circle,phase,sn,cs,point,at,dot,curve,squarePoint,curvedArc,fill,stroke}from'./helpers.mjs';
const ellipse=(x,y,rx,ry)=>curve(u=>[x+rx*cs(u),y+ry*sn(u)],8,0,1,true);
const ballShape=(t,s)=>{const u=phase(t),v=2*u;if(!s){const h=9*(1-cs(v))/2;return[16,22.5-h-2.5,2.5,2.5];}const m=v-Math.floor(v),pulse=((1+cs(v))/2)**8,h=12*Math.sin(Math.PI*m),q=.24*pulse-.1*Math.abs(Math.cos(Math.PI*m))*(1-pulse),rx=2.5*(1+q),ry=2.5*(1-q);return[16,22.5-h-ry,rx,ry];};
export const icons=[
 make('square-snake','Square snake','A short stroke follows a drafted square circuit.',[
  ...tube('frame',curve(squarePoint,64,0,1,true),{visibility:()=>.2}),
  ...tube('snake',(t,s)=>{const u=phase(t),h=s?u-.03*sn(4*u):u,span=.25+(s?.1*(1-cs(4*u)):0);return curve(squarePoint,24,h,h+span);})
 ],'The highlight keeps moving through rounded corners. Strong lengthens its span four times per cycle.'),
 make('sync-arrows','Sync arrows','A pair of arrows circulates continuously.',Array.from({length:2},(_,i)=>[
  ...tube('arc-'+i,(t,s)=>{const u=phase(t),h=s?u+.04*sn(2*u):u,span=(120+(s?30*sn(2*u):0))/360;return curvedArc(16,16,10,h+i/2,span);}),
  ...tube('head-'+i,(t,s)=>{const u=phase(t),q=(s?u+.04*sn(2*u):u)+i/2+(120+(s?30*sn(2*u):0))/360,p=point(10,q),a=q+.25+.5-35/360,b=q+.25+.5+35/360;return`M${p[0]+3*cs(a)} ${p[1]+3*sn(a)} L${p} L${p[0]+3*cs(b)} ${p[1]+3*sn(b)}`;})
 ]).flat(),'Constant rotation with attached arrowheads; Strong breathes the arc length while keeping the turn continuous.'),
 make('pinwheel','Pinwheel','Six swept paddles turn around a fixed hub.',[
  ...dot('hub',1,{pose:at(()=>[16,16])}),
  ...Array.from({length:6},(_,i)=>tube('paddle-'+i,(t,s)=>{const u=phase(t),q=i/6+u*(s?2:1),r=9.5+(s?.5*(1-cs(2*u)):0);return`M${point(3.7,q)} L${point(r,q+25/360)}`;})).flat()
 ],'Light has six steady swept paddles; Strong also extends the paddles twice per cycle.'),
 make('bounce-ball','Bounce ball','A ball compresses at a clear contact point.',[
  ...tube('floor','M9 25 L23 25'),
  stroke('ball',(t,s)=>{const[x,y,rx,ry]=ballShape(t,s);return ellipse(x,y,rx,ry);}),
  fill('ball-solid',(t,s)=>{const[x,y,rx,ry]=ballShape(t,s);return ellipse(x,y,rx+1.3,ry+1.3);},'solid'),
  fill('ball-tint',(t,s)=>{const[x,y,rx,ry]=ballShape(t,s);return ellipse(x,y,rx,ry);},'tint')
 ],'Two smooth bounces. Strong adds a larger jump and brief contact compression. The ball rim meets the floor rim at Bold without a loose cap.'),
 make('travel-wave','Travel wave','A continuous waveform moves across the frame.',tube('wave',(t,s)=>curve(x=>{const q=2*x-(s?3:2)*phase(t);return[5+22*x,16+(s?5:3)*sn(q)+(s?1.2*sn(2*q):0)];},8)),'Light moves one sine wave continuously rightward; Strong adds a sharper second harmonic without reversing its phase.')
];
