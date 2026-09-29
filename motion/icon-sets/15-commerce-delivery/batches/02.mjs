import {make,F as P,posed,rigid,rect,circle,polygon,fill,stroke,ink,surface,plainRing,policy,capsule,held,bidir,mix,holes,bothFill,annulus} from './helpers.mjs';
const strike=P([[0,1],[.03,1],[.25,.18,'in'],[.7,1,'out']],[[0,1],[.06,1],[.32,.02,'launch'],[.5,1.04],[.72,.97],[1.05,1]]);
const stampPress=P([[0,0]],[[0,0],[.42,0],[.52,.6,'hit'],[.82,0]]);
const slashEnd=(t,s)=>[mix(21,11,strike(t,s)),mix(9.5,22.5,strike(t,s))];
const discount=policy(make('discount','Discount','Light retracts and restores the percent slash. Strong retracts almost fully, pauses, then strikes the slash in a short decisive stroke. Its dots remain fixed within the badge.',posed([
 ...surface('badge',rect(5,5,22,22,4),[circle(11.5,11.5,2.4),circle(20.5,20.5,2.4),(t,s)=>capsule([21,9.5],slashEnd(t,s),2.6)]),stroke('dot-one',circle(11.5,11.5,2.2)),stroke('dot-two',circle(20.5,20.5,2.2)),stroke('slash',(t,s)=>`M21 9.5 L${slashEnd(t,s)}`)
],rigid((t,s)=>({dy:stampPress(t,s)}))),.78,1.33));discount.constraints=[{type:'rigid',part:'discount-badge'}];
const tear=bidir([[0,0],[.03,0],[.7,1,'out']],[[0,0],[.14,0,'launch'],[.46,2.2],[.76,.85],[1.08,1]],[[0,1],[.65,0,'smooth']],[[0,1],[.08,1],[.38,-.12],[.64,0]]);
const couponTug=P([[0,0]],[[0,0],[.03,0],[.28,-.6],[.78,0]]);
const couponD='M7 8 L16.5 8 C17.605 8 18.5 8.895 18.5 10 L18.5 22 C18.5 23.105 17.605 24 16.5 24 L7 24 C5.895 24 5 23.105 5 22 L5 17.5 C5.828 17.5 6.5 16.828 6.5 16 C6.5 15.172 5.828 14.5 5 14.5 L5 10 C5 8.895 5.895 8 7 8 Z';
const coupon=policy(held(make('coupon','Coupon','Light removes the redeemable stub. Strong tugs against the coupon, tears the stub upward with visible extra travel, and settles at the same redeemed position.',[
 ...posed([...surface('body',couponD,[circle(9,12,1.1),circle(15,20,1.1),capsule([15,11.5],[9,20.5],2.6)]),fill('dot-one',circle(9,12,1.1),'outline'),fill('dot-two',circle(15,20,1.1),'outline'),stroke('percent','M15 11.5 L9 20.5')],rigid((t,s,r)=>({dx:r?0:couponTug(t,s)}))),
 ...posed(surface('stub',rect(22.5,8,4.5,16,1.5)),rigid((t,s,r)=>({dx:.5*Math.min(1,tear(t,s,r)),dy:-1.4*tear(t,s,r),angle:-3*tear(t,s,r),pivot:[22.5,24]})))
],.78,1.33),'attached','redeemed',.73,.88));coupon.constraints=[{type:'rigid',part:'coupon-body'},{type:'rigid',part:'coupon-stub'}];
const earned=bidir([[0,0],[.15,0],[.65,1,'out']],[[0,0],[.39,0],[.55,1,'out']],[[0,1],[.4,0,'smooth']],[[0,1],[.45,0,'smooth']]);
const stampY=P([[0,0],[.03,0],[.2,1.6,'in'],[.55,0,'out']],[[0,0],[.06,0],[.2,-1.1,'launch'],[.4,3.3,'hit'],[.64,-.7],[1.05,0]]);
const loyalty=policy(held(make('loyalty-points','Loyalty points','Light presses the stamp quietly. Strong lifts it first and drives it onto the card with a larger stroke. The earned point remains after the stamp returns.',[
 ...surface('card',rect(4.5,12,23,14,2.5),[circle(11,20,3.6),circle(21,20,3.6)]),stroke('cell-one',circle(11,20,2.5)),stroke('cell-two',circle(21,20,2.5)),...bothFill('earned-one',circle(11,20,1.85)),...bothFill('earned-two',(t,s,r)=>circle(21,20,1.85*earned(t,s,r)),{visibility:(t,s,r)=>earned(t,s,r)>.00001?1:0}),
 ...posed([...bothFill('stamp-grip',rect(18.75,5.1,4.5,2.2,.7)),...ink('stamp-stem','M21 5.5 L21 8.5'),...ink('stamp-foot','M18.5 8.5 L23.5 8.5')],rigid((t,s,r)=>({dy:r?0:stampY(t,s)})))
],.78,1.33),'one-point','two-points',.48,.53));loyalty.constraints=[{type:'fixed',part:'loyalty-points-card'},{type:'rigid',part:'loyalty-points-stamp-foot'}];
const cinch=P([[0,0],[.03,0],[.35,10,'out'],[.7,0,'smooth']],[[0,0],[.03,0],[.18,6,'launch'],[.5,-22],[.82,3],[1.2,0]]);
const leftLoop='M16 12 C13.5 11 9.5 9.5 10 7 C10.5 4.5 15.5 6.5 16 12 Z',rightLoop='M16 12 C18.5 11 22.5 9.5 22 7 C21.5 4.5 16.5 6.5 16 12 Z';
const giftCard=policy(make('gift-card','Gift card','Light cinches the bow once. Strong prepares the loops, opens them through a wide outward rotation and lets the ribbon settle around its fixed knot.',[
 ...surface('card',rect(5,12,22,13,2.5),[capsule([16,15.5],[16,21.5],2.6)]),stroke('ribbon','M16 12 L16 25'),...posed(surface('loop-left',leftLoop),rigid((t,s)=>({angle:-cinch(t,s),pivot:[16,12]}))),...posed(surface('loop-right',rightLoop),rigid((t,s)=>({angle:cinch(t,s),pivot:[16,12]}))),...bothFill('knot',circle(16,12,1.1))
],.78,1.33));giftCard.constraints=[{type:'fixed',part:'gift-card-card'},{type:'rigid',part:'gift-card-loop-left'}];
const cycle=P([[0,0],[.03,0],[.7,360,'smooth']],[[0,0],[.03,0],[.18,-30,'out'],[.72,735,'out'],[1.1,720,'smooth']]);
const parcelY=P([[0,0]],[[0,0]]);
const subscription=policy(make('subscription-box','Subscription box','Light advances the delivery dial once. Strong takes up its hand, makes two quick full turns, then catches the next equivalent index. The parcel stays fixed.',posed([
 ...surface('body',rect(5,6.5,22,21,2),[circle(11.5,9.2,1.2),circle(16,18,5.8)]),stroke('seam','M11.5 6.5 L11.5 9'),stroke('dial',circle(16,18,5.8)),
 ...posed(ink('hand','M16 15.75 L16 18 L18.5 18'),rigid((t,s)=>({angle:cycle(t,s),pivot:[16,18]})))
],rigid((t,s)=>({dy:parcelY(t,s)}))),.78,1.33));
// Preserve the hand's independent rigid rotation while the sealed parcel settles.
for(const p of subscription.parts.filter(p=>p.name.includes('-hand'))){p.pose=rigid((t,s)=>({angle:cycle(t,s),pivot:[16,18],dy:parcelY(t,s)}));}
subscription.cycleDegrees=360;
subscription.constraints=[{type:'rigid',part:'subscription-box-body'},{type:'rigid',part:'subscription-box-hand'}];
export const icons=[discount,coupon,loyalty,giftCard,subscription];
