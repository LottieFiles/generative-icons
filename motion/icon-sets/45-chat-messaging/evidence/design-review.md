# Chat & messaging design and review

Exactly 20 meanings. Replaced typing-indicator, reply-message, open-thread and new-chat to avoid the existing typing dots, thread reply, reopen action and add-comment concepts. Search evidence: meaning-search.json and family-neighbors.json.

All drawings use round native strokes and real enclosed tint surfaces. Solid borders preserve interior detail. Physical contacts and clearance are checked on exported world geometry. No catalog or other set changes are authorized.

## composing-message

Compose → the insertion caret advances along a text row → it returns to the insertion point → the draft is ready. Light: a 3-unit sweep. Strong: 5 units with a .3-unit preparation and brisk main stroke.

Policy: gesture, shared initial and final drawing. Active seconds: Light 0.86; Strong 0.86. Named parts: composing-message-bubble, composing-message-text, composing-message-lower-text, composing-message-caret. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## message-sent

Send acknowledgement → a single check enters the receipt lane → it seats → sent is retained. Light: a 2-unit rise over .64 s. Strong: a 3-unit rise with a decisive .30 s arrival. Reverse withdraws the check while the message stays.

Policy: Pending → Sent; authored reverse. Active seconds: Light 0.8; Strong 0.72. Named parts: message-sent-bubble, message-sent-text, message-sent-check. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## message-delivered

Recipient receives → the second receipt check joins the first → both align → delivered is retained. Light: a 3-unit leftward seat. Strong: a 4.5-unit snap with delayed release. Reverse removes only the second receipt.

Policy: Sent → Delivered; authored reverse. Active seconds: Light 0.8; Strong 0.72. Named parts: message-delivered-bubble, message-delivered-text, message-delivered-first-check, message-delivered-second-check. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## message-read

Reader opens → the eye opens in the receipt lane → its pupil appears → read is retained. Light: the lid opens smoothly. Strong: the lid opens decisively after a short wait. Reverse closes the lid and removes the pupil.

Policy: Unread → Read; authored reverse. Active seconds: Light 0.8; Strong 0.72. Named parts: message-read-bubble, message-read-text, message-read-eye, message-read-pupil. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## message-reaction

React → a heart rises below the message → it settles beside the tail → reaction stays. Light: a 1.6-unit rise. Strong: a 2.4-unit rise and a 14-degree turn. Reverse lowers and removes the heart.

Policy: No reaction → Reacted; authored reverse. Active seconds: Light 0.8; Strong 0.72. Named parts: message-reaction-bubble, message-reaction-text, message-reaction-heart. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## quoted-message

Quote selection → paired quote marks indent toward the quoted text → the inset line follows → quote rests in its source bubble. Light: 1.4 units. Strong: a 3.2-unit indent and one bounded response.

Policy: gesture, shared initial and final drawing. Active seconds: Light 0.8; Strong 0.8. Named parts: quoted-message-bubble, quoted-message-quote-left, quoted-message-quote-left-tail, quoted-message-quote-right, quoted-message-quote-right-tail, quoted-message-quotation. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## forward-message

Forward command → a right arrow travels away from the source bubble → the route stretches → it rests ready for another forward. Light: 2 units right. Strong: one-unit pullback and a 4-unit drive.

Policy: gesture, shared initial and final drawing. Active seconds: Light 0.8; Strong 0.8. Named parts: forward-message-bubble, forward-message-text, forward-message-arrow-shaft, forward-message-arrow-head. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## pin-message

Pin command → the pin descends into the upper edge of the chat → its needle seats → pinned is retained. Light: 2 units of insertion. Strong: 2 units with an 18-degree preparation. Reverse lifts the pin out.

Policy: Unpinned → Pinned; authored reverse. Active seconds: Light 0.8; Strong 0.72. Named parts: pin-message-bubble, pin-message-text, pin-message-pin-head, pin-message-needle. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## conversation-history

Browse earlier conversation → stacked transcript rows move upward beneath a fixed top bubble → the history cue stays in place → the transcript returns. Light: a 2-unit scroll. Strong: a 4-unit scroll with preparation.

Policy: gesture, shared initial and final drawing. Active seconds: Light 0.86; Strong 0.9. Named parts: conversation-history-top-message, conversation-history-history-rail, conversation-history-older-row, conversation-history-oldest-row. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## mute-chat

Mute command → a diagonal strike spans the conversation → notifications stop → muted stays. Light: the stroke wipes in over .64 s. Strong: a fast .30 s wipe after .10 s preparation. Reverse retracts the strike.

Policy: Audible → Muted; authored reverse. Active seconds: Light 0.8; Strong 0.72. Named parts: mute-chat-bubble, mute-chat-slash. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## group-chat

Members speak → two participant portraits respond in turn under a common conversation → the shoulders follow their own heads → the group settles. Light: 1.1-unit nods. Strong: 2.4-unit staggered lifts.

