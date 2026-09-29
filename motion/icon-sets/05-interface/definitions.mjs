import { directInterface } from '../library/directed-interface.mjs';
import svgpath from 'svgpath';

export const mix=(a,b,p)=>a+(b-a)*p;
// Each interval is a deliberate eased move or a still hold. Sample once at 60 fps.
export function phrase(keys,t){if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i];return mix(a[1],b[1],(1-Math.cos(Math.PI*(t-a[0])/(b[0]-a[0])))/2);}
export function rotate(x,y,a,cx=16,cy=16){const r=a*Math.PI/180,c=Math.cos(r),s=Math.sin(r);return[cx+(x-cx)*c-(y-cy)*s,cy+(x-cx)*s+(y-cy)*c];}
export const circle=(x,y,r)=>`M${x} ${y-r} C${x+r*.552285} ${y-r} ${x+r} ${y-r*.552285} ${x+r} ${y} C${x+r} ${y+r*.552285} ${x+r*.552285} ${y+r} ${x} ${y+r} C${x-r*.552285} ${y+r} ${x-r} ${y+r*.552285} ${x-r} ${y} C${x-r} ${y-r*.552285} ${x-r*.552285} ${y-r} ${x} ${y-r} Z`;
export const rect=(x,y,w,h,r)=>`M${x+r} ${y} L${x+w-r} ${y} C${x+w-r*.448} ${y} ${x+w} ${y+r*.448} ${x+w} ${y+r} L${x+w} ${y+h-r} C${x+w} ${y+h-r*.448} ${x+w-r*.448} ${y+h} ${x+w-r} ${y+h} L${x+r} ${y+h} C${x+r*.448} ${y+h} ${x} ${y+h-r*.448} ${x} ${y+h-r} L${x} ${y+r} C${x} ${y+r*.448} ${x+r*.448} ${y} ${x+r} ${y} Z`;
export function polygon(vertices,r){const corners=vertices.map((v,i)=>{const before=vertices[(i+vertices.length-1)%vertices.length],after=vertices[(i+1)%vertices.length];const radius=Array.isArray(r)?r[i%r.length]:r;const step=p=>{const d=Math.hypot(p[0]-v[0],p[1]-v[1]),a=Math.min(radius,d*.35);return[v[0]+(p[0]-v[0])*a/d,v[1]+(p[1]-v[1])*a/d];};return {v,a:step(before),b:step(after)};});return corners.map(({v,a,b},i)=>`${i?'L':'M'}${a} C${mix(a[0],v[0],2/3)} ${mix(a[1],v[1],2/3)} ${mix(b[0],v[0],2/3)} ${mix(b[1],v[1],2/3)} ${b}`).join(' ')+' Z';}
export function capsule(a,b,width){const len=Math.hypot(b[0]-a[0],b[1]-a[1]);return svgpath(rect(-width/2,-width/2,len+width,width,width/2)).rotate(Math.atan2(b[1]-a[1],b[0]-a[0])*180/Math.PI).translate(...a).toString();}
// Reverse a closed contour without reflecting its geometry. One final Close
// and a retraced zero-width seam preserve transparent holes in this exporter.
export function reverseContour(d){const segments=svgpath(d).abs().unarc().unshort().segments;let first,last;const edges=[];for(const [cmd,...v]of segments){if(cmd==='M'){first=v;last=v;}else if(cmd==='L'||cmd==='C'){edges.push({cmd,start:last,v});last=v.slice(-2);}else if(cmd==='Z'){if(last[0]!==first[0]||last[1]!==first[1])edges.push({cmd:'L',start:last,v:first});last=first;}else throw Error('Unsupported hole command '+cmd);}return 'M'+first+' '+edges.reverse().map(e=>e.cmd==='L'?'L'+e.start:'C'+[...e.v.slice(2,4),...e.v.slice(0,2),...e.start]).join(' ')+' Z';}
function winding(d){const p=svgpath(d).abs().unarc().unshort().segments.filter(s=>s[0]!=='Z').map(s=>s.slice(-2));return Math.sign(p.reduce((sum,a,i)=>{const b=p[(i+1)%p.length];return sum+a[0]*b[1]-b[0]*a[1];},0))||1;}
export const hole=(outer,inner)=>outer.replace(/Z\s*$/,'')+' L'+svgpath(outer).abs().segments[0].slice(1)+' '+(winding(outer)===winding(inner)?reverseContour(inner):inner).replace(/^M/,'L');
const fill=(name,d,style='solid',extra={})=>({name,d,style,...extra});
const stroke=(name,d,extra={})=>({name,d,style:'outline',type:'stroke',width:2,...extra});
const line=(name,a,b,extra={})=>[stroke(name,`M${a} L${b}`,extra),fill(name+'-solid',capsule(a,b,2.6),'solid',extra)];
const rigid=(fn)=>Object.assign((x,y,t,s,r)=>{const {dx=0,dy=0,angle=0,pivot=[16,16]}=fn(t,s,r);const p=rotate(x,y,angle,...pivot);return[p[0]+dx,p[1]+dy];},{transform:fn});
const gesture=(t,s,light,strong)=>phrase(s?strong:light,t);

