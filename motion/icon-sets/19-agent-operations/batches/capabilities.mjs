import{toggle,tube,stroke,fill,bothFill,framed,circle,rect,polygon,capsule,annulus,hole,rigid,curve}from'./helpers.mjs';
const dockY=(t,s,r)=>r?0:curve(t,s,[[0,0]],[[0,0],[.36,0],[.46,.6],[.7,0]]);
const cartridgeY=(t,s,r)=>r?curve(t,false,[[0,0],[.06,0],[.32,-3.2,'out']],[]):curve(t,s,[[0,-3.2],[.1,-3.2],[.52,0,'in']],[[0,-3.2],[.16,-3.6],[.36,0,'hit'],[.46,.6],[.7,0]]);
const skills=toggle('agent-skills','Agent skills','A procedure cartridge seats into its dock. Strong prepares the cartridge, lands it decisively and lets the dock receive the impact. Eject restores the detached state.',[
 ...framed('dock',rect(6,19,20,7,2.5),rect(6,19.5,20,6.5,2.5),{pose:rigid((t,s,r)=>({dy:dockY(t,s,r)}))}),...framed('cartridge',rect(9,7,14,7,2),undefined,{pose:rigid((t,s,r)=>({dy:cartridgeY(t,s,r)}))}),fill('tab',rect(13,13.5,6,6,.5),'outline',{pose:rigid((t,s,r)=>({dy:cartridgeY(t,s,r)}))}),fill('tab-solid',rect(13,13,6,7.5,.5),'solid',{pose:rigid((t,s,r)=>({dy:cartridgeY(t,s,r)}))})
],.52,.7,'Detached','Loaded',.32,.32);skills.duotoneFill=true;
const lensX=(t,s,r)=>r?curve(t,false,[[0,23.5],[.1,23.5],[.42,8.5,'out']],[]):curve(t,s,[[0,8.5],[.08,8.5],[.62,23.5,'out']],[[0,8.5],[.12,7.9],[.4,23.5,'hit']]);
const lensY=(t,s,r)=>!r&&s?curve(t,true,[],[[0,0],[.4,0],[.48,.7],[.6,0]]):0;
const lensPose=rigid((t,s,r)=>({dx:lensX(t,s,r)-8.5,dy:lensY(t,s,r)}));
const candidateY=(t,s,r)=>r?curve(t,false,[[0,-2.5],[.18,0,'out']],[]):curve(t,s,[[0,0],[.62,0],[.88,-2.5,'out']],[[0,0],[.62,0],[.78,-2.5]]);
const candidatesAlpha=(t,s,r)=>r?curve(t,false,[[0,.45],[.18,1]],[]):curve(t,s,[[0,1],[.62,1],[.88,.45]],[[0,1],[.62,1],[.78,.45]]);
const discovery=toggle('tool-discovery','Tool discovery','A magnifier searches across candidate tools and lifts the selected result. Strong scans faster with a short braking response; clear selection restores the candidates.',[
 fill('glass',circle(8.5,13,4.5),'tint',{pose:lensPose}),stroke('lens',circle(8.5,13,4.5),{pose:lensPose}),fill('lens-solid',annulus(8.5,13,4.5,2.6),'solid',{pose:lensPose}),...tube('handle','M10.95 8.76 L13.5 4.34',{pose:lensPose}),...[8.5,16,23.5].flatMap((x,i)=>bothFill('candidate-'+i,circle(x,24.5,1.7),{pose:rigid((t,s,r)=>({dy:i===2?candidateY(t,s,r):0})),visibility:(t,s,r)=>i===2?1:candidatesAlpha(t,s,r)}))
],.88,.78,'Searching','Found',.42,.42);discovery.duotoneFill=true;
const plugX=(t,s,r)=>r?curve(t,false,[[0,3],[.05,3],[.32,0,'out']],[]):curve(t,s,[[0,0],[.1,0],[.55,3,'in']],[[0,0],[.14,-.7],[.36,3,'hit'],[.46,3.5],[.7,3]]);
const housingX=(t,s,r)=>!r&&s?curve(t,true,[],[[0,0],[.36,0],[.46,.5],[.7,0]]):0;
const plugPose=rigid((t,s,r)=>({dx:plugX(t,s,r)}));
const mcp=toggle('mcp-tools','MCP tools','A two-prong interface seats into its tool port. Strong pulls back, inserts faster and moves the receiving housing with the contact. Disconnect stays fully opaque.',[
 ...framed('housing',rect(19.5,8,8.5,16,2.5),rect(20,8,8,16,2.5),{pose:rigid((t,s,r)=>({dx:housingX(t,s,r)}))}),...framed('plug',rect(4,11,8,10,2.5),undefined,{pose:plugPose}),...[13.5,18.5].flatMap((y,i)=>[stroke('prong-'+i,`M12.2 ${y} L15.6 ${y}`,{pose:plugPose}),fill('prong-'+i+'-solid',capsule([12.2,y],[16.4,y],2.6),'solid',{pose:plugPose})])
],.55,.7,'Detached','Attached',.32,.32);mcp.duotoneFill=true;
const pointerX=(t,s,r)=>r?curve(t,false,[[0,9.5],[.1,9.5],[.45,19.5,'out']],[]):curve(t,s,[[0,19.5],[.08,19.5],[.55,9.5,'out']],[[0,19.5],[.12,19.5],[.3,9.5,'hit']]);
const press=(t,s,r)=>r?0:curve(t,s,[[0,0],[.55,0],[.62,.3],[.74,0]],[[0,0],[.3,0],[.38,.5],[.54,0]]);
const pointerY=(t,s,r)=>r?curve(t,false,[[0,15.6],[.1,15.6],[.45,14,'out']],[]):s?curve(t,true,[],[[0,14],[.12,13.7],[.3,15.6,'hit'],[.38,16.1],[.54,15.6]]):curve(t,false,[[0,14],[.08,14],[.55,15.6,'out']],[])+press(t,s,r);
const resultAlpha=(t,s,r)=>r?curve(t,false,[[0,1],[.1,0]],[]):curve(t,s,[[0,0],[.6,0],[.8,1,'out']],[[0,0],[.38,0],[.56,1]]);
const browser=toggle('browser-agent','Browser agent','The cursor crosses the page, presses a button and reveals its result. Strong has a much faster main approach and a firmer click. Preview reset clears the result before the cursor returns.',[
 fill('page',rect(4,5,24,22,2.5),'tint'),stroke('window',rect(4,5,24,22,2.5)),stroke('tabbar','M4 10 L28 10'),fill('window-solid',hole(rect(4,5,24,22,2.5),rect(6,12.5,20,12,1.5))),...bothFill('button',capsule([8.5,16],[12,16],2.6),{pose:rigid((t,s,r)=>({dy:press(t,s,r)}))}),...bothFill('result',capsule([17.5,21],[23.5,21],2.6),{visibility:resultAlpha}),...bothFill('pointer',polygon([[0,0],[0,6.2],[1.7,4.7],[3.1,7.2],[4.7,6.4],[3.3,3.9],[5.4,3.9]],.6),{pose:rigid((t,s,r)=>({dx:pointerX(t,s,r),dy:pointerY(t,s,r)}))})
],.8,.56,'Ready','Clicked',.5,.5);Object.assign(browser,{duotoneFill:true,previewReset:true,reverseSuffix:'-reset',reverseLabel:'Reset preview'});
export const capabilities={'agent-skills':skills,'tool-discovery':discovery,'mcp-tools':mcp,'browser-agent':browser};
