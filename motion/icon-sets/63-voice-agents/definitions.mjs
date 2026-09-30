import {icons as dialogue} from './batches/dialogue.mjs';
import {icons as understanding} from './batches/understanding.mjs';
import {icons as telephony} from './batches/telephony.mjs';
import {icons as operations} from './batches/operations.mjs';

const fixed={
  'turn-taking':['caller','agent'], 'barge-in':['agent','caller'],
  'end-of-turn-detection':['utterance','speech-start','deadline'],
  'backchannel':['utterance','phrase'], 'speaker-diarization':['lane-a','lane-b'],
  'live-transcript':['transcript','line-one','line-two'],
  'call-summary':['utterance-a','utterance-b','branch-a','branch-b','summary'],
  'sentiment-meter':['face','sentiment-scale'], 'intent-confirmation':['confirmation'],
  'language-detection':['language','letter','letter-crossbar'],
  'silence-timeout':['silence','quiet','clock','short-hand'],
  'recording-consent':['record-ring','permission-line'],
  'ivr-menu':['trunk','left-branch','right-branch','option-one','option-two'],
  'speech-buffer':['feed','speech-output'], 'escalation-flag':['pole','call-issue'],
  'voicemail-detection':['greeting','detection-eye']
};
export const icons=[...dialogue,...understanding,...telephony,...operations].map((icon,i)=>({
  ...icon,number:1086+i,
  constraints:(fixed[icon.name]||[]).map(part=>({type:'fixed',part:icon.name+'-'+part}))
}));
if(icons.length!==20||new Set(icons.map(i=>i.name)).size!==20)throw Error('Voice agents requires exactly 20 distinct icons');
if(icons.filter(i=>i.stateful).length!==10)throw Error('Voice agents requires ten authored two-state icons');
