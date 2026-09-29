// Model training & fine-tuning. Per-icon design and review: evidence/design-review.md.
import {icons as a} from './batches/optimization.mjs';
import {icons as b} from './batches/data.mjs';
import {icons as c} from './batches/training.mjs';
import {icons as d} from './batches/outputs.mjs';
const byName=new Map([...a,...b,...c,...d].map(i=>[i.name,i]));
const ORDER=['epoch-counter','loss-curve','learning-rate','gradient-step','overfitting',
 'validation-split','data-augmentation','data-labeling','gpu-cluster','hyperparameter',
 'early-stopping','distillation','adapter-layer','reward-model','benchmark-score',
 'tokenizer','quantize','pretrained-weights','data-shuffle','convergence'];
if(ORDER.length!==20||byName.size!==20||ORDER.some(n=>!byName.has(n)))throw Error('Model training set requires exactly 20 complete definitions');
const fixed={
 'epoch-counter':['counter-frame','tally'],'loss-curve':['axes'],
 'learning-rate':['initial-step-frame'],'gradient-step':['stairs','parameter-frame'],
 'overfitting':['plot-frame','sample-0','sample-1','sample-2'],
 'data-augmentation':['source-frame','source-mark'],'data-labeling':['record-frame'],
 'early-stopping':['axes','plateau'],'distillation':['teacher-filter-frame','student-frame'],
 'adapter-layer':['base-upper-frame','base-lower-frame'],'reward-model':['candidate-left-frame','candidate-right-frame'],
 'benchmark-score':['baseline-frame','benchmark'],'tokenizer':['word'],
 'quantize':['level-0-frame','level-1-frame','level-2-frame'],
 'pretrained-weights':['shelf','load-guide'],'convergence':['target-frame']
};
const rigidParts={
 'validation-split':['partition-0-frame','partition-1-frame','partition-2-frame'],
 'data-augmentation':['augmented-frame'],'data-labeling':['tag-frame'],
 'gpu-cluster':['processor-0-frame','processor-1-frame','processor-2-frame','processor-3-frame'],
 'hyperparameter':['handle-0-frame','handle-1-frame','handle-2-frame'],
 'early-stopping':['stop-block-frame'],'adapter-layer':['adapter-frame','adapter-pin'],
 'reward-model':['reward-frame'],'pretrained-weights':['cartridge-frame'],
 'data-shuffle':['packet-0-frame','packet-1-frame','packet-2-frame']
};
export const icons=ORDER.map(n=>byName.get(n)).map((icon,index)=>({...icon,number:1046+index,
 constraints:[...(fixed[icon.name]||[]).map(p=>({type:'fixed',part:icon.name+'-'+p})),
 ...(rigidParts[icon.name]||[]).map(p=>({type:'rigid',part:icon.name+'-'+p}))]}));
