// Claude design plus equivalent native rigid transform composition.
import {make,F as P,fill,stroke,bothFill,framed,rect,circle,arc,hole,polygon,rotate,rigid,track,mix} from '../../library/ai-authoring.mjs';
import {flow as flowTrack} from '../../library/direction.mjs';

const S=Math.SQRT1_2;
// Cubic arc segments only, appended to an open contour that already sits at the arc start.
const A=(cx,cy,r,a,b)=>arc(cx,cy,r,a,b).replace(/^M\S+\s*/,'');
// Preview reset as a real reverse slide (0.28 s) instead of a fade for parts with stroke/fill joins.
const back=(from,to,fn)=>(t,s,r)=>r?track([[0,from],[.28,to,'io']],t):fn(t,s);
function build(name,label,intent,parts,light,strong,rearm,slide=[]){
 const keep=parts.map(p=>[p,p.name,p.d,p.pose]);
 const icon=make(name,label,intent,parts,light,strong,rearm);
 for(const[p,n,d,pose]of keep)if(slide.some(s=>n===s||n.startsWith(s+'-'))){p.d=d;p.pose=pose;p.visibility=()=>1;}
 return icon;
}

const agent=(()=>{
 const body=rect(7,12,18,14,4.5),slot=x=>hole(body,rect(9.75+x,17.75,6,2.5,1.25));
 const lensX=P([[0,0],[.45,6,'out']],[[0,0],[.1,-.8],[.28,6.6],[.4,6]]),lensAt=back(6,0,lensX);
 const antA=(t,s)=>s?flowTrack([[0,0],[.1,-14],[.22,5],[.32,0]],t):0;
 const tilt=(t,s)=>s?flowTrack([[0,0],[.08,0],[.3,2.5],[.44,-.8],[.56,0]],t):0;
 const world=rigid((t,s,r)=>({angle:tilt(r?0:t,s),pivot:[16,26]}));
 const ant=rigid((t,s,r)=>{const time=r?0:t,a=tilt(time,s),p=rotate(16,12,a,16,26);return{angle:a+antA(time,s),pivot:[16,12],dx:p[0]-16,dy:p[1]-12};});
 const lens=rigid((t,s,r)=>{const a=tilt(r?0:t,s),x=lensAt(t,s,r),radians=a*Math.PI/180;return{angle:a,pivot:[16,26],dx:x*Math.cos(radians),dy:x*Math.sin(radians)};});
 return build('ai-agent','AI agent','Lens scans right; Strong nods the antenna first and settles the body',[
  fill('body-interior',body,'tint',{pose:world}),stroke('body',body,{pose:world}),
  fill('body-solid',(t,s,r)=>slot(lensAt(t,s,r)),'solid',{pose:world}),
  stroke('lens','M11,19 L14.5,19',{pose:lens}),
  stroke('stalk','M16,12 L16,8.5',{pose:ant}),fill('stalk-solid',rect(15,7.5,2,5.5,1),'solid',{pose:ant}),
  ...bothFill('tip',circle(16,7,1.75),{pose:ant})
 ],.45,.9,['lens'],['lens','body-solid']);
})();

const team=(()=>{
 const peer=side=>{const X=x=>side>0?x:32-x,Q=(x,y)=>`${X(x)},${y}`,
  Ar=(cx,cy,r,a,b)=>A(X(cx),cy,r,side>0?a:180-a,side>0?b:180-b);
  return{outline:`M${Q(11.9,11.22)} ${Ar(9,12,3,-15,-90)} L${Q(6.5,9)} ${Ar(6.5,12,3,-90,-180)} L${Q(3.5,16)} ${Ar(6.5,16,3,180,114.6)}`,
   body:`M${Q(6.5,9)} L${Q(9,9)} ${Ar(9,12,3,-90,0)} L${Q(12,13.78)} ${Ar(12.5,18.5,4.75,-96.04,-180)} L${Q(7.75,19)} L${Q(6.5,19)} ${Ar(6.5,16,3,90,180)} L${Q(3.5,12)} ${Ar(6.5,12,3,180,270)} Z`};};
 const surface=(n,{outline,body})=>[fill(n+'-tint',body,'tint'),stroke(n,outline),fill(n+'-solid',body,'solid')];
 const ant=(n,x,y0,L,e)=>{const tip=y0-L-1.25;return[
  stroke(n+'-stalk',(t,s,r)=>`M${x},${y0} L${x},${y0-L-e(t,s,r)}`),
  fill(n+'-stalk-solid',(t,s,r)=>rect(x-1,tip+.5-e(t,s,r),2,L+2+e(t,s,r),1),'solid'),
  ...bothFill(n+'-tip',circle(x,tip,1.5),{pose:rigid((t,s,r)=>({dy:-e(t,s,r)}))})];};
 const eL=back(1,0,P([[0,0],[.25,1,'out']],[[0,0],[.14,0],[.3,1.2],[.42,1]]));
 const eR=back(1,0,P([[0,0],[.2,0],[.45,1,'out']],[[0,0],[.22,0],[.38,1.2],[.5,1]]));
 const eLead=(t,s)=>s?flowTrack([[0,0],[.14,1.1],[.5,-.2],[.62,0]],t):0;
 return build('agent-team','Agent team','Peers raise antennas in turn; Strong has the lead signal first',[
  ...surface('peerL',peer(1)),...surface('peerR',peer(-1)),
  ...ant('antL',7.75,9,2,eL),...ant('antR',24.25,9,2,eR),
  ...framed('lead',rect(9,15,14,12,3.5)),...ant('antLead',16,15,3,eLead)
 ],.5,.95,['antL','antR'],['antL','antR']);
})();

