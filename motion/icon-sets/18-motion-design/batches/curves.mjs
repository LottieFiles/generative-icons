import{make,toggle,tube,stroke,fill,bothFill,framed,rect,circle,rigid,curve}from'./helpers.mjs';
const p=(t,s)=>curve(t,s,[[0,0],[.6,1,'out']],[[0,0],[.08,-.15],[.5,1.35,'pass'],[.78,1]]);
const graph=make('graph-editor','Graph editor','A value graph changes from a straight interpolation to an eased curve. Strong bends against the change briefly, then makes a deeper, faster curve adjustment.',[
 ...tube('axes','M5 5 L5 27 L27 27'),...tube('curve',(t,s)=>{const u=p(t,s),v=Math.min(u,1);return`M10 22 C${14.67+4.33*u} ${17.33+4.67*v} ${19.33-4.33*u} ${12.67-4.67*v} 24 8`;}),
 ...[[10,22],[24,8]].flatMap(([x,y],i)=>[fill('key-'+i,circle(x,y,1.6),'outline'),fill('key-'+i+'-solid',circle(x,y,2))])
],.6,.78,['curve']);graph.duotoneFill=false;
const theta=(t,s)=>curve(t,s,[[0,0],[.45,28,'out'],[.75,28,'hold'],[1.2,0]],[[0,0],[.12,-5],[.5,44,'pass'],[.68,38],[.95,38,'hold'],[1.5,0]]),handlePose=rigid((t,s)=>({angle:theta(t,s),pivot:[16,16]}));
const bezier=make('bezier-handles','Bezier handles','A tangent handle turns and changes the curve at its anchor. Strong uses a backswing, a wider turn and one controlled return.',[
 ...tube('curve',(t,s)=>{const a=theta(t,s)*Math.PI/180,c=3*Math.cos(a),d=3*Math.sin(a);return`M7 25 C7 25 ${16-c} ${16-d} 16 16 C${16+c} ${16+d} 25 7 25 7`;}),...tube('handle-bar','M8 16 L24 16',{pose:handlePose}),
 ...[8,24].flatMap((x,i)=>[fill('handle-'+i,circle(x,16,1.5),'outline',{pose:handlePose}),fill('handle-'+i+'-solid',circle(x,16,1.9),'solid',{pose:handlePose})]),fill('anchor',rect(14.3,14.3,3.4,3.4,.6),'outline'),fill('anchor-solid',rect(14.1,14.1,3.8,3.8,.6))
],1.2,1.5);bezier.duotoneFill=false;
const q=(t,s,r)=>r?curve(t,false,[[0,8],[.4,8],[.85,16,'out']],[]):curve(t,s,[[0,16],[.45,8,'out']],[[0,16],[.1,17.2],[.4,8,'hit']]);
const rot=(t,s,r)=>r?curve(t,false,[[0,-18],[.4,0]],[]):curve(t,s,[[0,0],[.45,0],[1,-18]],[[0,0],[.4,0],[.72,-21,'pass'],[.95,-18]]);
const anchor=toggle('anchor-point','Anchor point','The anchor moves along the layer rim and the layer turns around that new pivot. Strong pulls back before the move and turns the layer farther before settling.',[
 ...framed('layer',rect(8,9,16,14,2),undefined,{pose:rigid((t,s,r)=>({angle:rot(t,s,r),pivot:[8,9]}))}),
 ...tube('cross-h','M13 9 L19 9',{pose:rigid((t,s,r)=>({dx:q(t,s,r)-16}))}),...tube('cross-v','M16 6 L16 12',{pose:rigid((t,s,r)=>({dx:q(t,s,r)-16}))}),
 fill('anchor',circle(16,9,1.7),'outline',{pose:rigid((t,s,r)=>({dx:q(t,s,r)-16}))}),fill('anchor-solid',circle(16,9,2),'solid',{pose:rigid((t,s,r)=>({dx:q(t,s,r)-16}))})
],1,.95,'Centered anchor','Corner anchor',.85,.85);anchor.duotoneFill=true;
const stack=make('layer-stack','Layer stack','The top layer lifts for inspection while the supporting layers move down slightly. Strong pulls farther, lifts higher and adds a controlled tilt.',[0,1,2].flatMap(i=>framed('layer-'+i,rect(7,6.5+i*8,18,4.5,2.25),undefined,{pose:rigid((t,s)=>i?({dy:s?curve(t,true,[],[[0,0],[.12,0],[.4,.6],[.75,.6,'hold'],[1.1,0]]):0}):({dx:curve(t,s,[[0,0],[.35,1.5,'out'],[.6,1.5,'hold'],[.95,0]],[[0,0],[.1,-1],[.38,3,'pass'],[.52,2.6],[.75,2.6,'hold'],[1.1,0]]),dy:curve(t,s,[[0,0],[.35,-2.5,'out'],[.6,-2.5,'hold'],[.95,0]],[[0,0],[.1,0],[.38,-3.2,'pass'],[.52,-2.8],[.75,-2.8,'hold'],[1.1,0,'hit']]),angle:s?curve(t,true,[],[[0,0],[.1,0],[.38,5],[.75,5,'hold'],[1.1,0]]):0,pivot:[7,8.75]}))})),.95,1.1);stack.duotoneFill=true;
export const curves={'graph-editor':graph,'bezier-handles':bezier,'anchor-point':anchor,'layer-stack':stack};
