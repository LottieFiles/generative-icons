import {icon,progress,kick,framed,rect,polygon,posed,shown,rigid,tube,fill} from './helpers.mjs';
const a=rect(6,5.5,14,14,1.3),b=rect(12,12.5,14,14,1.3);
// Each selected region is authored directly; no background paint simulates subtraction.
const regions={
 'boolean-union':polygon([[6,5.5],[20,5.5],[20,12.5],[26,12.5],[26,26.5],[12,26.5],[12,19.5],[6,19.5]],.85),
 'boolean-subtract':polygon([[6,5.5],[20,5.5],[20,12.5],[12,12.5],[12,19.5],[6,19.5]],.85),
 'boolean-intersect':rect(12,12.5,8,7,.85),
 'boolean-exclude':[polygon([[6,5.5],[20,5.5],[20,12.5],[12,12.5],[12,19.5],[6,19.5]],.85),polygon([[20,12.5],[26,12.5],[26,26.5],[12,26.5],[12,19.5],[20,19.5]],.85)]
};
const labels=['Boolean union','Boolean subtract','Boolean intersect','Boolean exclude'];
const results=['United region','Subtracted region','Intersection','Excluded overlap'];
export const icons=Object.entries(regions).map(([id,d],j)=>icon(id,labels[j],
 `Two overlapping square operands select the ${['combined area','first area outside the second','shared area','two areas outside their overlap'][j]}. Light: a quiet separation then a smooth selection. Strong: both operands draw apart 1.2 units and snap together before the result holds. Reverse restores both operands with its own timing.`,[
 ...[a,b].flatMap((shape,k)=>posed(shown([
  ...tube('operand-'+k,shape),fill('operand-'+k+'-surface',shape,'tint')
 ],(t,s,r)=>1-.82*progress(t,s,r)),rigid((t,s,r)=>({pivot:k?[19,19.5]:[13,12.5],dx:(k?1:-1)*1.2*kick(t,s),dy:(k?1:-1)*.6*kick(t,s)})))),
 ...(Array.isArray(d)?d:[d]).flatMap((shape,k)=>shown(framed(k?'result-second':'result',shape),(t,s,r)=>progress(t,s,r)))
],{state:['Separate operands',results[j]],fixed:['result-frame']}));
