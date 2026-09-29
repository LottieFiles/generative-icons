// Claude Code design; Codex integration and verification.
import {make,F as P,stroke,fill,tube,line,framed,rigid,rect,circle,polygon,arc,capsule,holes,track,clamp,mix} from '../../library/ai-authoring.mjs';
import {flow} from '../../library/direction.mjs';
const seg=(nm,pt,x={})=>[stroke(nm,(t,s,r)=>{const[a,b]=pt(t,s,r);return`M${a} L${b}`;},x),fill(nm+'-solid',(t,s,r)=>capsule(...pt(t,s,r),2.6),'solid',x)];
const count=(t,times)=>times.filter(c=>t>=c-1e-6).length;
const grow=(t,times,u)=>times.reduce((a,c)=>a+u*track([[c,0],[c+.05,1,'out']],t),0);

const styleTransfer=(()=>{
 const rodX=P([[0,0],[.5,15.5,'io']],[[0,0],[.08,-.5],[.34,15.8],[.44,15.4],[.52,15.55],[.6,15.5]]);
 const wob=P([[0,0]],[[0,0],[.34,0],[.42,1],[.52,-.5],[.6,.2],[.68,0]]);
 const X=[11.5,14.5,17,19.5,21.5],photo=[21,12,18,13.5,21],paint=[19,15.5,18.5,15,17],dir=[1,-1,1,-1,1];
 const scene=(t,s)=>{const rx=8.5+rodX(t,s),w=wob(t,s);
  const p=X.map((x,i)=>{const k=clamp((rx-x+2)/2.5);return[x,mix(photo[i],paint[i],k)+w*k*dir[i],k];});
  let d=`M${p[0][0]} ${p[0][1]}`;
  for(let i=1;i<5;i++){const a=p[i-1],b=p[i],k=(a[2]+b[2])/2,mx=(a[0]+b[0])/2,dx=b[0]-a[0],dy=b[1]-a[1];
   d+=` C${mix(a[0]+dx/3,mx,k)} ${mix(a[1]+dy/3,a[1],k)} ${mix(b[0]-dx/3,mx,k)} ${mix(b[1]-dy/3,b[1],k)} ${b[0]} ${b[1]}`;}
  return d;};
 const box=rect(4,4,24,24,3);
 return make('style-transfer','Style transfer','A wipe rod slides across a framed photo; each ridge it crosses softens into painted strokes and stays painted.',
  [...framed('canvas',box,holes(box,rect(6.4,6.4,19.2,19.2,1.4))),...tube('scene',scene),
   ...line('rod',[8.5,9],[8.5,23],{pose:rigid((t,s)=>({dx:rodX(t,s)}))})],.55,.7,['rod','scene']);
})();

const smartCrop=(()=>{
 const tl=P([[0,0],[.4,1,'out']],[[0,0],[.06,-.08],[.26,1.14],[.36,.95],[.44,1]]);
 const br=P([[0,0],[.4,1,'out']],[[0,0],[.1,0],[.14,-.08],[.32,1.14],[.42,.95],[.5,1]]);
 const tr=P([[0,0],[.42,0],[.6,1,'out']],[[0,0],[.42,0],[.52,1]]);
 const bl=P([[0,0],[.42,0],[.6,1,'out']],[[0,0],[.5,0],[.6,1]]);
 const bend=(a,c,b,u)=>{const la=Math.hypot(c[0]-a[0],c[1]-a[1]),lb=Math.hypot(b[0]-c[0],b[1]-c[1]),q=u*(la+lb);
  const m=q<la?[mix(a[0],c[0],q/la),mix(a[1],c[1],q/la)]:c,e=q<la?m:[mix(c[0],b[0],(q-la)/lb),mix(c[1],b[1],(q-la)/lb)];
  return`M${a} L${m} L${e}`;};
 const draw=(nm,a,c,b,fn)=>tube(nm,(t,s)=>bend(a,c,b,fn(t,s)),{visibility:(t,s)=>fn(t,s)>.02?1:0});
 return make('smart-crop','Smart crop','Two crop corners slide in around the subject, then the missing corners draw to close the frame.',
  [...tube('hill','M10 21 C12 15.5 16 15.5 18 21'),...framed('sun',circle(20,13,2),circle(20,13,3.3)),
   ...tube('tl','M4 13 L4 4 L13 4',{pose:rigid((t,s)=>({dx:2*tl(t,s),dy:3*tl(t,s)}))}),
   ...tube('br','M19 28 L28 28 L28 19',{pose:rigid((t,s)=>({dx:-1.5*br(t,s),dy:-3*br(t,s)}))}),
   ...draw('tr',[21,7],[26.5,7],[26.5,12.5],tr),...draw('bl',[6,19.5],[6,25],[11.5,25],bl)],.6,.7,['tl','br','tr','bl']);
})();

