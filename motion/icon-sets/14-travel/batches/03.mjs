import {make,P,F,posed,rigid,rect,circle,polygon,fill,stroke,ink,surface,policy,capsule,arch,framed,holes} from './helpers.mjs';
const flutter=F([[0,0],[.25,.3,'out'],[.5,-.15,'smooth'],[.75,0,'smooth']],[[0,0],[.15,-.7],[.45,1.3],[.8,-.5],[1.15,0]]);
const flag=(t,s)=>{const a=flutter(t,s);return `M10.5 4.8 C13 4.8 ${15+a} ${4.8+a} 17.5 ${5.1+a} L17.5 ${8.5+a} C15 ${8.2+a} 13 8.2 10.5 8.2 Z`;};
const hotel=policy(make('hotel','Hotel','A welcome flag catches and releases a short breeze above a still, readable hotel facade.',[
 ...surface('facade',rect(7,12.5,18,15,1.5),[capsule([10.5,16.5],[13.5,16.5],2.2),capsule([18.5,16.5],[21.5,16.5],2.2),rect(13.5,21,5,5.2,1)]),stroke('window-left','M10.5 16.5 L13.5 16.5'),stroke('window-right','M18.5 16.5 L21.5 16.5'),stroke('door',arch(13.5,18.5,27.5,21,1)),...ink('mast','M10.5 4.8 L10.5 12.5'),...surface('flag',flag)
],.83,1.23));hotel.constraints=[{type:'fixed',part:'hotel-facade'},{type:'fixed',part:'hotel-mast'}];
const fluff=F([[0,12.5],[.3,11.7,'out'],[.7,12.5,'smooth']],[[0,12.5],[.15,13.3],[.45,10.2],[.85,12.8],[1.15,12.5]]);
const quilt=F([[0,13.5],[.35,13.2,'out'],[.75,13.5,'smooth']],[[0,13.5],[.25,13.5],[.6,12.3],[1,13.65],[1.28,13.5]]);
const pillow=(t,s)=>arch(9.5,15.5,17,fluff(t,s),1.5),blanket=(t,s)=>arch(19,25.5,17,quilt(t,s),1.5);
const mattress='M6 17 L25.5 17 C26.328 17 27 17.672 27 18.5 L27 20.5 C27 21.328 26.328 22 25.5 22 L6 22';
const cushion=(name,d)=>[fill(name+'-interior',(t,s)=>d(t,s)+' Z','tint'),...ink(name,d),fill(name+'-solid',(t,s)=>d(t,s)+' Z','solid')];
const bed=policy(make('bed','Bed','The pillow compresses then lofts; the adjacent duvet rises a moment later and both return to the mattress seam.',[
 ...ink('headboard','M6 25 L6 10'),...ink('foot','M27 22 L27 25'),...cushion('pillow',pillow),...cushion('blanket',blanket),fill('mattress-interior',mattress+' Z','tint'),fill('mattress-solid',mattress+' Z','solid'),...ink('mattress',mattress)
],.83,1.28));bed.constraints=[{type:'fixed',part:'bed-mattress'},{type:'fixed',part:'bed-headboard'}];
const left=F([[0,12],[.35,10.5,'out'],[.75,12,'smooth']],[[0,12],[.14,12.6],[.48,8.8],[.88,12.4],[1.15,12]]);
const right=F([[0,20],[.42,21.5,'out'],[.8,20,'smooth']],[[0,20],[.2,19.4],[.56,23.2],[.96,19.6],[1.25,20]]);
const shell=polygon([[5,26],[16,7],[27,26]],1.4);
const opening=(t,s)=>`M${left(t,s)} 26 L15.65 16.6 C15.9 15.75 16.1 15.75 16.35 16.6 L${right(t,s)} 26`;
const canvas=(t,s)=>polygon([[5,26],[16,7],[27,26],[right(t,s),26],[16,15.5],[left(t,s),26]],.65);
const tent=policy(make('tent','Tent','Two fabric flaps open from a shared fixed apex; their hems slide along the floor and close with a short stagger.',[
 fill('canvas-tint',canvas,'tint'),fill('canvas-solid',canvas,'solid'),...ink('shell',shell),...ink('door',opening)
],.88,1.28));tent.constraints=[{type:'fixed',part:'tent-shell'}];
const lean=F([[0,0],[.35,-5,'out'],[.75,0,'smooth']],[[0,0],[.18,2.5],[.55,-7.4],[1,1.5],[1.3,0]]);
const cloth=F([[0,0],[.45,.2,'out'],[.8,0,'smooth']],[[0,0],[.28,-.3],[.62,1.1],[1,-.5],[1.32,0]]);
const canopy=(t,s)=>{let a=cloth(t,s);return `M6 14 C6 9.029 10.029 5 15 5 C19.971 5 24 9.029 24 14 C22.2 ${15.6+a} 20.3 ${15.6+a} 18.5 14 C16.2 ${15.6+a} 13.8 ${15.6+a} 11.5 14 C9.7 ${15.6+a} 7.8 ${15.6+a} 6 14 Z`;};
const umbrella=policy(make('beach-umbrella','Beach umbrella','A breeze leans the complete umbrella around its planted foot; the canopy scallops flex a moment later.',[
 ...posed([...ink('pole','M15 5 L15 27'),...surface('canopy',canopy)],rigid((t,s)=>({angle:lean(t,s),pivot:[15,27]}))),...ink('sand','M7 27 L27 27')
],.88,1.38));umbrella.constraints=[{type:'fixed',part:'beach-umbrella-sand'},{type:'rigid',part:'beach-umbrella-pole'}];
const foldL=F([[0,7],[.75,7]],[[0,7],[.12,7.5],[.45,3],[.9,7.5],[1.2,7]]);
const foldR=F([[0,7],[.3,5,'out'],[.7,7,'smooth']],[[0,7],[.2,7.5],[.55,3],[1,7.5],[1.3,7]]);
const mapOuter=(t,s)=>polygon([[12-foldL(t,s),9],[12,6],[20,9],[20+foldR(t,s),6],[20+foldR(t,s),23],[20,26],[12,23],[12-foldL(t,s),26]],.6);
const map=policy(make('map','Map','The paper folds at its two fixed creases. Strong folds both outer panels in a short cascade before spreading flat.',[
 fill('paper-tint',mapOuter,'tint'),stroke('paper',mapOuter),stroke('fold-left','M12 6.6 L12 23'),stroke('fold-right','M20 9 L20 25.4'),
 fill('panel-left',(t,s)=>polygon([[12-foldL(t,s),9],[11.25,6.4],[11.25,22.7],[12-foldL(t,s),26]],.6),'solid'),fill('panel-middle',polygon([[12.75,6.4],[19.25,8.85],[19.25,25.6],[12.75,23.15]],.5),'solid'),fill('panel-right',(t,s)=>polygon([[20.75,9.25],[20+foldR(t,s),6],[20+foldR(t,s),23],[20.75,25.65]],.6),'solid'),{name:'solid-outer',d:mapOuter,type:'stroke',style:'solid',width:2.6}
],.83,1.38));map.constraints=[{type:'fixed',part:'map-fold-left'},{type:'fixed',part:'map-fold-right'}];
export const icons=[hotel,bed,tent,umbrella,map];
