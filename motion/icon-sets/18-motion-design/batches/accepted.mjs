import{make,tube,stroke,fill,bothFill,framed,rect,polygon,rigid,curve,circle}from'./helpers.mjs';
const head=rigid((t,s)=>({dx:curve(t,s,[[0,0],[.5,12,'out']],[[0,0],[.08,-.6],[.32,13,'pass'],[.48,12]])}));
const timeline=make('timeline','Timeline','A playhead scrubs across two clip tracks. Strong prepares briefly, crosses the same range faster and brakes with one controlled overshoot.',[
 ...tube('ruler','M5 7 L27 7'),...tube('clip-a','M7 13 L15 13'),...tube('clip-b','M12 22 L26 22'),...tube('playhead-guide-a','M10 16.6 L10 18.4',{pose:head}),...tube('playhead-guide-b','M10 25.6 L10 27',{pose:head}),...bothFill('playhead-head',polygon([[7.5,3.5],[12.5,3.5],[12.5,7],[10,10],[7.5,7]],.4),{pose:head})
],.5,.48,['playhead']);timeline.duotoneFill=false;
const route='M6 24 C6 8 26 26 26 8',B=u=>{const v=1-u;return[v*v*v*6+3*v*v*u*6+3*v*u*u*26+u*u*u*26,v*v*v*24+3*v*v*u*8+3*v*u*u*26+u*u*u*8];};
const position=rigid((t,s)=>{const p=B(curve(t,s,[[0,0],[.8,1]],[[0,0],[.1,.02],[.45,.75,'pass'],[.7,1]]));return{dx:p[0]-6,dy:p[1]-24};});
const diamond=(x,y,h)=>polygon([[x,y-h],[x+h,y],[x,y+h],[x-h,y]],.4);
const path=make('motion-path','Motion path','A position marker travels the actual cubic path between two keyframes. Strong lifts off, crosses the route decisively and brakes at the destination.',[
 ...tube('route',route),...[['start',6,24],['end',26,8]].flatMap(([n,x,y])=>[fill(n,diamond(x,y,3.4/Math.SQRT2),'outline'),fill(n+'-solid',diamond(x,y,4/Math.SQRT2))]),fill('traveler',circle(6,24,2.1),'outline',{pose:position}),fill('traveler-solid',circle(6,24,2.5),'solid',{pose:position})
],.8,.7,['traveler']);path.duotoneFill=false;
const progress=(row,t,s)=>row===0?1:row===1?curve(t,s,[[0,.5],[.04,.5],[.56,1,'out']],[[0,.5],[.04,.5],[.30,1]]):curve(t,s,[[0,0],[.50,0],[1,1,'out']],[[0,0],[.22,0],[.62,1]]);
const queue=make('render-queue','Render queue','Render jobs complete in order. Strong fills each job with a faster complete stroke and overlaps the next start with the prior finish.',[0,1,2].flatMap(i=>{const y=9+i*7;return[
 ...bothFill('dot-'+i,circle(6,y,1.5),{visibility:(t,s)=>i<2?1:curve(t,s,[[0,.32],[.44,.32],[.56,1]],[[0,.32],[.18,.32],[.28,1]])}),...tube('track-'+i,`M10 ${y} L26 ${y}`,{visibility:()=>.32}),...tube('progress-'+i,(t,s)=>`M10 ${y} L${10+16*progress(i,t,s)} ${y}`,{visibility:(t,s)=>progress(i,t,s)<.05?0:1})
];}),1,.62,['progress-1','progress-2','dot-2']);queue.duotoneFill=false;
export const accepted={timeline,'motion-path':path,'render-queue':queue};
