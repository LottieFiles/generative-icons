import{continuous,tube,TAU,cycle,field}from'./helpers.mjs';
export const activity=continuous('voice-activity','Voice activity','Five level bars carry a left-to-right voice signal. Strong has almost twice the excursion and two crests per cycle.',Array.from({length:5},(_,i)=>tube('bar-'+i,(t,s)=>field(6+5*i,25,9.5+(s?7.5:4)*Math.sin(TAU*(s?2:1)*cycle(t)-i*TAU/5)))).flat());
activity.constraints=Array.from({length:5},(_,i)=>({type:'inside',part:'voice-activity-bar-'+i,box:[6+5*i,8,6+5*i,25]}));
