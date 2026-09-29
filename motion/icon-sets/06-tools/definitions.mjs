import { keyframes } from '../library/keyframes.mjs';
import { directTools } from '../library/directed-tools.mjs';
import svgpath from 'svgpath';

export const mix=(a,b,p)=>a+(b-a)*p;
// Each interval is a deliberate eased move or a still hold. Sample once at 60 fps.
export function phrase(keys,t){if(t<=keys[0][0])return keys[0][1];if(t>=keys.at(-1)[0])return keys.at(-1)[1];const i=keys.findIndex(k=>k[0]>t),a=keys[i-1],b=keys[i];return mix(a[1],b[1],(1-Math.cos(Math.PI*(t-a[0])/(b[0]-a[0])))/2);}
export function rotate(x,y,a,cx=16,cy=16){const r=a*Math.PI/180,c=Math.cos(r),s=Math.sin(r);return[cx+(x-cx)*c-(y-cy)*s,cy+(x-cx)*s+(y-cy)*c];}
export const circle=(x,y,r)=>`M${x} ${y-r} C${x+r*.552285} ${y-r} ${x+r} ${y-r*.552285} ${x+r} ${y} C${x+r} ${y+r*.552285} ${x+r*.552285} ${y+r} ${x} ${y+r} C${x-r*.552285} ${y+r} ${x-r} ${y+r*.552285} ${x-r} ${y} C${x-r} ${y-r*.552285} ${x-r*.552285} ${y-r} ${x} ${y-r} Z`;
export const rect=(x,y,w,h,r)=>{r=Math.min(r,w/2,h/2);return `M${x+r} ${y} L${x+w-r} ${y} C${x+w-r*.448} ${y} ${x+w} ${y+r*.448} ${x+w} ${y+r} L${x+w} ${y+h-r} C${x+w} ${y+h-r*.448} ${x+w-r*.448} ${y+h} ${x+w-r} ${y+h} L${x+r} ${y+h} C${x+r*.448} ${y+h} ${x} ${y+h-r*.448} ${x} ${y+h-r} L${x} ${y+r} C${x} ${y+r*.448} ${x+r*.448} ${y} ${x+r} ${y} Z`};
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

