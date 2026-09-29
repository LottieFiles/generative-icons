import {make,F as P,posed,rigid,rect,circle,polygon,fill,stroke,ink,surface,plainRing,policy,capsule,held,bidir,holes,arch,panel} from './helpers.mjs';
// Strong drive uses a visible recoil, acceleration and body pitch in both directions.
const drive=bidir([[0,0],[.03,0],[.7,3,'smooth']],[[0,0],[.03,0],[.16,-1.1,'launch'],[.44,3.7],[.64,2.8],[.82,3]],[[0,3],[.65,0,'smooth']],[[0,3],[.03,3],[.14,3.5,'launch'],[.46,-.7],[.64,.2],[.82,0]]);
const suspension=bidir([[0,0]],[[0,0],[.03,0],[.16,.9,'launch'],[.44,-.45],[.64,.3],[.82,0]],[[0,0]],[[0,0],[.03,0],[.14,.45,'launch'],[.46,-.25],[.64,.2],[.82,0]]);
const pitch=bidir([[0,0]],[[0,0],[.03,0],[.16,1.8,'launch'],[.44,-5.5],[.64,1],[.82,0]],[[0,0]],[[0,0],[.03,0],[.14,3,'launch'],[.46,-2.2],[.64,.5],[.82,0]]);
const truckPose=rigid((t,s,r)=>({dx:drive(t,s,r),dy:suspension(t,s,r),angle:pitch(t,s,r),pivot:[9.5,24]}));
const truckOuter='M7 9 L15 9 C15.828 9 16.5 9.672 16.5 10.5 L16.5 13 L20 13 C22 13 23.5 14.5 23.5 16.5 L23.5 19 L7 19 C6.172 19 5.5 18.328 5.5 17.5 L5.5 10.5 C5.5 9.672 6.172 9 7 9 Z';
const truck=policy(held(make('delivery-truck','Delivery truck','Light rolls forward smoothly. Strong recoils onto its suspension, accelerates with a raised nose and settles under braking. Its reverse has a clear brake dip and powered back-out.',[
 ...posed([stroke('cargo',rect(5.5,9,11,10,1.5)),stroke('cab','M16.5 13 L20 13 C22 13 23.5 14.5 23.5 16.5 L23.5 19 L15 19'),fill('cargo-tint',rect(5.5,9,11,10,1.5),'tint'),fill('body-solid',truckOuter,'solid'),{name:'body-edge',d:truckOuter,type:'stroke',width:2.6,style:'solid'}],truckPose),
 ...ink('support-left',(t,s,r)=>`M${truckPose(9.5,19,t,s,r)} L${9.5+drive(t,s,r)} 21.5`),...ink('support-right',(t,s,r)=>`M${truckPose(19.5,19,t,s,r)} L${19.5+drive(t,s,r)} 21.5`),
 ...posed([...plainRing('wheel-left',9.5,24,2.5),...plainRing('wheel-right',19.5,24,2.5)],rigid((t,s,r)=>({dx:drive(t,s,r)})))
],.78,.90),'waiting','advanced',.73,.88));truck.revision='strong-motion-audit-v2';truck.constraints=[{type:'rigid',part:'delivery-truck-cargo'},{type:'rigid',part:'delivery-truck-wheel-left'}];
const trackX=bidir([[0,0],[.03,0],[.7,16,'smooth']],[[0,0],[.04,0],[.5,8,'pass'],[.96,16]],[[0,16],[.75,0,'smooth']],[[0,16],[.42,8,'pass'],[.86,0]]);
const hop=bidir([[0,0],[.03,0],[.365,-.5,'smooth'],[.7,0,'smooth']],[[0,0],[.04,0],[.26,-3.2],[.5,0,'hit'],[.74,-3.2],[.96,0,'hit'],[1.06,-.4],[1.16,0]],[[0,0],[.375,-.5,'smooth'],[.75,0,'smooth']],[[0,0],[.2,-3.2],[.42,0,'hit'],[.64,-3.2],[.86,0,'hit'],[.94,-.4],[1.02,0]]);
const pinD='M8 17.5 L10.64 13.98 C11.05 13.43 11.3 12.74 11.3 12 C11.3 10.18 9.82 8.7 8 8.7 C6.18 8.7 4.7 10.18 4.7 12 C4.7 12.74 4.95 13.43 5.36 13.98 Z';
const tracking=policy(held(make('order-tracking','Order tracking','Light advances the shipment marker smoothly. Strong makes two high, connected hops between fixed shipment stops.',[
 ...plainRing('stop-one',8,24,2),...plainRing('stop-two',16,24,2),...plainRing('stop-three',24,24,2),...ink('track-one','M10.15 24 L13.85 24'),...ink('track-two','M18.15 24 L21.85 24'),
 ...posed(surface('pin',pinD,[circle(8,12,1.2)]),rigid((t,s,r)=>({dx:trackX(t,s,r),dy:hop(t,s,r)})))
],.78,1.33),'first-stop','delivered',.83,1.08));tracking.constraints=[{type:'fixed',part:'order-tracking-stop-one'},{type:'rigid',part:'order-tracking-pin'}];
const slide=bidir([[0,0],[.03,0],[.7,5,'smooth']],[[0,0],[.05,0],[.18,-.6,'in'],[.42,6,'out'],[.68,4.65,'smooth'],[1.02,5,'smooth']],[[0,5],[.65,0,'smooth']],[[0,5],[.12,5.6,'out'],[.43,0,'in']]);
const locker=policy(held(make('pickup-locker','Pickup locker','Light slides the door open smoothly. Strong unlatches, snaps the door farther open and settles. The complete fixed frame stays clear of the moving door.',[
 ...panel('frame',4.5,5.5,23,21.5,2).filter(p=>p.style!=='tint'),...posed([...surface('door',rect(8.75,9,9.25,14,1.2),[capsule([14.5,13.5],[14.5,18.5],2.6)]),stroke('handle','M14.5 13.5 L14.5 18.5')],rigid((t,s,r)=>({dx:slide(t,s,r)})))
],.78,1.33),'latched','released',.73,.88));locker.constraints=[{type:'fixed',part:'pickup-locker-frame-frame'},{type:'rigid',part:'pickup-locker-door'},{type:'rigid',part:'pickup-locker-handle'}];
const rail=bidir([[0,23.5],[.03,23.5],[.7,18.5,'out']],[[0,23.5],[.04,23.5],[.2,24.6,'in'],[.5,16.6,'out'],[.78,19.2,'smooth'],[1.12,18.5,'smooth']],[[0,18.5],[.65,23.5,'smooth']],[[0,18.5],[.14,17.5,'out'],[.48,23.5,'in']]);
const building=polygon([[4.5,13],[16,6.5],[27.5,13],[27.5,27],[4.5,27]],.9),doorway=rect(10,15,12,10.75,1);
const warehouse=policy(held(make('warehouse','Warehouse','Light rolls the shutter up quietly. Strong takes up the slack, lifts it rapidly to a higher opening and settles on its stop. The jambs stay fixed.',[
 fill('walls-tint',holes(building,doorway),'tint'),stroke('building',building),fill('building-solid',holes(building,doorway),'solid'),{name:'solid-edge',d:building,type:'stroke',width:2.6,style:'solid'},stroke('jambs',arch(10,22,27,15,1)),
 fill('curtain-tint',(t,s,r)=>rect(10,15,12,rail(t,s,r)-15,.5),'tint'),fill('curtain-solid',(t,s,r)=>rect(10,15,12,rail(t,s,r)-15+1.3,.5),'solid'),...ink('rail',(t,s,r)=>`M10 ${rail(t,s,r)} L22 ${rail(t,s,r)}`)
],.78,1.33),'lowered','raised',.73,.88));warehouse.constraints=[{type:'fixed',part:'warehouse-building'},{type:'fixed',part:'warehouse-jambs'}];
const stockX=bidir([[0,0],[.25,0],[.6,13,'smooth']],[[0,0],[.1,0],[.32,13]],[[0,13],[.2,13],[.5,0,'smooth']],[[0,13],[.06,13],[.6,0]]);
const stockY=bidir([[0,0],[.03,0],[.25,-1.5,'out'],[.6,-1.5],[.7,0,'in']],[[0,0],[.03,0],[.18,-2.6],[.34,0,'hit'],[.42,-.3],[.54,0]],[[0,0],[.2,-1.5,'out'],[.5,-1.5],[.65,0,'in']],[[0,0],[.03,0],[.26,-2.6],[.6,0,'hit'],[.7,-.3],[.84,0]]);
const stockTilt=bidir([[0,0]],[[0,0],[.06,0],[.2,8],[.34,0],[.48,0],[.6,0]],[[0,0]],[[0,0],[.06,0],[.26,-8],[.54,0],[.72,0],[.86,0]]);
const parcel=(name,x,y)=>[...surface(name,rect(x,y,9,8,1)),stroke(name+'-tape',`M${x+4.5} ${y} L${x+4.5} ${y+8}`)];
const inventory=policy(held(make('inventory','Inventory','Light lifts and transfers the parcel smoothly. Strong lifts it clear, transfers it faster with a rigid tilt and seats it in the next column.',[
 ...parcel('box-left',5,19),...parcel('box-right',18,19),...posed(parcel('moving-box',5,7.5),rigid((t,s,r)=>({dx:stockX(t,s,r),dy:stockY(t,s,r),angle:stockTilt(t,s,r),pivot:[9.5,11.5]})))
],.78,.8),'left-column','right-column',.73,.88));inventory.constraints=[{type:'fixed',part:'inventory-box-left'},{type:'fixed',part:'inventory-box-right'},{type:'rigid',part:'inventory-moving-box'}];
export const icons=[truck,tracking,locker,warehouse,inventory];