Policy: gesture, shared initial and final drawing. Active seconds: Light 0.8; Strong 0.86. Named parts: group-chat-bubble, group-chat-member-a, group-chat-shoulders-a, group-chat-member-b, group-chat-shoulders-b. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## voice-note

Play recorded note → three waveform bars articulate around a fixed baseline → the recorded clip remains in its pill → playback rests. Light: shallow staggered amplitude. Strong: a larger travelling waveform. This is a saved clip, distinct from live Voice chat.

Policy: gesture, shared initial and final drawing. Active seconds: Light 0.82; Strong 0.9. Named parts: voice-note-clip, voice-note-play, voice-note-wave-0, voice-note-wave-1, voice-note-wave-2. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## sticker

Pick sticker → its lower corner peels back → the diagonal fold follows the edge → the sticker lies flat again. Light: 1-unit peel. Strong: 2-unit peel with a short wind-up and settling response.

Policy: gesture, shared initial and final drawing. Active seconds: Light 0.8; Strong 0.86. Named parts: sticker-paper, sticker-fold, sticker-eye-a, sticker-eye-b, sticker-smile. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## gif-picker

Browse GIFs → the preview tile lifts from its stacked tray → the GIF label rides with it → the selected tile settles. Light: 1.1-unit lift. Strong: 2.2-unit lift with a brief preparation.

Policy: gesture, shared initial and final drawing. Active seconds: Light 0.8; Strong 0.8. Named parts: gif-picker-stack, gif-picker-tile, gif-picker-g, gif-picker-i, gif-picker-f, gif-picker-f-bar. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## edit-message

Edit → the pen moves along the message baseline → ink extends exactly to its tip → the edited line is retained. Light: a 7.5-unit writing stroke. Strong: the same meaningful stroke arrives in .30 s after preparation. Reverse erases back along the baseline.

Policy: Original → Edited; authored reverse. Active seconds: Light 0.8; Strong 0.72. Named parts: edit-message-bubble, edit-message-existing-line, edit-message-ink, edit-message-pen, edit-message-pen-cap. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## delete-message

Delete → two message lines retract into a removal mark → the bubble becomes a deleted-message placeholder → deletion is retained. Light: gradual replacement. Strong: fast leftward removal. Reverse restores the message content.

Policy: Message → Deleted; authored reverse. Active seconds: Light 0.8; Strong 0.72. Named parts: delete-message-bubble, delete-message-text-0, delete-message-text-1, delete-message-removed. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## unread-divider

Jump to unread → the unread separator moves down between two message rows → its centre marker leads → the boundary rests. Light: 1.3-unit travel. Strong: 3-unit travel with a short rebound.

Policy: gesture, shared initial and final drawing. Active seconds: Light 0.8; Strong 0.8. Named parts: unread-divider-upper-message, unread-divider-lower-message, unread-divider-divider-l, unread-divider-unread-dot, unread-divider-divider-r. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## archive-chat

Archive → the message slides into the storage tray → the chat remains visible as the stored record → archived is retained. Light: 7 units downward. Strong: a decisive 7-unit delivery with an accelerated middle stroke. Reverse lifts the record out.

Policy: Inbox → Archived; authored reverse. Active seconds: Light 0.8; Strong 0.72. Named parts: archive-chat-tray, archive-chat-tray-slot, archive-chat-message. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## disappearing-message

Inspect expiry → sand runs through the message timer → the lower chamber fills → the expiry control returns ready. Light: a quiet transfer. Strong: the timer turns 12 degrees while sand transfers faster. This previews the duration setting, not message deletion.

Policy: gesture, shared initial and final drawing. Active seconds: Light 0.92; Strong 0.98. Named parts: disappearing-message-bubble, disappearing-message-timer, disappearing-message-sand. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.

## chat-search

Search transcript → the lens sweeps beneath the message row → its handle follows rigidly → the lens returns to its centred result. Light: a 2.6-unit scan. Strong: a 5-unit scan with a bounded response.

Policy: gesture, shared initial and final drawing. Active seconds: Light 0.82; Strong 0.92. Named parts: chat-search-bubble, chat-search-text, chat-search-lens, chat-search-handle. Pivots are explicit in the source; unmoving parts use (16,16). Neighbors: message, reply-thread, voice-chat, add-comment (reference only).

Design review: preserve interior counters, visible prepared housing, common endpoints, and useful Strong travel. Initial proposal reviewed; final findings and package hashes are in visual-review.md.


## Native review correction, round 2

Round 1 passed the complete 58-action browser gate. Native pose/style review found crowded caret/border and sticker/smile gaps, an hourglass-like pin, a check-like pen, and weak Solid spacing in the history rail, mute slash and GIF G. Round 2 revises those drawings, gives the archive tray an open top, and includes ai-authoring.mjs in source packaging. No shared files or other sets changed.

Independent checks now cover pen/ink attachment, both sticker-fold contacts, waveform/border clearance, sticker smile/fold clearance and caret/border clearance across both profiles and every exported action. Final native review passed; see visual-review.md, native-review-capture.json and browser-verification.json.
