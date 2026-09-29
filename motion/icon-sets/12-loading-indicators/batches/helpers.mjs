import {circle,rect,hole,annulus,fill,stroke,tube,rigid,polygon} from '../../library/drawing.mjs';
export {circle,rect,hole,annulus,fill,stroke,tube,rigid,polygon};
export const TAU=Math.PI*2,T=2.4;
export const phase=t=>t/T;
export const sn=u=>Math.sin(TAU*u),cs=u=>Math.cos(TAU*u);
export const point=(r,u,cx=16,cy=16)=>[cx+r*cs(u),cy+r*sn(u)];
export const ring=(name,cx,cy,r,extra={})=>[
 stroke(name,circle(cx,cy,r),extra),
 fill(name+'-tint',circle(cx,cy,r),'tint',extra),
 fill(name+'-solid',annulus(cx,cy,r,2.6),'solid',extra)
];
export const outlineBox=(name,x,y,w,h,r,extra={})=>[
 stroke(name,rect(x,y,w,h,r),extra),fill(name+'-tint',rect(x,y,w,h,r),'tint',extra),
 fill(name+'-solid',hole(rect(x-1.3,y-1.3,w+2.6,h+2.6,r+1.3),rect(x+1.3,y+1.3,w-2.6,h-2.6,Math.max(.15,r-1.3))),'solid',extra)
];
// Four cubic arc segments retain exact morph topology for every span.
export function curvedArc(cx,cy,r,start,span){
 const a=TAU*start,b=TAU*span,p=q=>[cx+r*Math.cos(q),cy+r*Math.sin(q)];let d='M'+p(a);
 for(let i=0;i<4;i++){const u=a+b*i/4,v=a+b*(i+1)/4,k=4/3*Math.tan((v-u)/4),x=p(u),y=p(v);d+=' C'+[x[0]-k*r*Math.sin(u),x[1]+k*r*Math.cos(u),y[0]+k*r*Math.sin(v),y[1]-k*r*Math.cos(v),...y];}return d;
}
export const make=(name,label,intent,parts,brief)=>({name,label,intent,heading:label,description:intent,continuous:true,duotoneFill:parts.some(p=>p.style==='tint'),parts:parts.map(p=>({...p,name:name+'-'+p.name})),timings:{light:{active:T,rest:0,keyPoses:[0,.4,.8,1.2,1.6,2]},strong:{active:T,rest:0,keyPoses:[0,.4,.8,1.2,1.6,2]}},brief,revision:'continuous-1'});
export const mod=u=>u-Math.floor(u);
// A dot is one round-capped native centerline, not a tiny closed counter.
export const dot=(name,r=1,extra={})=>tube(name,`M0 0 L0.001 0`,extra);
export function curve(fn,n=32,from=0,to=1,closed=false){
 const p=fn(from),epsilon=.00001;let d='M'+p;
 const tangent=u=>{const a=fn(u-epsilon),b=fn(u+epsilon);return b.map((x,i)=>(x-a[i])/(2*epsilon));};
 for(let i=0;i<n;i++){const a=from+(to-from)*i/n,b=from+(to-from)*(i+1)/n,x=fn(a),y=fn(b),ta=tangent(a),tb=tangent(b),h=(b-a)/3;d+=' C'+[x[0]+h*ta[0],x[1]+h*ta[1],y[0]-h*tb[0],y[1]-h*tb[1],...y];}return d+(closed?' Z':'');
}
export const at=(fn)=>rigid((t,s)=>{const p=fn(phase(t),s);return{dx:p[0],dy:p[1],pivot:[0,0]};});
export const spin=(turns=1,offset=0)=>rigid((t,s)=>({angle:360*phase(t)*(typeof turns==='function'?turns(s):turns)+offset}));
export function capsulePoint(u){const r=4,a=10,b=22,L=24+8*Math.PI,d=mod(u)*L;
 if(d<12)return[a+d,12];if(d<12+Math.PI*4){const q=(d-12)/4-Math.PI/2;return[b+4*Math.cos(q),16+4*Math.sin(q)];}
 if(d<24+Math.PI*4)return[b-(d-12-Math.PI*4),20];const q=(d-24-Math.PI*4)/4+Math.PI/2;return[a+4*Math.cos(q),16+4*Math.sin(q)];
}
export function squarePoint(u){const L=40+6*Math.PI,z=mod(u)*L,side=10+1.5*Math.PI,k=Math.floor(z/side),d=z-k*side;
 let p;if(d<10)p=[11+d,8];else{const a=-Math.PI/2+(d-10)/3;p=[21+3*Math.cos(a),11+3*Math.sin(a)];}
 const a=k*Math.PI/2,x=p[0]-16,y=p[1]-16;return[16+x*Math.cos(a)-y*Math.sin(a),16+x*Math.sin(a)+y*Math.cos(a)];
}
