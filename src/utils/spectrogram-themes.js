import { rgb } from 'd3-color'
import { interpolateRgbBasis } from 'd3-interpolate'
import { interpolateInferno } from 'd3-scale-chromatic'

// Entries in each theme's color lookup table, which sets how many distinct colors get drawn.
export const SPECTROGRAM_LUT_SIZE = 256

// Samples a d3 interpolator into a flat RGB lookup table; rgb() parses both hex and rgb() strings.
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

// Black-floored ramps topping out pale, like inferno; d3's Blues/Greens run light to dark.
const BLUE_STOPS = ['#000004', '#0a2f6b', '#2a7fc0', '#7fd4f0', '#cdf0ff']
const GREEN_STOPS = ['#000400', '#0b3b16', '#1f9a3c', '#62d967', '#d6ffcc']

export const SPECTROGRAM_THEMES = Object.freeze([
  defineTheme('inferno', 'Inferno', '#fb9606', interpolateInferno),
  defineTheme('blue', 'Blue', '#317eb8', interpolateRgbBasis(BLUE_STOPS)),
  defineTheme('green', 'Green', '#38ae4b', interpolateRgbBasis(GREEN_STOPS)),
])

export const DEFAULT_SPECTROGRAM_THEME_ID = 'inferno'

const THEMES_BY_ID = new Map(SPECTROGRAM_THEMES.map(theme => [theme.id, theme]))

// Coerce persisted or unknown values to an id that is always renderable.
export const normalizeSpectrogramThemeId = id =>
  THEMES_BY_ID.has(id) ? id : DEFAULT_SPECTROGRAM_THEME_ID

export const getSpectrogramLut = id => THEMES_BY_ID.get(normalizeSpectrogramThemeId(id)).lut
