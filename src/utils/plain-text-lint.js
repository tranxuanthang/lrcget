export const executeLint = (source) => {
  const lines = source.split('\n').map(line => line.trim())
  const problems = []

  // If there's only one line with content `[au: instrumental]`, that is still valid
  if (lines.length === 1 && lines[0].match(/^\[au: instrumental\]$/)) {
    return problems
  }

  lines.forEach((content, index) => {
    if (content.trim()) {
      // Check for lines starting with an opening square bracket
      if (content.match(/^\[/)) {
        problems.push({
          line: index + 1,
          severity: 'error',
          message: 'Line cannot start with an opening square bracket'
        })
      }
    } else {
      // Check for either: line at the first line OR 2 consecutive empty lines 
      if (
        (index === 0 && lines.length > 1) || 
        (index !== 0 && lines[index - 1].trim() === '')
      ) {
        problems.push({
          line: index + 1,
          severity: 'error',
          message: 'Unnecessary empty line'
        })
      }
    }
  })

  return problems
}