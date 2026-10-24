import { rgb } from 'd3-color'
import { interpolateRgbBasis } from 'd3-interpolate'
import { interpolateInferno } from 'd3-scale-chromatic'

// Entry count of every theme's colour lookup table. The spectrogram render hot
// loop indexes it with the normalized magnitude, so this doubles as the
// quantization resolution of the colour axis.
export const SPECTROGRAM_LUT_SIZE = 256

// Sample any d3 `t => colour` interpolator into a flat RGB lookup table.
//
// This is the single colour-calculation path shared by every theme: a theme is
// just an interpolator plugged into this builder. `interpolateInferno` returns
// "#rrggbb" while `interpolateRgbBasis` returns "rgb(r, g, b)", so values are
// normalized through d3-color's `rgb()` rather than parsing either format by
// hand — that is what keeps one builder viable for all themes.
const buildLut = interpolate => {
  const lut = new Uint8Array(SPECTROGRAM_LUT_SIZE * 3)
  for (let i = 0; i < SPECTROGRAM_LUT_SIZE; i++) {
    const { r, g, b } = rgb(interpolate(i / (SPECTROGRAM_LUT_SIZE - 1)))
    lut[i * 3] = r
    lut[i * 3 + 1] = g
    lut[i * 3 + 2] = b
  }
  return lut
}

const defineTheme = (id, label, swatchColor, interpolate) =>
  Object.freeze({
    id,
    label,
    swatchColor,
    lut: buildLut(interpolate),
  })

// Blue and green are Aegisub-flavoured ramps: a black floor climbing through the
// hue to a pale — not white — peak, mirroring how inferno tops out at pale
// yellow rather than blowing out to white.
//
// They run through `interpolateRgbBasis`, the same builder d3-scale-chromatic
// uses for its own named ramps (`interpolateBlues` is exactly
// `interpolateRgbBasis(schemeBlues[9])`), so all three themes share one code
// path. d3's stock Blues/Greens are unusable here: they run light -> dark with
// no black anchor, inverting the loudness polarity a spectrogram needs.
const BLUE_STOPS = ['#000004', '#0a2f6b', '#2a7fc0', '#7fd4f0', '#cdf0ff']
const GREEN_STOPS = ['#000400', '#0b3b16', '#1f9a3c', '#62d967', '#d6ffcc']

export const SPECTROGRAM_THEMES = Object.freeze([
  defineTheme('inferno', 'Inferno', '#fb9606', interpolateInferno),
  defineTheme('blue', 'Blue', '#317eb8', interpolateRgbBasis(BLUE_STOPS)),
  defineTheme('green', 'Green', '#38ae4b', interpolateRgbBasis(GREEN_STOPS)),
])

export const DEFAULT_SPECTROGRAM_THEME_ID = 'inferno'

const THEMES_BY_ID = new Map(SPECTROGRAM_THEMES.map(theme => [theme.id, theme]))

export const isSpectrogramThemeId = id => THEMES_BY_ID.has(id)

// Coerce persisted or unknown values to an id that is always renderable.
export const normalizeSpectrogramThemeId = id =>
  THEMES_BY_ID.has(id) ? id : DEFAULT_SPECTROGRAM_THEME_ID

export const getSpectrogramTheme = id => THEMES_BY_ID.get(normalizeSpectrogramThemeId(id))

export const getSpectrogramLut = id => getSpectrogramTheme(id).lut
