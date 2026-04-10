# mdue Replacement Plan (`unplugin-icons` + Iconify MDI)

## Why migrate

- `mdue` has not been maintained for years and can become a long-term risk (security, compatibility, ecosystem drift).
- This codebase uses `mdue` broadly (40+ imports across UI components), so migration should be staged to reduce breakage.
- `unplugin-icons` gives on-demand icon components with active maintenance and clean Vue 3 + Vite integration.

## Target stack

- Build/plugin: `unplugin-icons`
- Icon source: `@iconify-json/mdi` (Material Design Icons only)
- Runtime usage style: explicit per-file imports from `~icons/mdi/*`

## Current usage snapshot

- Dependency in `package.json`: `mdue` (`^0.1.4`)
- Estimated impact: ~44 `mdue` icon references across ~36 Vue files under `src/components/`
- Examples of currently used names: `Play`, `Pause`, `Loading`, `Close`, `Magnify`, `ContentCopy`, `DownloadMultiple`, `ArrowLeft`, `Check`, `AlertCircle`, `VolumeMute`, `WindowClose`

## Migration strategy (phased)

### Phase 1 - Add new icon infrastructure (no behavior change)

1. Install dev dependencies:
   - `unplugin-icons`
   - `@iconify-json/mdi`
2. Update `vite.config.js`:
   - add `Icons` plugin from `unplugin-icons/vite`
   - keep existing Vue plugin and current aliases unchanged
3. Do not remove `mdue` yet.

Deliverable: project builds with both icon systems available.

### Phase 2 - Bulk import switch to direct MDI imports

1. Replace each `from 'mdue'` import with direct imports from `~icons/mdi/*` in each file.
2. Keep current local symbol names where helpful by aliasing defaults, e.g. `import Replay from '~icons/mdi/replay'`.
3. Resolve any non-obvious icon name differences (`Rewind_10`, `FastForward_10`, etc.) during migration.

Deliverable: app no longer imports from `mdue`; all icons come from direct `~icons/mdi/*` imports.

### Phase 3 - Visual and behavior validation

Verify critical UI areas that are icon-heavy:
- window controls (`App.vue`)
- playback controls (`NowPlaying.vue`, track items, volume)
- search/filter/modals (library + edit lyrics)
- loading and success/error state icons

Validation checklist:
- icon sizes align with current button/text sizing
- spinning/loading icons still animate as expected where CSS depends on classes
- no layout shift in dense controls (toolbars, row actions)

Deliverable: parity confirmed in desktop flows.

### Phase 4 - Remove legacy package

1. Remove `mdue` from `package.json`.
2. Reinstall dependencies and ensure lockfile updates are clean.
3. Run frontend build (`npm run build`).

Deliverable: migration complete, old package removed.

## Suggested execution order in this repo

1. Add plugin + icon set.
2. Mechanical direct import rewrite across components.
3. Build and smoke test key screens.
4. Remove `mdue` and re-run build.

## Risk areas and mitigations

- **Name mismatch risk**: not every `mdue` symbol has an obvious MDI equivalent.
  - Mitigation: keep a migration table in this plan and verify each non-obvious symbol in UI smoke tests.
- **Visual regressions**: some icons may look heavier/lighter than before.
  - Mitigation: adjust icon choices directly in affected files where parity is most important.
- **Future maintainability**: direct imports can drift over time.
  - Mitigation: document import convention (`~icons/mdi/*` only) in architecture docs and PR review checklist.

## Optional follow-up (after stable migration)

- Introduce lint rule/convention to forbid `mdue` imports permanently.
- Consider grouping repeated icon imports into local barrel files for high-density modules if readability drops.
- Consider auto-resolved components later (`unplugin-vue-components`) only if it clearly improves maintainability.