const imageCaption=(()=>{
 const L1=[[.06,.12,.18,.24,.3,.35],[.34,.37,.4,.43,.47,.5]],L2=[[.4,.45,.5,.55,.6],[.6,.63,.67,.71,.75]],wrap=[[.35,.4],[.52,.6]];
 const e1=(t,s)=>11+2*count(t,L1[+s]),e2=(t,s)=>5+2.2*count(t,L2[+s]);
 const cur=(t,s)=>{const w=wrap[+s],two=t>=(w[0]+w[1])/2,x=(two?e2(t,s):e1(t,s))+3.75,y=two?26.5:22.5;return[[x,y-1.5],[x,y+1.5]];};
 const blink=(t,s)=>{const w=wrap[+s];return t>=w[0]&&t<w[1]?0:1;};
 const box=rect(4,4,24,14,3);
 return make('image-caption','Image caption','The picture is read, then a two-line caption is typed beneath it and stays written.',
  [...framed('photo',box,holes(box,polygon([[9,14.5],[12,10],[14.5,12.5],[17,10.5],[20,14.5]],1),circle(23,9,2.2))),
   stroke('ridge','M9 14.5 L12 10 L14.5 12.5 L17 10.5 L20 14.5'),stroke('sun',circle(23,9,1.5)),
   stroke('scan',(t,s)=>{const y=s?flow([[0,7.5],[.04,7.1],[.3,14.5]],t):track([[0,7.5],[.3,14.5,'linear']],t);return`M7.5 ${y} L24.5 ${y}`;},
    {visibility:(t,s)=>s?flow([[0,0],[.04,.6],[.26,.6],[.32,0]],t):0}),
   ...seg('l1',(t,s)=>[[5,22.5],[e1(t,s),22.5]]),
   ...seg('l2',(t,s)=>[[5,26.5],[e2(t,s),26.5]],{visibility:(t,s)=>e2(t,s)>5?1:0}),
   ...seg('cursor',cur,{visibility:blink})],.6,1,['l1','l2','cursor']);
})();

const textToSpeech=(()=>{
 const a1=P([[0,1]],[[0,1],[.1,.35],[.24,1.12],[.32,.94],[.4,1]]);
 const a2=P([[0,0],[.3,1,'out']],[[0,0],[.12,0],[.34,1.12],[.44,.94],[.54,1]]);
 const a3=P([[0,0],[.15,0],[.46,1,'out']],[[0,0],[.22,0],[.46,1.12],[.58,.94],[.69,1]]);
 const ring=(nm,r,fn)=>tube(nm,(t,s)=>arc(17,15.5,r,-40*fn(t,s),40*fn(t,s)),{visibility:(t,s)=>fn(t,s)>.02?1:0});
 const solidA=holes(polygon([[3.6,25],[11,4.5],[18.4,25],[15.9,25],[14.1,19.8],[7.9,19.8],[6.1,25]],[1.3,1.6,1.3,.6,.6,.6,.6]),
  polygon([[8.8,17.2],[11,10.9],[13.2,17.2]],.6));
 return make('text-to-speech','Text to speech','A rounded letter A speaks: sound arcs grow outward from its side and remain.',
  [fill('counter',polygon([[6.9,18.5],[11,7],[15.1,18.5]],.8),'tint'),
   stroke('glyph','M5 24 L10.6 8.3 C10.85 7.6 11.15 7.6 11.4 8.3 L17 24'),stroke('bar','M7.5 18.5 L14.5 18.5'),
   fill('glyph-solid',solidA,'solid'),...ring('a1',4.5,a1),...ring('a2',8,a2),...ring('a3',11.5,a3)],.5,.9,['a2','a3']);
})();

const speechToText=(()=>{
 const X=[5,8.5,12,15.5,19,22.5,26],H=[1.5,3.5,4.5,2.5,4,2,3];
 const half=(i,t,s)=>s?flow([[0,H[i]],[.08,H[i]+.5],[.15+.06*i,H[i]+1.6],[.24+.06*i,.7],[.3+.06*i,1.15],[.35+.06*i,1]],t)
  :track([[0,H[i]],[.05*i,H[i]],[.05*i+.15,1,'io']],t);
 const bars=X.flatMap((x,i)=>seg('bar-'+i,(t,s)=>[[x,10-half(i,t,s)],[x,10+half(i,t,s)]]));
 const W1=[[.02,.12,.22,.32],[.2,.26,.32,.38]],W2=[[.42,.5,.58],[.46,.52,.58]];
 return Object.assign(make('speech-to-text','Speech to text','Voice bars settle into pills as their words are written out in chunks below and stay.',
  [...bars,...seg('l1',(t,s)=>[[5,21],[11+grow(t,W1[+s],4),21]]),
   ...seg('l2',(t,s)=>[[5,26],[5+grow(t,W2[+s],4),26]],{visibility:(t,s)=>grow(t,W2[+s],4)>.2?1:0})],
  .65,.8,['bar','l1','l2']),{duotoneFill:false});
})();

export const icons=[styleTransfer,smartCrop,imageCaption,textToSpeech,speechToText];
