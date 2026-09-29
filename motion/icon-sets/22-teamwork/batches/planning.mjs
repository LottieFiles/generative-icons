// teamwork batch 2 revised: planning icons. Claude draft, unverified geometry.
import {make,stroke,fill,tube,bothFill,framed,circle,rect,arc,rigid,curve,geometricRearm} from './helpers.mjs';
const D=(l,s)=>(t,st)=>curve(t,st,l,s),cl=x=>Math.max(0,Math.min(1,x));
const smooth=(p,k=1/6)=>p.map((q,i)=>{if(!i)return'';const a=p[i-1],b=p[i-2]||a,c=p[i+1]||q;return` C${a[0]+(q[0]-b[0])*k} ${a[1]+(q[1]-b[1])*k} ${q[0]-(c[0]-a[0])*k} ${q[1]-(c[1]-a[1])*k} ${q[0]} ${q[1]}`;}).join('');
// Constant-topology reveal: points past the cut collapse onto it.
function trimPoly(pts,f){const seg=pts.slice(1).map((p,i)=>Math.hypot(p[0]-pts[i][0],p[1]-pts[i][1]));let left=cl(f)*seg.reduce((a,b)=>a+b,0),cur=pts[0];const out=[cur];for(let i=0;i<seg.length;i++){const p=pts[i+1];if(left>=seg[i]){cur=p;left-=seg[i];}else{const u=seg[i]?left/seg[i]:0;cur=[cur[0]+(p[0]-cur[0])*u,cur[1]+(p[1]-cur[1])*u];left=0;}out.push(cur);}return'M'+out.map(p=>p.join(' ')).join(' L');}

function kanban(){
 const parts=[];
 [4.5,13.5,22.5].forEach((x,i)=>parts.push(...tube('column-'+(i+1),`M${x} 7 L${x+5} 7`)));
 const card=(n,x,y)=>bothFill('card-'+n,rect(x,y,6,4,1.5));
 parts.push(...card('todo-a',4,11),...card('doing-a',13,11),...card('done-a',22,11),...card('done-b',22,18));
 const dx=D([[0,0],[.1,0],[.7,9]],[[0,0],[.14,0,'hold'],[.52,9.7],[.72,9]]);
 const dy=D([[0,0],[.22,-.6],[.7,0]],[[0,0],[.16,-1.5],[.42,-1.5,'hold'],[.66,0]]);
 const mover=card('move',4,18);
 for(const p of mover)p.pose=rigid((t,s)=>({dx:dx(t,s),dy:dy(t,s)}));
 parts.push(...mover);
 return{...make('kanban-board','Kanban board','A card is dragged from the first column into the next. Strong lifts it higher, slides faster with one small overshoot and drops it in place.',parts,.7,.72,['card-move']),duotoneFill:false};
}

function sprint(){
 const R=9.5,cx=16,cy=16,parts=[],rad=a=>a*Math.PI/180;
 [0,120,240].forEach((o,i)=>{
  const s=-150+o,e=s+80,a=rad(e),P=[cx+R*Math.cos(a),cy+R*Math.sin(a)],T=[-Math.sin(a),Math.cos(a)],N=[Math.cos(a),Math.sin(a)];
  const Q=[P[0]+2*T[0],P[1]+2*T[1]],b=k=>[Q[0]-1.8*T[0]+k*2.4*N[0],Q[1]-1.8*T[1]+k*2.4*N[1]];
  // Straight 2-unit approach after the arc; head vertex sits on the lap end cap: intentional opaque join.
  parts.push(...tube('lap-'+(i+1),arc(cx,cy,R,s,e)+` L${Q}`),...tube('head-'+(i+1),`M${b(-1)} L${Q} L${b(1)}`));
 });
 const ang=D([[0,0],[.9,120]],[[0,0],[.1,-14],[.64,360]]);
 for(const p of parts)p.pose=rigid((t,s)=>({angle:ang(t,s),pivot:[16,16]}));
 parts.push(...bothFill('goal',circle(16,16,2.2)));
 return{...make('sprint-cycle','Sprint cycle','Three iteration laps index around the sprint goal. Light advances one quiet 120° step to the next equivalent pose; Strong winds back and drives one full decisive turn.',parts,.9,.8),duotoneFill:false,cycleDegrees:120};
}

