// Claude Code proposal-remaining, revised and implemented by Codex after the
// compact authoring request timed out. See authoring-03/supervisor-review.md.
import {make,F as P,tube,line,framed,stroke,fill,bothFill,rigid,rect,circle,polygon,holes,arc,track,mix,rearmTime} from '../../library/ai-authoring.mjs';
import {flow} from '../../library/direction.mjs';
const out=[];
{
 const bubble=(x,y,right,o=0)=>polygon([[x-o,y-o],[x+16+o,y-o],[x+16+o,y+9+o],[x+(right?12:8),y+9+o],[x+(right?12:4),y+11.5+o],[x+(right?8:4),y+9+o],[x-o,y+9+o]],[2.5,2.5,2.5,.5,.65,.5,2.5]);
 const shell=(n,x,y,right)=>framed(n,bubble(x,y,right),holes(bubble(x,y,right,1.3),rect(x+1.3,y+1.3,13.4,6.4,1.2)));
 const H=[.4,1,.6];
 const bar=(side,i,x,y)=>tube(side+'-'+i,(t,s)=>{
  const h=H[i],start=side==='left'?0:.5,pk=[1.1,.35,1.1][i],tr=[.35,1.1,.35][i];
  const keys=s?[[0,h],[start,h],[start+.04,h*.7],[start+.11,pk],[start+.2,tr],[start+.28,pk*.85],[start+.36,tr*1.15],[start+.44,h]]:[[0,h],[.15,[1,.3,1][i],'out'],[.3,[.3,1,.5][i],'io'],[.5,h,'out']];
  const v=side==='right'&&!s?h:(s?flow(keys,t):track(keys,t));return `M${x} ${y-v} L${x} ${y+v}`;
 });
 out.push(make('voice-chat','Voice chat','Two fixed speech bubbles take turns speaking. Strong leaves a listening pause before the second voice replies.',[
  ...shell('left-shell',4,4,false),...shell('right-shell',12,17,true),
  ...[0,1,2].flatMap(i=>bar('left',i,8+i*4,8.5)),...[0,1,2].flatMap(i=>bar('right',i,16+i*4,21.5))],.5,1.05));
}
{
 const amp=P([[0,1.5],[.5,6,'out']],[[0,1.5],[.1,.8],[.5,6.9],[.62,5.6],[.72,6.15],[.82,6]]);
 const span=P([[0,1]],[[0,1],[.1,.3],[.48,1.06],[.6,.98],[.7,1]]);
 const wave=(t,s)=>{const a=amp(t,s),u=span(t,s),X=x=>10+(x-10)*u,Y=[16,16-a,16+a,16-a,16+a,16];let d='M10 16';const xs=[10,13.5,17,20.5,24,28];for(let i=1;i<6;i++){const mid=(xs[i-1]+xs[i])/2;d+=` C${X(mid)} ${Y[i-1]} ${X(mid)} ${Y[i]} ${X(xs[i])} ${Y[i]}`;}return d;};
 out.push(make('audio-generate','Audio generate','A source node generates a waveform. Strong gathers the signal, extends it, and settles the last response.',[...framed('source',circle(7,16,3)),...tube('wave',wave)],.55,.95,['wave']));
}
{
 const X=[5.5,9,12.5,16,19.5,23,26.5],dirty=[2,4,2,4,2.5,4,2],clean=[1.5,2,2.5,3,2.5,2,1.5];
 const close=P([[0,0],[.5,1,'out']],[[0,0],[.1,0],[.34,1]]);
 const atX=x=>{let a=0,b=1;for(let i=0;i<30;i++){const u=(a+b)/2,v=1-u,xx=4*v*v*v+30*v*v*u+66*v*u*u+28*u*u*u;if(xx<x)a=u;else b=u;}return(a+b)/2;};
 const bars=X.flatMap((x,i)=>tube('bar-'+i,(t,s)=>{const q=s?flow([[0,0],[.1+.04*i,0],[.14+.04*i,-.08],[.28+.04*i,1.1],[.36+.04*i,.96],[.44+.04*i,1]],t):track([[0,0],[.45,1,'out']],t),u=atX(x),ceiling=9-12*u*(1-u)+3*close(t,s);const h=Math.min(mix(dirty[i],clean[i],q),16-ceiling-3.8);return `M${x} ${16-h} L${x} ${16+h}`;}));
 const envelope=(bottom)=>(t,s)=>{const k=3*close(t,s),y=bottom?23-k:9+k,c=bottom?27-k:5+k;return `M4 ${y} C10 ${c} 22 ${c} 28 ${y}`;};
 out.push(Object.assign(make('audio-clean','Audio clean','A filter envelope closes around uneven audio and levels its peaks in a left-to-right pass.',[...tube('upper',envelope(false)),...tube('lower',envelope(true)),...bars],.55,.75,['upper','lower','bar']),{duotoneFill:false}));
}
{
 const flag=P([[0,0],[.05,0],[.4,1,'out']],[[0,0],[.14,0],[.32,1]]);
 const stem=P([[0,0],[.42,0],[.64,1,'out']],[[0,0],[.34,0],[.46,1]]);
 const head=P([[0,0],[.65,0],[.85,1,'out']],[[0,0],[.46,0],[.58,1]]);
 const dy1=P([[0,0]],[[0,0],[.08,.5],[.18,-.3],[.26,.1],[.32,0],[.58,0],[.64,-.8],[.72,.35],[.8,-.12],[.86,0]]);
 const dy2=P([[0,0]],[[0,0],[.64,0],[.7,-.8],[.78,.35],[.86,-.12],[.92,0]]);
 const pose1=rigid((t,s)=>({dy:dy1(t,s)}));
 const flagD=(t,s)=>{const g=flag(t,s),a=dy1(t,s),b=dy2(t,s);return `M12.5 ${8+a} C16.5 ${mix(10,7.33,g)+mix(a,b,.33)} ${mix(18,20.5,g)} ${mix(14,6.67,g)+mix(a,b,.67)} ${mix(16,24.5,g)} ${mix(18,6,g)+b}`;};
 const second=(t,s)=>{const y=mix(6,20,stem(t,s))+dy2(t,s),q=head(t,s);let d=`M24.5 ${6+dy2(t,s)} L24.5 ${y}`;for(let i=0;i<4;i++){const a=i*90*q,b=(i+1)*90*q;d+=' '+arc(21,y,3.5,a,b).replace(/^M[^C]*/,'');}return d;};
 const visible=(t,s)=>t>=(s?.34:.42)?1:0;
 const i=make('music-generate','Music generate','One note becomes a beamed pair. Strong adds a short pickup, then each attached note answers the rhythm.',[
  ...framed('head-1',circle(9,23,3.5),undefined,{pose:pose1}),...line('stem-1',[12.5,23],[12.5,8],{pose:pose1}),
  ...tube('flag',flagD),...tube('note-2',second,{visibility:visible}),
  fill('head-2-interior',(t,s)=>circle(21,20+dy2(t,s),3.5),'tint',{visibility:(t,s)=>head(t,s)===1?1:0}),
  fill('head-2-solid',(t,s)=>circle(21,20+dy2(t,s),3.5),'solid',{visibility:(t,s)=>head(t,s)===1?1:0})],.9,.95,['flag','note-2','head-2']);
 // Solid head exits before the joined stem/rim fades. The retained original
 // flag returns only after the new note has gone.
 for(const p of i.parts.filter(p=>p.name.includes('head-2'))){const v=p.visibility;p.visibility=(t,s,r)=>r?track([[0,1],[.04,0,'out']],t):v(t,s,r);}
 out.push(i);
}
{
 const angle=P([[0,0],[.2,-8,'out'],[.25,-8],[.4,0,'in']],[[0,0],[.18,-13],[.28,-11],[.43,0,'hit']]);
 const dip=P([[0,0],[.4,0],[.45,.2,'out'],[.55,0,'io']],[[0,0],[.43,0],[.49,.45],[.58,-.15],[.69,0]]);
 const board=rect(5,16,22,12,2.5),slate=rect(5,10,22,6,1),play=polygon([[14,19.5],[20,22],[14,24.5]],.7);
 const ride=rigid((t,s)=>({dy:dip(t,s)})),hinge=rigid((t,s)=>({angle:angle(t,s),pivot:[5,16],dy:dip(t,s)}));
 const stripe=(x)=>polygon([[x-.7,11.3],[x+1.3,11.3],[x-.7,14.7],[x-2.7,14.7]],.3);
 out.push(make('video-generate','Video generate','A hinged clapper opens and closes. The slate and board receive the impact together, then return to still.',[
  ...framed('board',board,holes(board,polygon([[12.7,18.2],[22,22],[12.7,25.8]],.7)),{pose:ride}),stroke('play',play,{pose:ride}),
  ...framed('slate',slate,holes(slate,stripe(12),stripe(21)),{pose:hinge}),stroke('stripe-a','M12 10 L8.5 16',{pose:hinge}),stroke('stripe-b','M21 10 L17.5 16',{pose:hinge})],.6,.95));
}
export const icons=out;