const tool=(()=>{
 const G=(p,q)=>`${21+S*(q-p)},${11+S*(p+q)}`,ang=(p,q)=>Math.atan2(p+q,q-p)*180/Math.PI;
 // Prong, throat arc, retraced handle spur, throat arc, prong: one path, one stroke, no doubled join.
 const wrench=q=>{const p=Math.sqrt(56.25-q*q);return`M${G(-3,-q)} L${G(p,-q)} ${A(21,11,7.5,ang(p,-q),135)} L${G(17.5,0)} L${G(7.5,0)} ${A(21,11,7.5,135,ang(p,q))} L${G(-3,q)}`;};
 const hex=polygon([0,1,2,3,4,5].map(i=>{const a=(15+60*i)*Math.PI/180;return[21+4.6188*Math.cos(a),11+4.6188*Math.sin(a)];}),1);
 const k=back(0,5,P([[0,5],[.4,0,'io']],[[0,5],[.12,6.2],[.4,0,'hit']]));
 const tq=(t,s)=>s?flowTrack([[0,0],[.36,0],[.5,7],[.64,-1.5],[.76,0]],t):0;
 const turn=rigid((t,s)=>({angle:tq(t,s),pivot:[21,11]}));
 const slide=rigid((t,s,r)=>{const v=k(t,s,r);return{dx:-S*v,dy:S*v,angle:r?0:tq(t,s),pivot:[21,11]};});
 return build('tool-call','Tool call','Wrench seats on the hex port; Strong pulls back, seats and torques',[
  fill('nut-tint',hex,'tint',{pose:turn}),
  stroke('wrench',wrench(5.25),{pose:slide}),
  {name:'wrench-solid',d:wrench(4.8),style:'solid',type:'stroke',width:2.6,pose:slide},
  stroke('nut',hex,{pose:turn}),fill('nut-solid',hole(hex,circle(21,11,2)),'solid',{pose:turn}),
  fill('bolt',circle(21,11,1.75),'outline',{pose:turn})
 ],.4,.85,['wrench'],['wrench']);
})();

const flow=(()=>{
 const node=(n,x,y)=>{const d=rect(x,y,9.5,9.5,3.25);return framed(n,d,hole(d,circle(x+4.75,y+4.75,3.25)));};
 const link=(n,d,solid)=>[stroke(n,d),{name:n+'-solid',d:solid,style:'solid',type:'stroke',width:2}];
 const bez=(u,a,b,c,d)=>{const v=1-u;return v*v*v*a+3*v*v*u*b+3*v*u*u*c+u*u*u*d;};
 const at=v=>v<1?[mix(8.25,14.25,v),16]:v<2?[bez(v-1,14.25,16.75,16,18.5),bez(v-1,16,16,8.25,8.25)]:[mix(18.5,23.5,v-2),8.25];
 const stage=P([[0,0],[.12,1,'linear'],[.45,2,'linear'],[.6,3,'out']],
  [[0,0],[.1,-.15],[.24,1,'pass'],[.36,2,'pass'],[.46,3.15],[.58,3]]);
 return build('agent-workflow','Agent workflow','Packet leaves the trunk and follows the upper branch into its node',[
  ...node('nodeA',3.5,11.25),...node('nodeB',18.75,3.5),...node('nodeC',18.75,19),
  ...link('linkB','M13.25,16 L14.25,16 C16.75,16 16,8.25 18.5,8.25','M12,16 L14.25,16 C16.75,16 16,8.25 18.5,8.25 L19.75,8.25'),
  ...link('linkC','M14.25,16 C16.75,16 16,23.75 18.5,23.75','M14.25,16 C16.75,16 16,23.75 18.5,23.75 L19.75,23.75'),
  ...bothFill('packet',circle(8.25,16,2.25),{pose:rigid((t,s)=>{const[x,y]=at(stage(t,s));return{dx:x-8.25,dy:y-16};})})
 ],.6,1,['packet']);
})();

const plan=(()=>{
 const bar=(n,x,y,w)=>{const d=typeof w==='function'?(t,s,r)=>rect(x,y,w(t,s,r),5,2.5):rect(x,y,w,5,2.5);return framed(n,d,d);};
 const w2=back(13,5,P([[0,5],[.5,13,'out']],[[0,5],[.26,13]]));
 const w3=back(13,5,P([[0,5],[.5,13,'out']],[[0,5],[.16,5],[.44,13]]));
 return build('task-plan','Task plan','Plan steps extend in order; Strong extends them one at a time',[
  ...bar('step1',6,6,11),...bar('step2',10,13.5,w2),...bar('step3',14,21,w3)
 ],.5,.85,['step2','step3'],['step2','step3']);
})();

export const icons=[agent,team,tool,flow,plan];