const pulse=(t,s,delay=0)=>gesture(t-delay,s,[[0,0],[.16,1],[.21,1],[.48,0]],[[0,0],[.24,1],[.32,1],[.76,0]]);
const bothFill=(name,d,extra={})=>[fill(name,d,'outline',extra),fill(name+'-solid',d,'solid',extra)];
const tube=(name,d,extra={})=>[stroke(name,d,extra),{name:name+'-solid',d,style:'solid',type:'stroke',width:2.6,...extra}];
const annulus=(x,y,r,w)=>hole(circle(x,y,r+w/2),circle(x,y,r-w/2));
const framed=(name,d,solid=d,extra={})=>[fill(name+'-interior',d,'tint',extra),stroke(name+'-frame',d,extra),fill(name+'-solid',solid,'solid',extra)];
// Explicit cubic segments keep arc topology fixed while its radius changes.
const arc=(cx,cy,r,a,b)=>{const n=Math.max(1,Math.ceil(Math.abs(b-a)/90-1e-8)),angle=d=>d*Math.PI/180,p=t=>[cx+r*Math.cos(t),cy+r*Math.sin(t)];let start=angle(a),d='M'+p(start);for(let i=1;i<=n;i++){const end=angle(a+(b-a)*i/n),k=4/3*Math.tan((end-start)/4),u=p(start),v=p(end);d+=' C'+[u[0]-k*r*Math.sin(start),u[1]+k*r*Math.cos(start),v[0]+k*r*Math.sin(end),v[1]-k*r*Math.cos(end),...v];start=end;}return d;};
const holes=(outer,...cuts)=>cuts.reduce((d,c)=>hole(d,c),outer);
const frame=rect(5.5,5.5,21,21,2.7);

