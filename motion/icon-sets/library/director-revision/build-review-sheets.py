from pathlib import Path
import json
from PIL import Image,ImageDraw,ImageFont
root=Path(__file__).resolve().parents[4]; records=[]
for sp in json.loads((root/'prototype/icons/catalog.json').read_text())['sets']:
 p=root/'prototype/icons'/sp
 for icon in json.loads(p.read_text())['icons']:records.append((p.parent,icon))
font=ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc',19)
for page in range((len(records) + 7) // 8):
 rows=records[page*8:page*8+8]; sheet=Image.new('RGB',(1280,len(rows)*186),(16,23,29));d=ImageDraw.Draw(sheet)
 for row,(base,icon) in enumerate(rows):
  d.text((12,row*186+5),icon['label'],font=font,fill='#ebf2f4')
  for j,m in enumerate(icon['motions']):
   pic=Image.open(base/(m['name']+'-poses.png')).convert('RGB');pic.thumbnail((620,148));sheet.paste(pic,(12+j*636,row*186+34));d.text((470+j*636,row*186+5),'Strong' if j else 'Light',font=font,fill='#9cb0ba')
 sheet.save(root/f'motion/icon-sets/library/director-revision/evidence/review-{page+1:02}.jpg',quality=92)