function roadmap(){
 // Rounded stepped route sampled as a dense polyline: two r=3 quarter corners.
 const pts=[[6,24],[10,24]],arcP=(x,y,a,b)=>{for(let i=1;i<=8;i++){const g=rad(a+(b-a)*i/8);pts.push([x+3*Math.cos(g),y+3*Math.sin(g)]);}},rad=a=>a*Math.PI/180;
 arcP(10,21,90,0);pts.push([13,11]);arcP(16,11,180,270);pts.push([26,8]);
 const f=D([[0,0],[.05,0],[1,1.07]],[[0,0],[.06,0,'hold'],[.55,1.07]]);
 // Nodes grow from the route fraction that reaches them, so they never lead the stroke.
 const nodeR=(fn,t,s)=>2.2*cl((f(t,s)-fn)/.07);
 const node=(n,x,y,r)=>{const v=(t,s)=>r(t,s)>.05?1:0;return[fill(n,(t,s)=>circle(x,y,r(t,s)),'outline',{visibility:v}),fill(n+'-solid',(t,s)=>circle(x,y,r(t,s)),'solid',{visibility:v})];};
 // Dots sit on the route centerline: intentional opaque joins.
 const parts=[...tube('route',(t,s)=>trimPoly(pts,f(t,s)),{visibility:(t,s)=>f(t,s)>0?1:0}),...node('node-1',6,24,()=>2.2),...node('node-2',13,16,(t,s)=>nodeR(.41,t,s)),...node('node-3',26,8,(t,s)=>nodeR(1,t,s))];
 return geometricRearm({...make('team-roadmap','Team roadmap','A rounded route draws upward from the first stop; each next stop appears as the route reaches it. Strong draws the whole route in one decisive stroke.',parts,1.02,.58,['route','node-2','node-3']),duotoneFill:false});
}

function milestone(){
 const amp=D([[0,0],[.22,.7],[.9,0]],[[0,0],[.18,1.8],[.5,1.4],[1.1,0]]),ph=(t,s)=>t*Math.PI*2*(s?3:2.2);
 const flag=(t,s)=>{const a=amp(t,s),p=ph(t,s),w=x=>a*Math.pow((x-9)/15,1.3)*Math.sin(Math.PI*2*(x-9)/12-p);
  const xs=[9,11.5,14,16.5,19,21.5,24],top=xs.map(x=>[x,7+w(x)]),bot=xs.map(x=>[x,15+w(x)]).reverse();
  return'M9 7'+smooth(top)+` L24 ${15+w(24)}`+smooth(bot)+' Z';};
 // Flag left edge on the pole centerline; base crosses the pole foot: intentional joins.
 const parts=[...tube('pole','M9 27 L9 5'),...tube('base','M5 27 L13 27'),...framed('flag',flag)];
 return{...make('milestone','Milestone','A planted flag marks the milestone. A travelling wave runs out along the banner and settles; Strong flaps with a larger, faster wave.',parts,.9,1.1),duotoneFill:true};
}

function workload(){
 const parts=[];
 [8,16,24].forEach((y,i)=>parts.push(stroke('member-'+(i+1),circle(7,y,2.2)),fill('member-'+(i+1)+'-solid',circle(7,y,3.1),'solid')));
 const l1=D([[0,16],[.1,16],[.75,12]],[[0,16],[.06,16,'hold'],[.3,12]]),l3=D([[0,5],[.1,5],[.75,10]],[[0,5],[.18,5,'hold'],[.46,10]]);
 const bar=(n,y,len)=>bothFill('bar-'+n,(t,s)=>rect(12,y-1.5,len(t,s),3,1.5));
 parts.push(...bar('1',8,l1),...bar('2',16,()=>10),...bar('3',24,l3));
 return{...make('team-workload','Team workload','Load moves off the overloaded member onto the lightest. Strong releases the top bar in a fast decisive drop and the bottom bar receives it quickly; values stay monotone.',parts,.75,.5,['bar-1','bar-3']),duotoneFill:false};
}

export const icons=[kanban(),sprint(),roadmap(),milestone(),workload()];
