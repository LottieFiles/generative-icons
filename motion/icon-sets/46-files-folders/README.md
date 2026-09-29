# Files & folders

Source set `46-files-folders` in the Files & folders gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-files-folders`.

## Icons

- New folder (`new-folder`)
- Folder open (`folder-open`)
- Move file (`move-file`)
- Rename file (`rename-file`)
- Duplicate file (`duplicate-file`)
- Restore file (`restore-file`)
- Approve file (`approve-file`)
- Lock file (`lock-file`)
- File search (`file-search`)
- File preview (`file-preview`)
- File tag (`file-tag`)
- File star (`file-star`)
- File sync (`file-sync`)
- Import file (`import-file`)
- Export file (`export-file`)
- Image file (`image-file`)
- Code file (`code-file`)
- Spreadsheet file (`spreadsheet-file`)
- Audio file (`audio-file`)
- Video file (`video-file`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/46-files-folders
node motion/icon-sets/46-files-folders/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/46/assets/set.json)