// Home: the outer construction stays still while the leaf folds to its hinge.
const home=polygon([[7.5,13.8],[16,6],[24.5,13.8],[24.5,26],[7.5,26]],[1.1,1.3,1.1,2,2]);
const door=(t,s)=>{const width=6-pulse(t,s)*(s?4.2:1.5);return rect(13,17.5,width,8.5,Math.min(1,width/2));};
const homeParts=[fill('home-interior',home,'tint'),stroke('home-shell',home),stroke('home-door',door),fill('home-solid', (t,s)=>hole(home,door(t,s)))];
// Calendar: no illegible date grid, only one moving current-day tile.
const calendar=rect(6,8,20,18.5,2.4),day=(t,s)=>rect(9+pulse(t,s)*(s?7:2.2),17.7,4,4,1);
const calendarParts=[fill('calendar-interior',calendar,'tint'),stroke('calendar-frame',calendar),stroke('calendar-header','M6.5 13 L25.5 13'),fill('calendar-day',day,'outline'),fill('calendar-solid',(t,s)=>holes(calendar,rect(7.3,12,17.4,2,1),day(t,s))),...line('calendar-binding-left',[11,5.5],[11,10]),...line('calendar-binding-right',[21,5.5],[21,10])];
const clockParts=framed('clock',circle(16,16,10.5),annulus(16,16,10.5,2.6));
const handPose=(hour)=>rigid((t,s)=>({angle:pulse(t,s,hour?.045:0)*(hour?(s?11:3):(s?95:28))}));
clockParts.push(...line('clock-minute',[16,16],[16,9.8],{pose:handPose(false)}),...line('clock-hour',[16,16],[20.5,18.2],{pose:handPose(true)}));
// Camera: the lens is an aperture, so its circular opening may close concentrically.
const camera=polygon([[5.5,11],[10,11],[12,7.5],[19,7.5],[21,11],[26.5,11],[26.5,25],[5.5,25]],[2,1,1.1,1.1,1,2,2.5,2.5]);
const shutter=(t,s)=>pulse(t,s,.055),cameraLens=(t,s)=>circle(16,18,4.4-shutter(t,s)*(s?2.65:.85));
const cameraParts=[fill('camera-interior',camera,'tint'),stroke('camera-body',camera),stroke('camera-aperture',cameraLens),fill('camera-solid',(t,s)=>hole(camera,circle(16,18,4.4-shutter(t,s)*(s?2.65:.85)))),...line('camera-button',[22.7,8.2],[24.2,8.2],{pose:rigid((t,s)=>({dy:pulse(t,s)*(s?.95:.4)}))})];
const hills=(t,s)=>{const p=pulse(t,s)*(s?1:.33);return polygon([[8,22],[13+1.5*p,15.5-1.5*p],[17+.6*p,20],[20.5+.4*p,17.5],[24,22]],[.6,1,.8,.8,.6]);};
const imageParts=framed('image',frame,hole(frame,rect(7.9,7.9,16.2,16.2,1.2)));
imageParts.push(stroke('image-hills',hills),fill('image-solid-hills',hills),...bothFill('image-sun',circle(20.5,10.8,1.4)));
// Mail: the fold has a fixed upper seam; only its lower vertex moves.
const envelope=rect(6,10,20,15.5,2.4),flapY=(t,s)=>18-pulse(t,s)*(s?5.2:2.0);
const mailParts=[fill('mail-interior',envelope,'tint'),stroke('mail-body',envelope),stroke('mail-flap',(t,s)=>{const y=flapY(t,s);return `M7 11.5 L15.1 ${y-.65} C15.6 ${y-.3} 16.4 ${y-.3} 16.9 ${y-.65} L25 11.5`;}),fill('mail-solid',(t,s)=>hole(envelope,polygon([[7.8,11.8],[16,flapY(t,s)],[24.2,11.8],[24.2,14.1],[16,flapY(t,s)+2.3],[7.8,14.1]],.6)))];
const bubble=polygon([[6,7],[26,7],[26,22],[13,22],[7,26],[8,22],[6,22]],[2.5,2.5,2.5,1,.7,.5,2.2]);
const dot=(i,t,s)=>circle(11+i*5,15-pulse(t,s,i*.045)*(s?2.6:.9),1.15);
const messageParts=[fill('message-interior',bubble,'tint'),stroke('message-body',bubble),fill('message-solid',(t,s)=>holes(bubble,...[0,1,2].map(i=>dot(i,t,s))))];
for(let i=0;i<3;i++)messageParts.push(fill('message-dot-'+i,(t,s)=>dot(i,t,s),'outline'));
const phone=polygon([[8,5.5],[11.5,5.5],[13.6,10.5],[10.9,13.2],[14,18],[18.8,21.1],[21.5,18.4],[26.5,20.5],[26.5,24],[24.4,26],[20,26],[13.1,22.1],[9.2,18.3],[5.5,11.5],[5.5,8]],[1.2,1,1.2,.8,1.5,.8,1.2,1,1.2,1.5,1.5,2,2,1.8,1.2]);
const phonePose=rigid((t,s)=>({angle:-pulse(t,s)*(s?9:3),dy:-pulse(t,s)*(s?.7:.2),pivot:[16,24]}));
const phoneParts=framed('phone',phone,phone,{pose:phonePose});
const capsuleMic=rect(13.1,5.5,5.8,11,2.9),micPose=rigid((t,s)=>({angle:pulse(t,s)*(s?14:4),pivot:[16,17]}));
const microphoneParts=[...framed('microphone-capsule',capsuleMic,capsuleMic,{pose:micPose}),...tube('microphone-cradle','M7 13.5 L7 14 C7 18.97 11.03 23 16 23 C20.97 23 25 18.97 25 14 L25 13.5'),...line('microphone-stand',[16,23],[16,27]),...line('microphone-foot',[11.5,27],[20.5,27])];
const speaker=polygon([[5.8,12],[10.5,12],[15.5,7],[15.5,25],[10.5,20],[5.8,20]],[1,.6,1,1,.6,1]);
const volumeParts=framed('volume-speaker',speaker);
for(let i=0;i<2;i++){const r=i?12:7,a=i?40:25;volumeParts.push(...tube('volume-wave-'+i,arc(14,16,r,-a,a),{pose:rigid((t,s)=>({dx:pulse(t,s,i*.045)*(s?1.5:.45)}))}));}
const wifiParts=[fill('wifi-interior','M4.69 13.69 A16 16 0 0 1 27.31 13.69 L16 25 Z','tint'),...bothFill('wifi-source',circle(16,25,1.4))];
for(let i=0;i<3;i++)wifiParts.push(...tube('wifi-band-'+i,(t,s)=>arc(16,25,6+i*5-pulse(t,s,i*.035)*(s?[.9,1.5,2][i]:[.3,.5,.65][i]),-135,-45)));
const cloud='M9 24 C6 24 4.8 22 4.8 19.8 C4.8 17.6 6.3 15.6 8.7 15.3 C8.9 10 13 6.8 17.5 8.8 C20.4 8.4 23.1 11.1 23 14 C25.9 14.2 27.5 16.2 27.5 19 C27.5 22 25.3 24 22.5 24 L9 24 Z';
const cloudPose=(x,y,t,s)=>{const p=pulse(t,s,x>18?.035:0),w=(24-y)/17.2;return[16+(x-16)*(1+p*w*(s?.07:.025)),y-p*w*(s?2.4:.75)];};
const cloudParts=framed('cloud',cloud,cloud,{pose:cloudPose});
const sunParts=framed('sun-disc',circle(16,16,4.2),circle(16,16,4.8));
for(let i=0;i<8;i++){const a=i*45,r=a*Math.PI/180,at=radius=>[16+Math.cos(r)*radius,16+Math.sin(r)*radius];sunParts.push(...line('sun-ray-'+i,at(8.9),at(11.1),{pose:(x,y,t,s)=>{const p=pulse(t,s),angle=p*(s?35:10),delta=p*(s?.45:.12);[x,y]=rotate(x,y,angle);const d=Math.hypot(x-16,y-16);return[x+(x-16)/d*delta,y+(y-16)/d*delta];}}));}
const moon='M12.05 6.82 A10 10 0 1 0 25.18 19.95 C25.42 19.41 24.77 19.35 24.43 19.5 A9.5 9.5 0 0 1 12.58 7.53 C12.77 7.16 12.60 6.57 12.05 6.82 Z';
const moonPose=rigid((t,s)=>({angle:pulse(t,s)*(s?13:4),dy:-pulse(t,s)*(s?1:.3)}));
const moonParts=framed('moon',moon,moon,{pose:moonPose});
const pin='M16 27 C13.6 24.5 7.5 19.3 7.5 14.5 C7.5 10.08 11.3 6.5 16 6.5 C20.7 6.5 24.5 10.08 24.5 14.5 C24.5 19.3 18.4 24.5 16 27 Z';
const pinPose=(x,y,t,s)=>{const p=pulse(t,s),w=(27-y)/20.5;return[16+(x-16)*(1+p*w*(s?.06:.02)),27+(y-27)*(1-p*(s?.135:.04))];};
const locationParts=[fill('location-interior',pin,'tint',{pose:pinPose}),stroke('location-pin',pin,{pose:pinPose}),stroke('location-counter',circle(16,14.5,2.9),{pose:pinPose}),fill('location-solid',hole(pin,circle(16,14.5,3.1)),'solid',{pose:pinPose})];
const needle=polygon([[18.5,10],[18,18],[13.5,22],[14,14]],[.7,.65,.7,.65]);
const compassParts=framed('compass',circle(16,16,10.5),annulus(16,16,10.5,2.6));
const bearing=(t,s)=>gesture(t,s,[[0,0],[.14,-16],[.22,-16],[.50,0]],[[0,0],[.20,-54],[.27,-54],[.56,3],[.78,0]]);
compassParts.push(...bothFill('compass-needle',needle,{pose:rigid((t,s)=>({angle:bearing(t,s)}))}));
const linkParts=[];
for(let i=0;i<2;i++){const a=i?[16.5,15.5]:[8.5,23.5],b=i?[23.5,8.5]:[15.5,16.5],outer=capsule(a,b,6.2),solid=hole(capsule(a,b,8.2),capsule(a,b,3.8)),pose=rigid((t,s)=>{const p=pulse(t,s)*(s?1.1:.35)*(i?-1:1);return{dx:p,dy:-p};});linkParts.push(...framed('link-loop-'+i,outer,solid,{pose}));}
const shield='M16 5.5 C19.8 7.9 23 8.4 24.3 8.5 C24.75 8.53 25 8.8 25 9.2 L25 15 C25 20.8 20.4 24.8 16 27 C11.6 24.8 7 20.8 7 15 L7 9.2 C7 8.8 7.25 8.53 7.7 8.5 C9 8.4 12.2 7.9 16 5.5 Z';
const shieldPose=rigid((t,s)=>({angle:pulse(t,s)*(s?8:2.5),pivot:[16,27]}));
const shieldCheck='M12.5 16.5 L14.4 18.4 C14.75 18.75 15.05 18.75 15.4 18.3 L19.7 13.3';
const shieldParts=[fill('shield-interior',shield,'tint',{pose:shieldPose}),stroke('shield-body',shield,{pose:shieldPose}),stroke('shield-check',shieldCheck,{pose:shieldPose}),fill('shield-solid',hole(shield,polygon([[11.4,16.5],[13,15],[14.9,17],[19,12.4],[20.8,14],[15,20.5]],[.55,.55,.55,.55,.55,.8])),'solid',{pose:shieldPose})];
const basket=polygon([[8.3,10.5],[24.5,10.5],[22.2,20],[10.3,20]],[.5,1,1.2,1]);
const cartTravel=(t,s)=>gesture(t,s,[[0,0],[.18,.7],[.23,.7],[.52,0]],[[0,0],[.05,-.25],[.26,2.4],[.34,2.4],[.80,0]]),cartPose=rigid((t,s)=>({dx:cartTravel(t,s)}));
const cartParts=[...framed('cart-basket',basket,basket,{pose:cartPose}),...tube('cart-handle','M5 6.8 L7.3 6.8 L10.3 20',{pose:cartPose})];
for(let i=0;i<2;i++){const x=i?21.5:12,y=25.5,shape=hole(circle(x,y,1.9),circle(x+.65,y,.55)),pose=(px,py,t,s)=>{const dx=cartTravel(t,s);[px,py]=rotate(px,py,dx/1.9*180/Math.PI,x,y);return[px+dx,py];};cartParts.push(...bothFill('cart-wheel-'+i,shape,{pose}));}
const gearVertices=[];for(let i=0;i<6;i++)for(const [offset,r]of [[-30,8.7],[-14,8.7],[-11,11],[11,11],[14,8.7]]){const a=(i*60+offset)*Math.PI/180;gearVertices.push([16+r*Math.cos(a),16+r*Math.sin(a)]);}
const gear=polygon(gearVertices,.55),gearPose=rigid((t,s)=>({angle:pulse(t,s)*(s?42:12)}));
const settingsParts=[fill('settings-interior',hole(gear,circle(16,16,3.7)),'tint',{pose:gearPose}),stroke('settings-gear',gear,{pose:gearPose}),stroke('settings-center',circle(16,16,3.7),{pose:gearPose}),fill('settings-solid',hole(gear,circle(16,16,4.2)),'solid',{pose:gearPose})];
const timing=(a=.52,b=.80,extra=0)=>({light:{active:a,rest:a+.04,keyPoses:[0,.08,.16,.21,.36,a]},strong:{active:b,rest:b+.04,keyPoses:[0,.12,.24,.32,.53,b]}});
const brief=(name,label,number,parts,intent,timings=timing())=>({name,label,number:String(number),stateful:false,heading:label,description:intent,intent,parts,timings});
export const icons=[
 brief('home','Home',19,homeParts,'The door folds toward its fixed hinge, then closes.'),
 brief('calendar','Calendar',20,calendarParts,'The current day tile moves across the fixed calendar and returns.'),
 brief('clock','Clock',21,clockParts,'The minute hand advances; the hour hand follows.',timing(.56,.84)),
 brief('camera','Camera',22,cameraParts,'The button presses before the aperture closes and opens.',timing(.58,.86)),
 brief('image','Image',23,imageParts,'A bounded landscape shifts inside a fixed frame.'),
 brief('mail','Mail',24,mailParts,'The envelope flap opens toward its top seam, then closes.'),
 brief('message','Message',25,messageParts,'Three dots reply in a short ordered sequence.',timing(.60,.88)),
 brief('phone','Phone',26,phoneParts,'The rigid handset lifts through a short arc and returns.'),
 brief('microphone','Microphone',27,microphoneParts,'The capsule leans toward the speaker within its fixed cradle.'),
 brief('volume','Volume',28,volumeParts,'Two sound arcs move away from the stationary speaker.',timing(.56,.84)),
 brief('wifi','Wi-Fi',29,wifiParts,'Concentric signal bands gather and reopen from a fixed source.',timing(.60,.88)),
 brief('cloud','Cloud',30,cloudParts,'The upper cloud lobes gather and release above a fixed baseline.',timing(.56,.84)),
 brief('sun','Sun',31,sunParts,'Rigid rays turn and extend around a fixed disc.'),
 brief('moon','Moon',32,moonParts,'The crescent rises through a short rigid arc and settles.'),
 brief('location','Location',33,locationParts,'The marker gathers toward its anchored lower point.'),
 brief('compass','Compass',34,compassParts,'The needle finds its bearing inside a fixed reference ring.',timing(.54,.82)),
 brief('link','Link',35,linkParts,'Two rigid loops draw together while remaining visibly joined.'),
 brief('shield','Shield',36,shieldParts,'The shield braces about its lower point, with its mark attached.'),
 brief('cart','Cart',37,cartParts,'The basket advances with rolling wheels and returns.',timing(.56,.84)),
 brief('settings','Settings',38,settingsParts,'A rigid rounded gear makes one deliberate adjustment.'),
];

directInterface(icons);
