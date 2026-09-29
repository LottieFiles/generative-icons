# Education

Source set `38-education` in the Education gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-education`.

## Icons

- Graduation cap (`graduation-cap`)
- Chalkboard (`chalkboard`)
- Flashcards (`flashcards`)
- Quiz (`quiz`)
- Certificate (`certificate`)
- Notebook (`notebook`)
- Pencil (`pencil`)
- Ruler (`ruler`)
- Lab flask (`lab-flask`)
- Microscope (`microscope`)
- Abacus (`abacus`)
- School bell (`school-bell`)
- Course progress (`course-progress`)
- Lecture (`lecture`)
- Library shelf (`library-shelf`)
- Language learning (`language-learning`)
- Alphabet blocks (`alphabet-blocks`)
- Homework (`homework`)
- Exam grade (`exam-grade`)
- Geometry compass (`geometry-compass`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/38-education
node motion/icon-sets/38-education/verify.mjs
node motion/icon-sets/38-education/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/38/assets/set.json)
