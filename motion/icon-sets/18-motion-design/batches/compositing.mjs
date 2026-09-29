import{make,tube,stroke,fill,framed,rect,circle,polygon,rigid,curve,hole}from'./helpers.mjs';
const onionF=(t,s)=>curve(t,s,[[0,.76],[.04,.76],[.12,.9,'linear'],[.54,1,'out']],[[0,.76],[.04,.76],[.36,1]]);
const onionA=(t,s)=>curve(t,s,[[0,0],[.12,0],[.36,.45,'out']],[[0,0],[.04,0],[.22,.45]]);
const onion=make('onion-skin','Onion skin','Neighboring poses spread from the current frame to show motion over time. Strong exposes a larger sweep earlier; the current pose stays fixed.',[
 ...framed('current',circle(16,10.5,3.5),circle(16,10.5,4.8)),...[-1,1].flatMap((sign,i)=>[stroke('ghost-'+i,circle(16,10.5,3.5),{pose:rigid((t,s)=>({dx:sign*9*onionF(t,s),dy:11*onionF(t,s)})),visibility:onionA}),fill('ghost-'+i+'-solid',circle(16,10.5,4.8),'solid',{pose:rigid((t,s)=>({dx:sign*9*onionF(t,s),dy:11*onionF(t,s)})),visibility:onionA})])
],.54,.36,['ghost']);onion.duotoneFill=true;
const circle8=()=>{const a=[],r=5,k=4/3*Math.tan(Math.PI/16);for(let j=0;j<8;j++){const t=(-112.5+45*j)*Math.PI/180,n=t+Math.PI/4,p=[16+r*Math.cos(t),10+r*Math.sin(t)],q=[16+r*Math.cos(n),10+r*Math.sin(n)];if(!j)a.push(...p);a.push(p[0]-k*r*Math.sin(t),p[1]+k*r*Math.cos(t),q[0]+k*r*Math.sin(n),q[1]-k*r*Math.cos(n),...q);}return a;};
const rect8=()=>{const p=[[12.9,4.5],[19.1,4.5],[21.5,6.9],[21.5,13.1],[19.1,15.5],[12.9,15.5],[10.5,13.1],[10.5,6.9]],k=.55228475*2.4,a=[...p[0]];for(let j=0;j<8;j++){const x=p[j],y=p[(j+1)%8];if(j%2===0)a.push(x[0]+(y[0]-x[0])/3,x[1]+(y[1]-x[1])/3,x[0]+2*(y[0]-x[0])/3,x[1]+2*(y[1]-x[1])/3,...y);else{const dirs=[[1,0,0,-1],[0,1,1,0],[-1,0,0,1],[0,-1,-1,0]][(j-1)/2];a.push(x[0]+k*dirs[0],x[1]+k*dirs[1],y[0]+k*dirs[2],y[1]+k*dirs[3],...y);}}return a;};
const ca=circle8(),cb=rect8();
const shape=(t,s)=>{const p=curve(t,s,[[0,0],[.04,0],[.62,1,'out']],[[0,0],[.06,0],[.42,1]]),v=ca.map((a,i)=>a+(cb[i]-a)*p);return`M${v.slice(0,2).join(' ')} `+Array.from({length:8},(_,i)=>'C'+v.slice(2+i*6,8+i*6).join(' ')).join(' ')+' Z';};
const morph=make('shape-morph','Shape morph','A circle becomes a rounded square through matching path points. Strong primes the shape with a short turn, then completes the contour change faster.',[
 ...framed('source',circle(8,23,3)),...framed('target',rect(21,20,6,6,1.6)),...framed('hero',shape,undefined,{pose:rigid((t,s)=>({angle:s?curve(t,true,[],[[0,0],[.02,0],[.09,-12],[.2,0]]):0,pivot:[16,10]}))})
],.62,.42,['hero']);morph.duotoneFill=true;
const maskV=(t,s)=>{const p=curve(t,s,[[0,0],[.04,0],[.34,1,'smooth'],[.42,1,'hold'],[.78,0,'out']],[[0,0],[.04,0],[.1,-.12],[.3,1.06,'pass'],[.4,1],[.72,0]]);return[20+p*(s?3.5:2.5),15.5+p*(s?1.1:.7)];};
const maskPoints=(t,s)=>[[11,14.5],[16.5,11],maskV(t,s),[15,20.5]];
const maskD=(t,s)=>{const p=maskPoints(t,s),tan=p.map((v,i)=>p[(i+1)%4].map((x,j)=>(x-p[(i+3)%4][j])*.25));return`M${p[0].join(' ')} `+p.map((v,i)=>{const j=(i+1)%4;return'C'+[...v.map((x,k)=>x+tan[i][k]),...p[j].map((x,k)=>x-tan[j][k]),...p[j]].join(' ');}).join(' ')+' Z';};
const mask=make('mask-path','Mask path','One control point pulls the mask boundary and then releases it. Strong pulls the actual curve farther, with a brief preparation and a single elastic settle.',[
 fill('surface',rect(4,6.5,24,19,2.5),'tint'),stroke('frame',rect(4,6.5,24,19,2.5)),stroke('mask',maskD),fill('surface-solid',(t,s)=>hole(rect(4,6.5,24,19,2.5),maskD(t,s))),...[0,1,2,3].flatMap(i=>{const d=rect(maskPoints(0,false)[i][0]-1.2,maskPoints(0,false)[i][1]-1.2,2.4,2.4,.3),pose=rigid((t,s)=>{const a=maskPoints(t,s)[i],b=maskPoints(0,false)[i];return{dx:a[0]-b[0],dy:a[1]-b[1]};});return[fill('node-'+i,d,'outline',{pose}),fill('node-'+i+'-solid',d,'solid',{pose})];})
],.78,.72);mask.duotoneFill=true;
const matteX=(t,s)=>curve(t,s,[[0,14],[.04,14],[.44,16,'smooth'],[.56,16,'hold'],[.98,14,'out']],[[0,14],[.04,14],[.12,13.4],[.4,18.3,'pass'],[.5,18],[.88,14]]);
const h=(x,t,s)=>Math.sqrt(Math.max(0,5.9**2-(x-matteX(t,s))**2));
const matte=make('track-matte','Track matte','A matte window moves across source columns to reveal the clipped result. Strong sweeps across twice as much content and returns with a clear brake.',[
 fill('matte-area',circle(14,16,9.5),'tint',{pose:rigid((t,s)=>({dx:matteX(t,s)-14}))}),...tube('matte-boundary',circle(14,16,9.5),{pose:rigid((t,s)=>({dx:matteX(t,s)-14}))}),...[9,12.6,16.2,19.8,23.4].flatMap((x,i)=>[...tube('content-'+i,`M${x} 8 L${x} 24`,{visibility:()=>.28}),...tube('result-'+i,(t,s)=>{const a=h(x,t,s);return`M${x} ${16-a} L${x} ${16+a}`;},{visibility:(t,s)=>Math.max(0,Math.min(1,(h(x,t,s)-.6)/.6))})])
],.98,.88);matte.duotoneFill=true;
export const compositing={'onion-skin':onion,'shape-morph':morph,'mask-path':mask,'track-matte':matte};