// People: stable shoulder silhouettes, with a clear head gesture or a second member.
const shoulders='M7 26 L7 24 C7 20.2 11 18 16 18 C21 18 25 20.2 25 24 L25 26 C25 26.6 24.6 27 24 27 L8 27 C7.4 27 7 26.6 7 26 Z';
const userHeadPose=rigid((t,s)=>({dx:pulse(t,s)*(s?2.2:.7),dy:pulse(t,s)*(s?1:.3)}));
const userParts=[...framed('user-shoulders',shoulders),...framed('user-head',circle(16,9,3.7),circle(16,9,4.5),{pose:userHeadPose})];
const frontShoulders='M5 26 L5 24 C5 20.4 8.4 18.8 12.5 18.8 C16.6 18.8 20 20.4 20 24 L20 26 Z';
const rearPose=rigid((t,s)=>({dy:-pulse(t,s,.045)*(s?1.8:.55),dx:pulse(t,s,.045)*(s?.65:.2)}));
const usersParts=[...framed('users-rear-head',circle(23,8.8,3),circle(23,8.8,3.8),{pose:rearPose}),...tube('users-rear-shoulder','M23 16 C25.4 16 27 19 27 23.5',{pose:rearPose}),...framed('users-front-shoulders',frontShoulders),...framed('users-front-head',circle(12.5,11,3.4),circle(12.5,11,4.1))];
// Key: all teeth remain attached to the rigid shaft as it turns at the bow.
const keyPose=rigid((t,s)=>({angle:-pulse(t,s)*(s?18:5),pivot:[9.5,11]}));
const keyParts=[...framed('key-bow',circle(9.5,11,4.7),annulus(9.5,11,4.7,2.6),{pose:keyPose}),...tube('key-shaft','M12.8 14.3 L23 24.5 L25.7 21.8',{pose:keyPose}),...line('key-tooth',[19,20.5],[21.7,17.8],{pose:keyPose})];
const scanParts=[fill('scan-field',rect(6,6,20,20,2.2),'tint')];
for(const [name,d] of [['tl','M6 12 L6 8.2 C6 6.8 6.8 6 8.2 6 L12 6'],['tr','M20 6 L23.8 6 C25.2 6 26 6.8 26 8.2 L26 12'],['br','M26 20 L26 23.8 C26 25.2 25.2 26 23.8 26 L20 26'],['bl','M12 26 L8.2 26 C6.8 26 6 25.2 6 23.8 L6 20']])scanParts.push(...tube('scan-corner-'+name,d));
scanParts.push(...line('scan-bar',[11,12],[21,12],{pose:rigid((t,s)=>({dy:pulse(t,s)*(s?8:2.5)}))}));
const battery=rect(5,10,20,12,2.2),charge=(t,s)=>rect(8,13,4+pulse(t,s)*(s?9:3.2),6,1);
const batteryParts=[...framed('battery-shell',battery,hole(battery,rect(7.4,12.4,15.2,7.2,.9))),...bothFill('battery-charge',charge),...line('battery-terminal',[28,14],[28,18])];
const plugBody='M9 12 L23 12 L23 16 C23 20 20 22 16 22 C12 22 9 20 9 16 Z';
const plugTravel=(t,s)=>pulse(t,s)*(s?2.2:.65),plugPose=rigid((t,s)=>({dy:plugTravel(t,s)}));
const plugParts=[...framed('plug-body',plugBody,plugBody,{pose:plugPose}),...line('plug-prong-left',[12,6],[12,12],{pose:plugPose}),...line('plug-prong-right',[20,6],[20,12],{pose:plugPose}),...tube('plug-cable',(t,s)=>`M16 ${22+plugTravel(t,s)} L16 27`)];
const bluetoothD='M10 10 L22 21 L16 26 L16 6 L22 11 L10 22';
const bluetoothPose=(x,y,t,s)=>{const p=pulse(t,s)*(s?2.4:.75);return[x+(x-16)/6*p,y];};
const bluetoothParts=[fill('bluetooth-upper-field','M16 6 L22 11 L16 16 Z','tint',{pose:bluetoothPose}),fill('bluetooth-lower-field','M16 16 L22 21 L16 26 Z','tint',{pose:bluetoothPose}),...tube('bluetooth-rune',bluetoothD,{pose:bluetoothPose})];
// Displays: preserve the stand or hinge and let the related part carry the action.
const monitor=rect(5,6.5,22,15,2.4),monitorPose=rigid((t,s)=>({angle:pulse(t,s)*(s?9:2.8),pivot:[16,21.5]}));
const monitorParts=[...framed('monitor-screen',monitor,hole(monitor,rect(8,17.5,16,1.8,.9)),{pose:monitorPose}),...line('monitor-stand',[16,21.5],[16,26.5]),...line('monitor-foot',[11,26.5],[21,26.5])];
const phoneFrame=rect(10,4.5,12,23,2.6),swipe=(t,s)=>pulse(t,s)*(s?4:1.2),phoneTile=(t,s)=>rect(13.4,12.5+swipe(t,s),5.2,4.2,.9);
const smartphoneParts=[fill('smartphone-field',phoneFrame,'tint'),stroke('smartphone-frame',phoneFrame),stroke('smartphone-speaker','M14 8 L18 8'),fill('smartphone-content',phoneTile,'outline'),fill('smartphone-solid',(t,s)=>holes(phoneFrame,rect(14,7.3,4,1.5,.75),phoneTile(t,s)))];
const laptopScreen=rect(7,6.5,18,14,1.8),lidPose=(x,y,t,s)=>[x,20.5+(y-20.5)*(1-pulse(t,s)*(s?.36:.10))];
const laptopBase='M4.5 24.5 C5 25.7 6 26.5 7.5 26.5 L24.5 26.5 C26 26.5 27 25.7 27.5 24.5';
const laptopParts=[...framed('laptop-screen',laptopScreen,laptopScreen,{pose:lidPose}),...tube('laptop-base',laptopBase)];
const housing='M8 22 L7.5 22 C6.4 22 5.5 21.1 5.5 20 L5.5 14.5 C5.5 13.4 6.4 12.5 7.5 12.5 L24.5 12.5 C25.6 12.5 26.5 13.4 26.5 14.5 L26.5 20 C26.5 21.1 25.6 22 24.5 22 L24 22';
const printFeed=(t,s)=>pulse(t,s)*(s?3:1),printPage=(t,s)=>rect(11,18,10,6+printFeed(t,s),.9);
const inputPaper='M10 12.5 L10 7 C10 6.2 10.7 5.5 11.5 5.5 L20.5 5.5 C21.3 5.5 22 6.2 22 7 L22 12.5';
const printerParts=[fill('printer-field',rect(5.5,12.5,21,9.5,2),'tint'),...tube('printer-input',inputPaper),...tube('printer-housing',housing),stroke('printer-sheet',printPage),stroke('printer-line',(t,s)=>`M14 ${21+printFeed(t,s)*.5} L18 ${21+printFeed(t,s)*.5}`),fill('printer-solid-sheet',(t,s)=>hole(printPage(t,s),rect(13.5,20.4+printFeed(t,s)*.5,5,1.7,.8)))];
// Paper controls keep content marks inside the page and never fade the container.
const documentBody=polygon([[8,5.5],[18.5,5.5],[24,11],[24,26.5],[8,26.5]],[1.7,.9,.9,1.7,1.7]);
const writing=(i,t,s)=>5-i*.5+pulse(t,s,i*.055)*(s?(i?3:4):(i?1:1.4));
const documentParts=[fill('document-field',documentBody,'tint'),stroke('document-page',documentBody),stroke('document-fold','M18.5 6.5 L18.5 10 C18.5 10.7 18.8 11 19.5 11 L23 11'),fill('document-solid',(t,s)=>holes(documentBody,rect(11,15,writing(0,t,s),2,1),rect(11,20.5,writing(1,t,s),2,1),polygon([[19.5,7.5],[22,10],[19.5,10]],.3)))];
for(let i=0;i<2;i++)documentParts.push(stroke('document-line-'+i,(t,s)=>`M11 ${16+i*5.5} L${11+writing(i,t,s)} ${16+i*5.5}`));
const clipboardBody='M12 8.5 L10 8.5 C8.9 8.5 8 9.4 8 10.5 L8 24.5 C8 25.6 8.9 26.5 10 26.5 L22 26.5 C23.1 26.5 24 25.6 24 24.5 L24 10.5 C24 9.4 23.1 8.5 22 8.5 L20 8.5';
const clip=(t,s)=>{const d=pulse(t,s)*(s?1.8:.55);return rect(12,5.5+d,8,4-d,.7);};
const clipboardParts=[fill('clipboard-field',rect(8,8.5,16,18,2),'tint'),stroke('clipboard-board',clipboardBody),stroke('clipboard-clip',clip),stroke('clipboard-line-0','M12 16 L20 16'),stroke('clipboard-line-1','M12 21.5 L17.5 21.5'),fill('clipboard-solid-board',holes(rect(8,8.5,16,18,2),rect(12,15,8,2,1),rect(12,20.5,5.5,2,1))),fill('clipboard-solid-clip',clip)];
const leftPage='M16 8 C12 5.8 8.3 5.5 5.5 6 L5.5 24 C9 23.5 12.5 24 16 26 Z';
const rightPage='M16 8 C20 5.8 23.7 5.5 26.5 6 L26.5 24 C23 23.5 19.5 24 16 26 Z';
const turnPage=(x,y,t,s)=>[16+(x-16)*(1-pulse(t,s)*(s?.50:.16)),y];
const bookParts=[fill('book-left-field',leftPage,'tint'),fill('book-right-field',rightPage,'tint',{pose:turnPage}),stroke('book-left-page',leftPage),stroke('book-right-page',rightPage,{pose:turnPage}),fill('book-left-solid',leftPage.replaceAll('M16','M14.8').replace(' 16 26',' 14.8 26')),fill('book-right-solid',rightPage.replaceAll('M16','M17.2').replace(' 16 26',' 17.2 26'),'solid',{pose:turnPage})];
const creditCard=rect(6,8.5,20,15.5,2.5),cardSwipe=rigid((t,s)=>({dx:pulse(t,s)*(s?2.4:.75)}));
const creditParts=[fill('credit-card-field',creditCard,'tint',{pose:cardSwipe}),stroke('credit-card-frame',creditCard,{pose:cardSwipe}),stroke('credit-card-stripe','M6.5 14 L25.5 14',{pose:cardSwipe}),fill('credit-card-chip',rect(9,18.5,4.5,2,1),'outline',{pose:cardSwipe}),fill('credit-card-solid',holes(creditCard,rect(7,12.8,18,2.2,1.1),rect(9,18.5,4.5,2,1)),'solid',{pose:cardSwipe})];
const walletFrame='M24.5 13.5 L24.5 10.7 C24.5 9.5 23.5 8.5 22.3 8.5 L7.7 8.5 C6.5 8.5 5.5 9.5 5.5 10.7 L5.5 22.3 C5.5 23.5 6.5 24.5 7.7 24.5 L22.3 24.5 C23.5 24.5 24.5 23.5 24.5 22.3 L24.5 20.5';
const walletReach=(t,s)=>pulse(t,s)*(s?3:.9),walletFlap=(t,s)=>rect(20-walletReach(t,s),13.5,7+walletReach(t,s),7,1.4),walletClasp=(t,s)=>circle(23.5-walletReach(t,s)*.5,17,.85);
const walletOuter=rect(5.5,8.5,19,16,2.2);
const walletParts=[fill('wallet-field',walletOuter,'tint'),stroke('wallet-body',walletFrame),stroke('wallet-flap',walletFlap),fill('wallet-clasp',walletClasp,'outline'),{name:'wallet-solid-body',d:walletFrame,style:'solid',type:'stroke',width:2.6},fill('wallet-solid-flap',(t,s)=>hole(walletFlap(t,s),walletClasp(t,s)))];
// The lid carries the upper box junctions; the base stays planted. Sharing this
// driver prevents three loose round caps from appearing at the Strong peak.
const giftTravel=(t,s)=>pulse(t,s)*(s?2.2:.7),giftTop=(t,s)=>15.5-giftTravel(t,s);
const giftBody=(t,s)=>`M7 ${giftTop(t,s)} L7 25.2 C7 26.2 7.8 27 8.8 27 L23.2 27 C24.2 27 25 26.2 25 25.2 L25 ${giftTop(t,s)}`;
const giftField=(t,s)=>rect(7,giftTop(t,s),18,27-giftTop(t,s),1.8);
const giftLid=rect(5.5,10.5,21,5,1.3),giftLift=rigid((t,s)=>({dy:-giftTravel(t,s)}));
const giftParts=[fill('gift-field',giftField,'tint'),stroke('gift-box',giftBody),stroke('gift-ribbon',(t,s)=>`M16 ${giftTop(t,s)} L16 27`),fill('gift-solid-box',(t,s)=>hole(giftField(t,s),rect(14.7,16.8-giftTravel(t,s),2.6,8.9+giftTravel(t,s),1))),...framed('gift-lid',giftLid,giftLid,{pose:giftLift}),...tube('gift-bow-left','M16 10.5 C14 7 11 5.7 9.6 7 C8.2 8.3 10.6 10.5 16 10.5',{pose:giftLift}),...tube('gift-bow-right','M16 10.5 C18 7 21 5.7 22.4 7 C23.8 8.3 21.4 10.5 16 10.5',{pose:giftLift})];
const flagCloth='M8 6 C14 4.5 18 10 25 7 L25 18 C18 21 14 15.5 8 17 Z';
const flagWave=(x,y,t,s)=>{const w=(x-8)/17,p=pulse(t,s,w*.065);return[x+p*w*(s?.6:.2),y-p*w*(s?3.2:1)];};
const flagParts=[...framed('flag-cloth',flagCloth,flagCloth,{pose:flagWave}),...line('flag-pole',[8,5],[8,27])];
const chartParts=[];
for(let i=0;i<3;i++){const bar=(t,s)=>{const h=8+i*5+pulse(t,s,i*.05)*(s?[5,4,2][i]:[1.5,1.2,.6][i]);return rect(6+i*8,26-h,4,h,1.2);};chartParts.push(...framed('chart-bar-'+i,bar));}
const filterWidth=(i,t,s)=>[20,13,6][i]-(i?pulse(t,s,(i-1)*.055)*(s?[0,5,3][i]:[0,1.5,.9][i]):0);
const filterParts=[fill('filter-field',(t,s)=>polygon([[6,8],[26,8],[16+filterWidth(2,t,s)/2,24],[16-filterWidth(2,t,s)/2,24]],.9),'tint')];
for(let i=0;i<3;i++)filterParts.push(...tube('filter-tier-'+i,(t,s)=>`M${16-filterWidth(i,t,s)/2} ${8+i*8} L${16+filterWidth(i,t,s)/2} ${8+i*8}`));
const timing=(a=.54,b=.82)=>({light:{active:a,rest:a+.04,keyPoses:[0,.08,.16,.21,.36,a]},strong:{active:b,rest:b+.04,keyPoses:[0,.12,.24,.32,.53,b]}});
const brief=(name,label,number,parts,intent,timings=timing())=>({name,label,number:String(number),stateful:false,heading:label,description:intent,intent,parts,timings});
export const icons=[
brief('user','User',39,userParts,'The head inclines while the shoulders stay still.'),
brief('users','Users',40,usersParts,'The second member steps forward beside a fixed foreground profile.',timing(.58,.86)),
brief('key','Key',41,keyParts,'The complete key turns at its bow and returns.'),
brief('scan','Scan',42,scanParts,'A scan bar travels inside fixed focus brackets.'),
brief('battery','Battery',43,batteryParts,'The charge region expands inside a fixed cell and returns.'),
brief('plug','Plug',44,plugParts,'The rigid plug seats toward its cable attachment and releases.'),
brief('bluetooth','Bluetooth',45,bluetoothParts,'The two signal facets open around a fixed spine.'),
brief('monitor','Monitor',46,monitorParts,'The screen tilts at its fixed stand and returns.'),
brief('smartphone','Smartphone',47,smartphoneParts,'A content tile scrolls within the stationary phone.'),
brief('laptop','Laptop',48,laptopParts,'The display folds toward its lower hinge and reopens.'),
brief('printer','Printer',49,printerParts,'Paper advances from a fixed output slot and returns.'),
brief('document','Document',50,documentParts,'Two lines compose in order inside the fixed page.',timing(.60,.88)),
brief('clipboard','Clipboard',51,clipboardParts,'The clip presses toward its attachment and releases.'),
brief('book','Book',52,bookParts,'The right leaf turns toward the fixed spine and opens again.'),
brief('credit-card','Credit card',53,creditParts,'The rigid payment card makes one straight swipe and returns.'),
brief('wallet','Wallet',54,walletParts,'The clasped flap opens from its fixed right edge and returns.'),
brief('gift','Gift',55,giftParts,'The lid and bow lift; the box sides and ribbon follow while the base stays fixed.'),
brief('flag','Flag',56,flagParts,'One cloth wave travels toward the free edge while the pole stays fixed.',timing(.60,.90)),
brief('chart','Chart',57,chartParts,'Three bars grow in order from one fixed baseline.',timing(.64,.92)),
brief('filter','Filter',58,filterParts,'The lower tiers narrow in order below the fixed top line.',timing(.60,.88)),
];

directTools(icons);

// Additions retain the earlier twenty definitions and directed motions.
icons.push(keyframes);
