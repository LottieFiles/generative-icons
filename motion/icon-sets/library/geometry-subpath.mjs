// Select an explicit real contour from paths that join holes with retraced zero-area seams.
// [start,end) command indexes are checked in each icon's geometry contract.
export function subpathRange(commands,range){
 if(!Array.isArray(range)||range.length!==2||range.some(n=>!Number.isInteger(n))||range[0]<0||range[1]>commands.length||range[0]>=range[1])throw Error('Invalid subpath range');
 const first=commands[range[0]];if(![0,1,'M','L'].includes(first))throw Error('Subpath must start with a point');
 return range;
}
export function sceneSubpath(node,range){
 if(range==null)return node;
 const[start,end]=subpathRange(node.commands,range),width=c=>c===2?6:c===0||c===1?2:0;
 const offset=node.commands.slice(0,start).reduce((n,c)=>n+width(c),0),length=node.commands.slice(start,end).reduce((n,c)=>n+width(c),0),commands=node.commands.slice(start,end);commands[0]=0;
 return {...node,commands,points:node.points.map(([t,p])=>[t,p.slice(offset,offset+length)])};
}
