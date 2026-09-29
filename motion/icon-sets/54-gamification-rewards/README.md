# Gamification & rewards

Source set `54-gamification-rewards` for the Gamification & rewards category. Inspect native output with the [verification server](../../../docs/verification.md) at `/icons/54/review.html`; the full native check is at `/icons/54/verify.html?save=1`.

## Icons

- Trophy (`trophy`)
- Medal (`medal`)
- Level up (`level-up`)
- XP gain (`xp-gain`)
- Streak flame (`streak-flame`)
- Badge unlock (`badge-unlock`)
- Leaderboard (`leaderboard`)
- Dice (`dice`)
- Game controller (`game-controller`)
- Coin (`coin`)
- Gem (`gem`)
- Treasure chest (`treasure-chest`)
- Quest (`quest`)
- Achievement (`achievement`)
- Star rating (`star-rating`)
- High score (`high-score`)
- Bonus (`bonus`)
- Power up (`power-up`)
- Lives (`lives`)
- Round countdown (`round-countdown`)

Stateful icons: Level up, XP gain, Badge unlock, Leaderboard, Treasure chest, Quest, Star rating, Round countdown. Each includes an explicit reverse or preview reset in both motion profiles. The complete set has 56 native actions.

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/54-gamification-rewards
node motion/icon-sets/54-gamification-rewards/verify.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest. `verify.mjs` includes six drawing-specific contact and clearance checks. After native render verification, run `python3 motion/icon-sets/54-gamification-rewards/package.py` to refresh the download with current evidence.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/54/assets/set.json)
- [Design and review](evidence/design-review.md)
- [Visual review](evidence/visual-review.md)
