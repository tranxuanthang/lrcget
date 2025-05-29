export const countLines = (lines) => {
  return (lines.match(/\n/g) || []).length + 1
}
