# Emoji

Source set `64-emoji` in the Emoji category: the 20 most-used emoji, chosen from Unicode's emoji frequency ranking (2021). Heart suit (♥️) and smiling face (☺️) draw the same as red heart and smiling face with smiling eyes in a one-colour set, so the next two in the ranking take their places. Every icon is a spec (`specs/<id>.icon.json`) added with `generative-icons library adopt 64 --add`; `definitions.mjs` defines none.

## Icons

- Face with tears of joy (`face-with-tears-of-joy`) 😂, Unicode rank 1
- Red heart (`red-heart`) ❤️, Unicode rank 2
- Rolling on the floor laughing (`rolling-on-the-floor-laughing`) 🤣, Unicode rank 3
- Thumbs up (`thumbs-up`) 👍, Unicode rank 4
- Loudly crying face (`loudly-crying-face`) 😭, Unicode rank 5
- Folded hands (`folded-hands`) 🙏, Unicode rank 6
- Face blowing a kiss (`face-blowing-a-kiss`) 😘, Unicode rank 7
- Smiling face with hearts (`smiling-face-with-hearts`) 🥰, Unicode rank 8
- Smiling face with heart-eyes (`smiling-face-with-heart-eyes`) 😍, Unicode rank 9
- Smiling face with smiling eyes (`smiling-face-with-smiling-eyes`) 😊, Unicode rank 10
- Party popper (`party-popper`) 🎉, Unicode rank 11
- Beaming face with smiling eyes (`beaming-face-with-smiling-eyes`) 😁, Unicode rank 12
- Two hearts (`two-hearts`) 💕, Unicode rank 13
- Pleading face (`pleading-face`) 🥺, Unicode rank 14
- Grinning face with sweat (`grinning-face-with-sweat`) 😅, Unicode rank 15
- Fire (`fire`) 🔥, Unicode rank 16
- Person facepalming (`person-facepalming`) 🤦, Unicode rank 18
- Person shrugging (`person-shrugging`) 🤷, Unicode rank 20
- Face with rolling eyes (`face-with-rolling-eyes`) 🙄, Unicode rank 21
- Grinning squinting face (`grinning-squinting-face`) 😆, Unicode rank 22

## Drawing

The faces share one construction: a circle of radius 11 at the canvas centre, dot eyes of radius 1.3 at x 12 and 20, smiling eyes as arcs, and one open-grin and one smile shape. A face shrinks or moves only where the emoji needs room outside it (the kiss, the hearts, the hand, the shrug). Every heart is the library heart contour, scaled. Solid is the filled face with its features cut out; tears, hands and hearts on the rim are drawn filled with a gap cut in the face around them.

## Motion and states

Every icon is a gesture: one reaction that ends on its rest pose, the emoji as people know it. Light plays the event once; Strong adds anticipation, travel and beats. In Solid a face can move only one group of cut-out features against itself, so a few icons keep a feature still in Solid that moves in Outline and Duotone (the design record lists them).
